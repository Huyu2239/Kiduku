# データベース（PostgreSQL）セットアップ

## 目的

`/check-reads` の対象メッセージと既読情報を、Discord履歴ではなくDBに保存して参照するためのセットアップ手順です。

## 起動（Docker Compose）

リポジトリ直下で以下を実行します。

```bash
docker compose up -d postgres
```

初回起動時に `docker/init/01_schema.sql` が適用されます。

## 接続先（DATABASE_URL）

デフォルトの接続先は以下です。

```bash
postgres://kiduku:kiduku@localhost:5432/kiduku
```

必要であれば環境変数で上書きします。

```bash
export DATABASE_URL=postgres://kiduku:kiduku@localhost:5432/kiduku
```

## 注意

- 本番用の認証情報はリポジトリに含めないでください。
- `GUILD_MEMBERS` Intent が無効だと、メンション対象の展開が不完全になります。
