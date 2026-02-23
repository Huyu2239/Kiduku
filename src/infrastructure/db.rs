use std::collections::HashMap;

use anyhow::Context as _;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{NoTls, Transaction};

pub type DbPool = Pool;

#[derive(Debug, Clone)]
pub struct Db {
    pool: DbPool,
}

#[derive(Debug, Clone)]
pub struct NewMention {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub author_id: u64,
    pub content: String,
    pub mention_everyone: bool,
    pub created_at_unix: i64,
    pub targets: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct StoredMention {
    pub author_id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub content: String,
    pub created_at_unix: i64,
    pub target_user_ids: Vec<u64>,
    pub read_user_ids: Vec<u64>,
    pub done_user_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct MentionForTarget {
    pub mention_id: i64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub author_id: u64,
    pub content: String,
    pub created_at_unix: i64,
    pub is_read: bool,
    pub is_done: bool,
    pub extended_until: Option<i64>,
}

impl Db {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let config: tokio_postgres::Config = database_url
            .parse()
            .context("DATABASE_URL の形式が不正です")?;
        let manager_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let manager = Manager::from_config(config, NoTls, manager_config);
        let pool = Pool::builder(manager)
            .max_size(16)
            .build()
            .context("PostgreSQL のプール構築に失敗しました")?;

        let client = pool.get().await.context("DB接続の取得に失敗しました")?;
        client
            .simple_query("SELECT 1")
            .await
            .context("DB接続の疎通確認に失敗しました")?;

        Ok(Self { pool })
    }

    pub async fn insert_mention(&self, mention: NewMention) -> anyhow::Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;
        let tx = client
            .transaction()
            .await
            .context("トランザクション開始に失敗しました")?;

        let mention_id = upsert_mention(&tx, &mention).await?;
        insert_targets(&tx, mention_id, &mention.targets).await?;

        tx.commit()
            .await
            .context("トランザクションのコミットに失敗しました")?;
        Ok(())
    }

    pub async fn record_read(
        &self,
        message_id: u64,
        user_id: u64,
        read_at_unix: i64,
    ) -> anyhow::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let mention_id = match client
            .query_opt(
                "SELECT id FROM mentions WHERE message_id = $1",
                &[&(message_id as i64)],
            )
            .await
            .context("メンションの検索に失敗しました")?
        {
            Some(row) => row.get::<_, i64>(0),
            None => return Ok(()),
        };

        client
            .execute(
                "INSERT INTO mention_reads (mention_id, user_id, read_at) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (mention_id, user_id) DO NOTHING",
                &[&mention_id, &(user_id as i64), &read_at_unix],
            )
            .await
            .context("既読情報の保存に失敗しました")?;

        Ok(())
    }

    pub async fn record_done(
        &self,
        message_id: u64,
        user_id: u64,
        done_at_unix: i64,
    ) -> anyhow::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let mention_id = match client
            .query_opt(
                "SELECT id FROM mentions WHERE message_id = $1",
                &[&(message_id as i64)],
            )
            .await
            .context("メンションの検索に失敗しました")?
        {
            Some(row) => row.get::<_, i64>(0),
            None => return Ok(()),
        };

        client
            .execute(
                "INSERT INTO mention_dones (mention_id, user_id, done_at) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (mention_id, user_id) DO NOTHING",
                &[&mention_id, &(user_id as i64), &done_at_unix],
            )
            .await
            .context("解決情報の保存に失敗しました")?;

        Ok(())
    }

    pub async fn fetch_mentions_for_author(
        &self,
        author_id: u64,
        since_unix: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<StoredMention>> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let rows = client
            .query(
                "SELECT id, guild_id, channel_id, message_id, author_id, content, created_at \
                 FROM mentions \
                 WHERE author_id = $1 AND created_at >= $2 \
                 ORDER BY created_at DESC \
                 LIMIT $3",
                &[&(author_id as i64), &since_unix, &limit],
            )
            .await
            .context("メンション一覧の取得に失敗しました")?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mention_ids = rows
            .iter()
            .map(|row| row.get::<_, i64>("id"))
            .collect::<Vec<_>>();

        let targets = fetch_user_ids_by_mention(&client, &mention_ids, "mention_targets").await?;
        let reads = fetch_user_ids_by_mention(&client, &mention_ids, "mention_reads").await?;
        let dones = fetch_user_ids_by_mention(&client, &mention_ids, "mention_dones").await?;

        let mut result = Vec::new();
        for row in rows {
            let mention_id = row.get::<_, i64>("id");
            let target_user_ids = targets.get(&mention_id).cloned().unwrap_or_default();
            let read_user_ids = reads.get(&mention_id).cloned().unwrap_or_default();
            let done_user_ids = dones.get(&mention_id).cloned().unwrap_or_default();
            result.push(StoredMention {
                author_id: row.get::<_, i64>("author_id") as u64,
                guild_id: row.get::<_, i64>("guild_id") as u64,
                channel_id: row.get::<_, i64>("channel_id") as u64,
                message_id: row.get::<_, i64>("message_id") as u64,
                content: row.get::<_, String>("content"),
                created_at_unix: row.get::<_, i64>("created_at"),
                target_user_ids,
                read_user_ids,
                done_user_ids,
            });
        }

        Ok(result)
    }

    pub async fn fetch_mention_by_message_id(
        &self,
        message_id: u64,
    ) -> anyhow::Result<Option<StoredMention>> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let row = match client
            .query_opt(
                "SELECT id, guild_id, channel_id, message_id, author_id, content, created_at \
                 FROM mentions WHERE message_id = $1",
                &[&(message_id as i64)],
            )
            .await
            .context("メンションの検索に失敗しました")?
        {
            Some(row) => row,
            None => return Ok(None),
        };

        let mention_id = row.get::<_, i64>("id");
        let mention_ids = vec![mention_id];

        let targets = fetch_user_ids_by_mention(&client, &mention_ids, "mention_targets").await?;
        let reads = fetch_user_ids_by_mention(&client, &mention_ids, "mention_reads").await?;
        let dones = fetch_user_ids_by_mention(&client, &mention_ids, "mention_dones").await?;

        Ok(Some(StoredMention {
            author_id: row.get::<_, i64>("author_id") as u64,
            guild_id: row.get::<_, i64>("guild_id") as u64,
            channel_id: row.get::<_, i64>("channel_id") as u64,
            message_id: row.get::<_, i64>("message_id") as u64,
            content: row.get::<_, String>("content"),
            created_at_unix: row.get::<_, i64>("created_at"),
            target_user_ids: targets.get(&mention_id).cloned().unwrap_or_default(),
            read_user_ids: reads.get(&mention_id).cloned().unwrap_or_default(),
            done_user_ids: dones.get(&mention_id).cloned().unwrap_or_default(),
        }))
    }

    pub async fn fetch_mentions_for_target(
        &self,
        user_id: u64,
        offset: i64,
        limit: i64,
        show_done: bool,
    ) -> anyhow::Result<Vec<MentionForTarget>> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let rows = client
            .query(
                "SELECT m.id, m.guild_id, m.channel_id, m.message_id, m.author_id, \
                        m.content, m.created_at, mt.extended_until, \
                        EXISTS(SELECT 1 FROM mention_reads \
                               WHERE mention_id = m.id AND user_id = $1) AS is_read, \
                        EXISTS(SELECT 1 FROM mention_dones \
                               WHERE mention_id = m.id AND user_id = $1) AS is_done \
                 FROM mentions m \
                 JOIN mention_targets mt ON m.id = mt.mention_id \
                 WHERE mt.user_id = $1 AND mt.ignored_at IS NULL \
                   AND ($4 OR NOT EXISTS(\
                        SELECT 1 FROM mention_dones \
                        WHERE mention_id = m.id AND user_id = $1)) \
                 ORDER BY m.created_at DESC \
                 LIMIT $2 OFFSET $3",
                &[&(user_id as i64), &limit, &offset, &show_done],
            )
            .await
            .context("被メンション一覧の取得に失敗しました")?;

        let result = rows
            .into_iter()
            .map(|row| MentionForTarget {
                mention_id: row.get::<_, i64>("id"),
                guild_id: row.get::<_, i64>("guild_id") as u64,
                channel_id: row.get::<_, i64>("channel_id") as u64,
                message_id: row.get::<_, i64>("message_id") as u64,
                author_id: row.get::<_, i64>("author_id") as u64,
                content: row.get::<_, String>("content"),
                created_at_unix: row.get::<_, i64>("created_at"),
                is_read: row.get::<_, bool>("is_read"),
                is_done: row.get::<_, bool>("is_done"),
                extended_until: row.get::<_, Option<i64>>("extended_until"),
            })
            .collect();

        Ok(result)
    }

    pub async fn extend_mention_for_user(
        &self,
        mention_id: i64,
        user_id: u64,
        extended_until: i64,
    ) -> anyhow::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        client
            .execute(
                "UPDATE mention_targets SET extended_until = $1 \
                 WHERE mention_id = $2 AND user_id = $3",
                &[&extended_until, &mention_id, &(user_id as i64)],
            )
            .await
            .context("延命情報の更新に失敗しました")?;

        Ok(())
    }

    pub async fn ignore_mention_for_user(
        &self,
        mention_id: i64,
        user_id: u64,
        ignored_at: i64,
    ) -> anyhow::Result<()> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        client
            .execute(
                "UPDATE mention_targets SET ignored_at = $1 \
                 WHERE mention_id = $2 AND user_id = $3",
                &[&ignored_at, &mention_id, &(user_id as i64)],
            )
            .await
            .context("無視情報の更新に失敗しました")?;

        Ok(())
    }

    /// 週次バッチ用: 未読かつ未DONEのターゲット (mention_id, user_id) を返す
    pub async fn fetch_unread_targets_for_weekly_batch(&self) -> anyhow::Result<Vec<(i64, u64)>> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let rows = client
            .query(
                "SELECT mt.mention_id, mt.user_id \
                 FROM mention_targets mt \
                 WHERE mt.ignored_at IS NULL \
                   AND NOT EXISTS(SELECT 1 FROM mention_reads \
                                  WHERE mention_id = mt.mention_id AND user_id = mt.user_id) \
                   AND NOT EXISTS(SELECT 1 FROM mention_dones \
                                  WHERE mention_id = mt.mention_id AND user_id = mt.user_id)",
                &[],
            )
            .await
            .context("週次バッチ用未読ターゲットの取得に失敗しました")?;

        let result = rows
            .into_iter()
            .map(|row| {
                let mention_id = row.get::<_, i64>("mention_id");
                let user_id = row.get::<_, i64>("user_id") as u64;
                (mention_id, user_id)
            })
            .collect();

        Ok(result)
    }

    /// 月次バッチ用: 期限切れ (created_at < cutoff かつ extended_until が NULL または < now)
    /// かつ未DONEのターゲット情報を返す
    pub async fn fetch_expiring_targets_for_monthly_batch(
        &self,
        cutoff_unix: i64,
        now_unix: i64,
    ) -> anyhow::Result<Vec<(u64, MentionForTarget)>> {
        let client = self
            .pool
            .get()
            .await
            .context("DB接続の取得に失敗しました")?;

        let rows = client
            .query(
                "SELECT m.id, m.guild_id, m.channel_id, m.message_id, m.author_id, \
                        m.content, m.created_at, mt.extended_until, mt.user_id, \
                        EXISTS(SELECT 1 FROM mention_reads \
                               WHERE mention_id = m.id AND user_id = mt.user_id) AS is_read, \
                        EXISTS(SELECT 1 FROM mention_dones \
                               WHERE mention_id = m.id AND user_id = mt.user_id) AS is_done \
                 FROM mentions m \
                 JOIN mention_targets mt ON m.id = mt.mention_id \
                 WHERE m.created_at < $1 \
                   AND mt.ignored_at IS NULL \
                   AND (mt.extended_until IS NULL OR mt.extended_until < $2) \
                   AND NOT EXISTS(SELECT 1 FROM mention_dones \
                                  WHERE mention_id = m.id AND user_id = mt.user_id) \
                 ORDER BY m.created_at DESC",
                &[&cutoff_unix, &now_unix],
            )
            .await
            .context("月次バッチ用期限切れターゲットの取得に失敗しました")?;

        let result = rows
            .into_iter()
            .map(|row| {
                let target_user_id = row.get::<_, i64>("user_id") as u64;
                let item = MentionForTarget {
                    mention_id: row.get::<_, i64>("id"),
                    guild_id: row.get::<_, i64>("guild_id") as u64,
                    channel_id: row.get::<_, i64>("channel_id") as u64,
                    message_id: row.get::<_, i64>("message_id") as u64,
                    author_id: row.get::<_, i64>("author_id") as u64,
                    content: row.get::<_, String>("content"),
                    created_at_unix: row.get::<_, i64>("created_at"),
                    is_read: row.get::<_, bool>("is_read"),
                    is_done: row.get::<_, bool>("is_done"),
                    extended_until: row.get::<_, Option<i64>>("extended_until"),
                };
                (target_user_id, item)
            })
            .collect();

        Ok(result)
    }
}

async fn upsert_mention(tx: &Transaction<'_>, mention: &NewMention) -> anyhow::Result<i64> {
    if let Some(row) = tx
        .query_opt(
            "SELECT id FROM mentions WHERE message_id = $1",
            &[&(mention.message_id as i64)],
        )
        .await
        .context("既存メンションの検索に失敗しました")?
    {
        return Ok(row.get::<_, i64>(0));
    }

    let row = tx
        .query_one(
            "INSERT INTO mentions \
             (guild_id, channel_id, message_id, author_id, content, mention_everyone, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id",
            &[
                &(mention.guild_id as i64),
                &(mention.channel_id as i64),
                &(mention.message_id as i64),
                &(mention.author_id as i64),
                &mention.content,
                &mention.mention_everyone,
                &mention.created_at_unix,
            ],
        )
        .await
        .context("メンションの保存に失敗しました")?;

    Ok(row.get::<_, i64>(0))
}

async fn insert_targets(
    tx: &Transaction<'_>,
    mention_id: i64,
    targets: &[u64],
) -> anyhow::Result<()> {
    for user_id in targets {
        tx.execute(
            "INSERT INTO mention_targets (mention_id, user_id) \
             VALUES ($1, $2) \
             ON CONFLICT (mention_id, user_id) DO NOTHING",
            &[&mention_id, &(*user_id as i64)],
        )
        .await
        .context("メンション対象者の保存に失敗しました")?;
    }
    Ok(())
}

async fn fetch_user_ids_by_mention(
    client: &tokio_postgres::Client,
    mention_ids: &[i64],
    table: &str,
) -> anyhow::Result<HashMap<i64, Vec<u64>>> {
    let sql = format!(
        "SELECT mention_id, user_id FROM {} WHERE mention_id = ANY($1)",
        table
    );
    let rows = client
        .query(&sql, &[&mention_ids])
        .await
        .with_context(|| format!("{} の取得に失敗しました", table))?;

    let mut map: HashMap<i64, Vec<u64>> = HashMap::new();
    for row in rows {
        let mention_id = row.get::<_, i64>("mention_id");
        let user_id = row.get::<_, i64>("user_id") as u64;
        map.entry(mention_id).or_default().push(user_id);
    }
    for users in map.values_mut() {
        users.sort_unstable();
        users.dedup();
    }
    Ok(map)
}
