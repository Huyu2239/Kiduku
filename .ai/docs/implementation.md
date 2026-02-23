# 実装状況ドキュメント

設計ドキュメント（`docs/features/dd/mvp.md`）に対する現在の実装状況と乖離点をまとめる。

## 実装済み機能

### イベントハンドラー
- **メッセージ受信** (`presentation/entry/on_message.rs`): メンション検出 → KIDOKU/DONE リアクション付与 → DB 保存
- **リアクション追加** (`presentation/entry/on_reaction_add.rs`): KIDOKU/DONE リアクションを DB に記録
- **コンポーネントインタラクション** (`presentation/entry/on_component.rs`): ページネーション・延命・無視ボタン処理

### スラッシュコマンド
- `/help` — Bot 使用方法を表示
- `/通知一覧 [show_done]` — 自分宛てメンション一覧（5件/ページ、ページネーション付き）
- コンテキストメニュー「既読状況確認」— 選択したメッセージの既読ユーザー・未読ユーザー一覧

### バッチ処理 (`presentation/entry/batch.rs`)
- **週次バッチ**: 毎週月曜 8:00 JST、未読・未解決メンションがあるユーザーに DM 通知
- **月次バッチ**: 毎月第1月曜 8:00 JST、1ヶ月以上経過した未解決メンションに延命/無視ボタン付き DM

## 設計からの主な乖離

1. **DB 永続化が MVP から有効**
   設計では Phase 2 以降だったが、全機能が DB に依存するため最初から実装した。

2. **`/check-reads` → コンテキストメニュー「既読状況確認」**
   過去N時間のメッセージ一括チェックではなく、特定メッセージのコンテキストメニューとして実装。

3. **`/通知一覧` コマンドの追加**
   設計に存在しない新機能。自分宛てメンション一覧をページネーション付きで表示。

4. **延命・無視機能の追加**
   設計に存在しない新機能。月次バッチ通知と連動。

## 重要なファイル・定数

### 絵文字定数
`presentation/entry/util.rs` に集約:
- `KIDOKU_EMOJI_ID = 1475281418400698633` (「既読」リアクション)
- `DONE_EMOJI_ID = 1475281416370524414` (「解決済み」リアクション)

絵文字の名前 (`KIDOKU_EMOJI_NAME`, `DONE_EMOJI_NAME`) も同ファイルにある。
絵文字 ID を変更する場合はこのファイルのみ修正すればよい。

### ページネーション
- `PAGE_SIZE = 5` (`presentation/entry/slash_commands/my_mentions.rs`)
- DB クエリ側で `show_done` フィルタ済み（`infrastructure/db.rs:fetch_mentions_for_target`）
- `PAGE_SIZE + 1` 件取得して「次ページあり」を判定する設計

### バッチスケジューリング
- 次の月曜 8:00 JST まで `tokio::time::sleep` で待機するシンプルな実装
- `JST_OFFSET_SECS = 9 * 3600` で UTC ↔ JST 変換

### カスタムコンポーネント ID フォーマット
- ページネーション: `mm:p:{page}:{show_done_01}:{user_id}`
- 延命: `mm:extend:{mention_id}:{user_id}`
- 無視: `mm:ignore:{mention_id}:{user_id}`

## テストカバレッジ

ユニットテスト済みの層:
- `domain/policy/` — `greeting`, `mention_detection`, `read_status_calc`
- `usecase/on_message/` — `auto_add_read_reaction`, `greeting`
- `usecase/dto/output/discord_exec.rs`
- `infrastructure/config/mod.rs`

未テスト:
- `infrastructure/db.rs` (DB 操作)
- `presentation/` 全体 (Discord API 依存)
