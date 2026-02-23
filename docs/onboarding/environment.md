# 開発環境変数

## セットアップ

開発時は `.envrc` で環境変数を設定します。

```bash
# .envrc.example をコピー
cp .envrc.example .envrc

# .envrc を編集（以下の必須変数を設定）
vi .envrc

# direnv を許可
direnv allow
```

## 必須環境変数

### `DISCORD_BOT_TOKEN` (必須)

Discord Developer Portal で取得した Bot token。

- 長さ: 59-67文字程度
- **注意**: クレデンシャルのため、`.gitignore` に含め、リポジトリにコミットしない

## オプション環境変数

### `LOG_LEVEL` / `RUST_LOG`

ログレベルを設定（ドメイン指定も可能）。

```bash
# 例
export RUST_LOG=debug
export RUST_LOG=kiduku=debug,serenity=info
```

### `DEV_MODE`

開発モードの有効/無効（`true` / `false`）。

- `true` の場合: ログ出力にファイル名・行番号を含める
- デフォルト: `cargo run` 時は自動で `true`

### `SHARD_COUNT`

Discord Gateway シャード数（小規模開発は `0` で OK）。

```bash
export SHARD_COUNT=0  # シャードなし
```

## 優先順位

1. `.envrc` での明示的な設定
2. `.env` ファイル（存在する場合、dotenvy により自動読み込み）
3. システム環境変数
4. ハードコードされた既定値

## direnv の使用

`.envrc` を保存すると、自動的に環境変数が読み込まれます。

```bash
# .envrc の許可
direnv allow .envrc

# 確認
direnv status

# 明示的に再読み込み
direnv reload
```

**注意**: direnv をインストールしていない場合は、手動で `source .envrc` を実行してください。
