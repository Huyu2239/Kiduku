# 環境変数

開発時は `.envrc` で環境変数を設定する。

## 使用する変数

- `DISCORD_BOT_TOKEN`: Discord Bot のトークン（必須）。
- `LOG_LEVEL`: ログレベル（例: `info`, `debug`）。設定時は `RUST_LOG` より優先する。
- `RUST_LOG`: `LOG_LEVEL` が未指定のときに利用する。

## 備考

- `cargo run` で起動した場合は開発モードが有効になり、既定のログレベルが `debug` になる。

## 優先順位

1. `LOG_LEVEL`
2. `RUST_LOG`
3. 既定値（`debug` / `info`）
