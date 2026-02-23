# MVP DD - Kiduku

## 1. 設計概要

Discordのイベント駆動アーキテクチャを活用。ボットがメッセージイベントをリッスンし、メンション検出時に自動的にリアクションを付与する設計。

## 2. システムアーキテクチャ

### 全体構成

```
Discord API ←→ Serenity (Discord Client)
                    ↓
            Presentation層
            ├─ MessageCreate → イベント受信・DTO生成
            ├─ InteractionCreate → コマンド受信・DTO生成
            └─ Discord操作（リアクション付与、メッセージ送信）
                    ↓
            Interface層 (Mapper)
            ├─ DTO → Domain変換
            └─ Domain → DTO変換
                    ↓
            Usecase層
            ├─ ビジネスロジック実行
            └─ DTO形式で結果返却
                    ↓
            Domain層
            ├─ エンティティ
            ├─ ドメインロジック
            └─ ポリシー
```

### 層構成

プロジェクトの既存アーキテクチャ（DDD + Clean Architecture）に従う：

```
presentation/     → Discord イベント・コマンド受付、Serenity操作
                   (すべてのDiscord APIアクセスはこの層で行う)
interface/        → 入力/出力マッピング（DTO ↔ Domain）
usecase/          → ビジネスロジック（入力：DTO, 出力：DTO）
domain/           → ドメインモデル・ビジネスロジック
infrastructure/   → 環境設定・ロギング初期化など
```

## 3. ドメインモデル

### Entity: Message

メンション情報を含むメッセージを表現。

```rust
// domain/model/message.rs
use serenity::model::prelude::*;
use chrono::Utc;

pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub content: String,
    pub user_mentions: Vec<UserId>,      // @user メンション対象
    pub role_mentions: Vec<RoleId>,      // @role メンション対象
    pub mentions_everyone: bool,         // @everyone/@here フラグ
    pub created_at: DateTime<Utc>,
}

impl Message {
    /// メンションを含むかを判定
    pub fn has_mention(&self) -> bool {
        !self.user_mentions.is_empty()
            || !self.role_mentions.is_empty()
            || self.mentions_everyone
    }
}
```

### Value Object: MentionType

メンションの種類を表現。

```rust
// domain/model/mod.rs または独立ファイル
pub enum MentionType {
    User(UserId),
    Role(RoleId),
    Everyone,
    Here,
}
```

### Policy: メンション検出ロジック

```rust
// domain/policy/mention_detection.rs
use super::super::model::{Message, MentionType};

pub fn extract_mentions(msg: &Message) -> Vec<MentionType> {
    let mut mentions = Vec::new();

    // ユーザーメンション
    for user_id in &msg.user_mentions {
        mentions.push(MentionType::User(*user_id));
    }

    // ロールメンション
    for role_id in &msg.role_mentions {
        mentions.push(MentionType::Role(*role_id));
    }

    // @everyone/@here（それぞれ MentionType を生成）
    if msg.mentions_everyone {
        mentions.push(MentionType::Everyone);
    }

    mentions
}

pub fn should_add_read_reaction(msg: &Message) -> bool {
    msg.has_mention()
}
```

### 注記: ReadState について

MVP では ReadState Entity の永続化は **不要** です。
- 既読ユーザー = `message.reactions` に ✅を追加したユーザー（Discordが管理）
- 未読ユーザー = メンション対象 - 既読ユーザー（計算で求める）
- 保存の必要なし

Phase 2 以降で分析機能が必要になった場合、ReadState の永続化を検討してください。

## 4. ユースケース設計

### UC01: 既読リアクション自動付与

**トリガー**: MessageCreateイベント

**処理フロー**:

1. **Presentation層**: イベント受信・変換
   - Serenityの `Message` オブジェクトを受け取り
   - DTO (`MessageInputDto`) に変換

2. **Interface層**: DTO → Domain変換
   - `InputMapper` で DTO をドメインモデルに変換

3. **Usecase層**: `AutoAddReadReactionUseCase` 実行
   - **入力**: `MessageInputDto`
   - **処理**: メンション検出ロジック実行
   - **出力**: `AddReadReactionOutputDto` (リアクション付与対象メッセージID)

4. **Presentation層**: Discord操作実行
   - 出力DTO から messageId を取得
   - Serenity APIを使用してリアクション付与 (`message.react()`)
   - **例外処理**: 権限不足、既に追加済みなどをキャッチしてログ出力

**ボット自身のメッセージ除外**:
- Presentation層でチェック（イベント受信時に `message.author.id == bot_id` を確認）

### リアクション監視について（MVP では不要）

Presentation層の ReactionAdd イベントハンドラーは MVP では **オプション** です。
- 用途: リアクション追加がされたログを出力するのみ
- 機能提供なし（既読情報は `/check-reads` コマンドで確認）
- Phase 2 以降で「既読ユーザーへの通知」が必要になった際に実装推奨

### UC02: 未読ユーザー確認

**トリガー**: `/check-reads`コマンド実行

**処理フロー**:

1. **Presentation層**: インタラクション受信・コマンドパース
   - Serenityの `Interaction` オブジェクトを受け取り
   - コマンド引数をパース (`hours` パラメータ取得)
   - ユーザーIDから過去Nhours内のメッセージを Serenity APIで取得
   - DTO (`CheckReadsInputDto`) に変換

2. **Interface層**: DTO → Domain変換
   - `InputMapper` で DTO をドメインモデルに変換

3. **Usecase層**: `GetReadStatusUseCase` 実行
   - **入力**: `CheckReadsInputDto` (メッセージリスト、メンション情報)
   - **処理**:
     - 各メッセージのメンション対象者を抽出
     - 各メッセージのリアクション追加者を特定
     - 既読/未読ユーザーを分類
   - **出力**: `CheckReadsOutputDto[]` (各メッセージの既読状況)

4. **Presentation層**: Discord操作・メッセージ表示
   - 出力DTO から結果を取得
   - Embed メッセージに整形
   - Serenity APIで一時的メッセージ(ephemeral)として返信

**データ取得方法**:
- Presentation層で、Serenity APIを使用してメッセージとリアクション情報を取得

**メンション対象者の取得**:
  - ユーザーメンション: `message.mentions` から UserId を直接取得
  - ロールメンション:
    - `message.role_mentions` から RoleId を取得
    - ロール内メンバーを展開: `guild.roles.get(role_id).members()` で全メンバー UserId を取得
    - Presentation層で展開した UserId を CheckReadsInputDto に含める
  - @everyone/@here:
    - `message.mentions_everyone == true` の場合
    - チャンネル内全メンバーを対象: `channel.members()` で取得（Usecase側で処理）

**リアクション追加者の取得**:
  - `message.reactions` イテレーション
  - ✅ リアクション（EmojiはUnicode "✅"）を特定
  - 当該リアクションに対して `reaction.users()` を呼び出し
  - 結果をVec<UserId>に変換して CheckReadsInputDto に含める

## 5. 主要モジュール構成

### presentation/

```
presentation/
├── discord_exec.rs        # Serenityクライアント初期化・ハンドラー登録
├── entry/
│   ├── mod.rs
│   ├── on_message.rs      # MessageCreateイベント処理
│   │                       # - メッセージ受信
│   │                       # - DTO生成
│   │                       # - Usecase呼び出し
│   │                       # - リアクション付与実行
│   ├── on_error.rs        # エラーハンドリング
│   └── slash_commands/
│       ├── mod.rs
│       ├── check_reads.rs # /check-reads コマンド実装
│       │                  # - インタラクション受信
│       │                  # - Serenity APIでメッセージ取得
│       │                  # - DTO生成
│       │                  # - Usecase呼び出し
│       │                  # - 結果をEmbedに整形
│       │                  # - 一時的メッセージで返信
│       └── help.rs        # /help コマンド実装
│                          # - ヘルプEmbedの生成・返信
└── mod.rs
```

### interface/

```
interface/
├── mapper/
│   ├── input_mapper.rs    # DTO → Domain変換
│   │                       # - MessageInputDto → Message
│   │                       # - CheckReadsInputDto → CheckReadsData
│   └── output_mapper.rs   # Domain → DTO変換（必要に応じて）
└── mod.rs
```

### usecase/

```
usecase/
├── on_message/
│   ├── auto_add_read_reaction.rs  # UC01実装
│   │                              # 入力: MessageInputDto
│   │                              # 出力: AddReadReactionOutputDto
│   └── mod.rs
├── slash_commands/
│   ├── check_reads.rs     # UC02実装
│   │                       # 入力: CheckReadsInputDto
│   │                       # 出力: CheckReadsOutputDto[]
│   └── help.rs            # ヘルプ表示（Presentation層で処理）
├── dto/
│   ├── input/
│   │   └── mod.rs         # MessageInputDto, CheckReadsInputDto等
│   └── output/
│       └── mod.rs         # AddReadReactionOutputDto, CheckReadsOutputDto等
└── mod.rs
```

### domain/

```
domain/
├── model/
│   ├── message.rs         # Message Entity
│   ├── read_state.rs      # ReadState Entity
│   └── mod.rs
├── policy/
│   ├── mention_detection.rs   # メンション検出ロジック
│   ├── read_status_calc.rs    # 既読状態計算ロジック
│   └── mod.rs
└── mod.rs
```

### infrastructure/

```
infrastructure/
├── config/
│   ├── mod.rs             # 環境変数読み込み（TOKEN, GUILD_ID等）
│   │                       # 構体: AppConfig { discord_token, guild_id, log_level }
│   └── logging.rs         # ロギング初期化（tracing-subscriber設定）
└── mod.rs                 # infrastructure公開API

注記：
- MVP では Serenity API アクセスは一切行わない（Presentation層で実行）
- 永続化、キャッシュ、リポジトリは不要（Phase 2以降で導入予定）
```

## 6. APIインターフェース

### Presentation層

**MessageCreateイベントハンドラー**

```rust
// presentation/entry/on_message.rs
async fn message_create_handler(
    ctx: &Context,
    msg: &Message
) -> Result<(), SerenityError>
```

責務:
- ボット自身のメッセージを除外
- メッセージを `MessageInputDto` に変換
- Usecase呼び出し
- 結果に基づいてリアクション付与 (`msg.react()`)
- Discord API例外処理（権限不足など）

**InteractionCreateイベントハンドラー**

```rust
// presentation/entry/slash_commands/check_reads.rs
async fn check_reads_command_handler(
    ctx: &Context,
    interaction: &CommandInteraction
) -> Result<(), SerenityError>
```

責務:
- コマンド引数をパース
- Serenity APIでメッセージ取得
  - `channel.messages()` で過去N時間のメッセージを取得
  - フィルタリング: 実行ユーザーが送信したメッセージのみ抽出
  - メンション判定: メンション（`mentions`, `role_mentions`, `mentions_everyone`）を含むメッセージのみ対象
- メッセージ・リアクション情報を `CheckReadsInputDto` に変換
  - 各メッセージの既読リアクション（✅）を検出
  - リアクション追加者を取得: `message.reactions.iter()` → ✅を検出 → `reaction.users()` で UserId 取得
  - ロールメンション時は Usecase層で展開を指示（CheckReadsInputDto に role_mentions として RoleId を含める）
- Usecase呼び出し
- 結果をEmbedメッセージに整形
- 一時的メッセージで返信

### Usecase層

### DTO詳細定義

**usecase/dto/input/mod.rs:**

```rust
use serenity::model::prelude::{UserId, RoleId, MessageId, ChannelId};

/// UC01: 既読リアクション自動付与の入力
pub struct MessageInputDto {
    pub message_id: MessageId,
    pub channel_id: ChannelId,
    pub content: String,
    pub user_mentions: Vec<UserId>,      // @user メンション
    pub role_mentions: Vec<RoleId>,      // @role メンション
    pub mentions_everyone: bool,         // @everyone/@here
}

/// UC02: 既読状況確認の入力
pub struct CheckReadsInputDto {
    pub user_id: UserId,                 // コマンド実行者
    pub messages: Vec<MessageWithReactionsDto>,
    pub hours: i64,                      // 過去何時間を対象にするか
}

pub struct MessageWithReactionsDto {
    pub message_id: MessageId,
    pub content: String,
    pub user_mentions: Vec<UserId>,
    pub role_mentions: Vec<RoleId>,      // ロール内メンバー展開前のロールID
    pub mentions_everyone: bool,
    pub reaction_user_ids: Vec<UserId>,  // ✅リアクション追加者
}
```

**usecase/dto/output/mod.rs:**

```rust
use serenity::model::prelude::{UserId, MessageId};

/// UC01: 既読リアクション自動付与の出力
pub struct AddReadReactionOutputDto {
    pub message_id: MessageId,
    pub should_add_reaction: bool,       // true ならば Presentation層で リアクション付与
}

/// UC02: 既読状況確認の出力
pub struct CheckReadsOutputDto {
    pub message_id: MessageId,
    pub message_content: String,
    pub mentioned_users: Vec<UserId>,    // メンション対象者（ロール展開後）
    pub read_users: Vec<UserId>,         // リアクション追加者
    pub unread_users: Vec<UserId>,       // mentioned_users - read_users
}

pub enum UsecaseError {
    NoMentionedUsers,
    NoMessages,
    RoleNotFound(RoleId),
    InvalidHoursParameter,
    MessageIdNotFound(MessageId),
    InvalidMentionData,
}
```

### Usecase層の関数シグネチャ

**UC01: 既読リアクション自動付与**

```rust
// usecase/on_message/auto_add_read_reaction.rs
pub fn execute(
    input: MessageInputDto
) -> Result<AddReadReactionOutputDto, UsecaseError>
```

処理: メンション検出 → メンション対象者が存在すれば `should_add_reaction: true` を返却

**UC02: 既読状況確認**

```rust
// usecase/slash_commands/check_reads.rs
pub fn execute(
    input: CheckReadsInputDto
) -> Result<Vec<CheckReadsOutputDto>, UsecaseError>
```

処理:
- 各メッセージのメンション対象者を確定（ロール展開）
- リアクション追加者との差分を計算
- 未読ユーザーリストを生成

## 7. エラーハンドリング

### エラー分類

**Usecase層**:
- **DomainError**: ビジネスロジック上のエラー
  - `NoMentionedUsers`: メッセージがメンション対象者を含まない
  - `NoMessages`: 指定条件に合致するメッセージが存在しない
  - `RoleNotFound`: メンション対象のロールが見つからない

- **ValidationError**: 入力値の不正
  - `InvalidHoursParameter`: `hours` パラメータが負数または非数値
  - `MessageIdNotFound`: 指定メッセージIDが存在しない（内部的に取得失敗時）
  - `InvalidMentionData`: メンション情報の形式が不正

**Presentation層**:
- **SerenityError**: Discord API呼び出しのエラー
  - 権限不足 (`HttpError::UnknownRole` など)
  - レート制限 (`HttpError::RatelimitBin`)
  - ネットワークエラー

### エラー戦略

**Usecase層**:
- `Result<OutputDto, UsecaseError>` で返却
- 例外は発生させない（Result型で表現）

**Presentation層**:
- Usecase実行結果をチェック
  - Ok → 出力に基づいて処理
  - Err → ログ出力、ユーザーに通知

- Discord API操作時の例外処理
  - リアクション付与失敗 → ログ出力、処理続行
  - メッセージ取得失敗 → ユーザーにエラーメッセージで返信

- ユーザーへの通知
  - Embed メッセージで友好的なエラーメッセージを表示

## 8. 非機能設計

### ログ出力

- `tracing` を使用
- イベント処理: `debug`レベル
- コマンド実行: `info`レベル
- エラー: `error`レベル

### パフォーマンス

- メッセージ処理はイベントハンドラー内で非同期実行
- リアクション付与はbatch操作を避ける（1メッセージ1操作）
- キャッシュ: 必要に応じて直近メッセージをメモリ保持

### スケーリング

- 複数サーバー対応: サーバーIDでスコープ分離
- シャーディング: Serenityの自動シャーディング機能を活用

## 9. テスト設計

### Domain層 ユニットテスト

```rust
// tests/domain/policy/mention_detection_test.rs
#[test]
fn test_should_add_read_reaction_with_user_mention() {
    // user_mentions が空でない場合、true を返す
}

#[test]
fn test_should_add_read_reaction_with_role_mention() {
    // role_mentions が空でない場合、true を返す
}

#[test]
fn test_should_add_read_reaction_with_everyone_mention() {
    // mentions_everyone = true の場合、true を返す
}

#[test]
fn test_extract_mentions_all_types() {
    // ユーザー・ロール・everyone の混合メンションを正確に抽出
}
```

### Usecase層 ユニットテスト

```rust
// tests/usecase/on_message/auto_add_read_reaction_test.rs
#[test]
fn test_auto_add_read_reaction_with_mention() {
    let input = MessageInputDto { /* with mentions */ };
    let result = auto_add_read_reaction::execute(input);
    assert_eq!(result.unwrap().should_add_reaction, true);
}

#[test]
fn test_auto_add_read_reaction_without_mention() {
    let input = MessageInputDto { /* no mentions */ };
    let result = auto_add_read_reaction::execute(input);
    assert_eq!(result.unwrap().should_add_reaction, false);
}

// tests/usecase/slash_commands/check_reads_test.rs
#[test]
fn test_check_reads_calculates_unread_users() {
    // mentioned_users - read_users = unread_users の計算確認
}

#[test]
fn test_check_reads_with_role_mentions_expands_members() {
    // ロールメンション展開の確認
}
```

### Presentation層 E2E テスト（手動確認）

Discord テストサーバーでの手動確認：

1. **自動リアクション付与確認**
   - ユーザーメンション時にリアクション付与 → 確認
   - ロールメンション時にリアクション付与 → 確認
   - @everyone/@here 時にリアクション付与 → 確認
   - 通常メッセージ（メンションなし）はリアクション付与されない → 確認

2. **/check-reads コマンド確認**
   - `/check-reads` 実行 → 既読状況を表示 → 確認
   - `/check-reads 12` 実行 → 過去12時間のメッセージを対象 → 確認
   - 複数ユーザーがリアクション追加 → 既読ユーザー一覧が表示 → 確認
   - 未読ユーザーが表示される → 確認

3. **/help コマンド確認**
   - `/help` 実行 → ボット説明とコマンド一覧が表示 → 確認

4. **エラーハンドリング確認**
   - Bot に リアクション付与権限がない → エラーメッセージ表示 → 確認
   - Discord API が レート制限 → 処理が安全に失敗 → 確認

## 10. デプロイ・運用

### 環境変数

Infrastructure層（`infrastructure/config/mod.rs`）で読み込み:

```bash
# 必須
DISCORD_BOT_TOKEN=<your_bot_token_here>  # Discord Developer Portal から取得

# オプション
DISCORD_GUILD_ID=123456789  # スラッシュコマンド登録用（開発時のみ）
RUST_LOG=kiduku=debug,serenity=info  # ログレベル設定
```

### ログ出力

`tracing` crate を使用:

```rust
// infrastructure/config/logging.rs の初期化例
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
```

ログレベルガイドライン:
- Presentation層：
  - `debug!`: イベント受信（MessageCreate, InteractionCreate）
  - `info!`: コマンド実行（/check-reads, /help）
  - `error!`: Discord API エラー

- Usecase層：
  - `debug!`: ビジネスロジック実行
  - `error!`: DomainError, ValidationError

### デプロイ

**ローカル開発:**
```bash
export DISCORD_BOT_TOKEN=<your_token>
cargo run
```

**Docker:**
```dockerfile
FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/kiduku /usr/local/bin/
CMD ["kiduku"]
```

起動:
```bash
docker run -e DISCORD_BOT_TOKEN=<token> -e RUST_LOG=info kiduku
```

**運用:**
- ボットは無限ループで稼働（Serenity の自動リコネクション機能で再接続）
- ログは stdout 出力（journalctl や外部ログシステムで集約推奨）
- 定期的な再起動は不要（メモリリークテスト後に判断）
