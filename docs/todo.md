# 実装状況 - Kiduku

> **注記**: MVP ToDo として作成したリストを実装完了後に更新。
> 当初の設計から変更・追加された機能については末尾の「設計との乖離」を参照。

---

## Phase 1: ドメイン層・基盤実装

### [x] 1.1 ドメインモデル実装

- [x] 1.1.1 `domain/model/message.rs` - Message Entity実装
- [x] 1.1.2 `domain/model/mod.rs` - MentionType Value Object実装
- [x] 1.1.3 `domain/policy/mention_detection.rs` - メンション検出ロジック

### [x] 1.2 ユースケース層基盤

- [x] 1.2.1 `usecase/mod.rs` - ユースケース基盤

## Phase 2: コマンド実装

### [x] 2.0 DTO定義

- [x] 2.0.1 入力DTO実装 (`MessageInputDto` など)
- [x] 2.0.2 出力DTO実装 (`AddReadReactionOutputDto` など)

### [x] 2.1 /help コマンド実装

- [x] 2.1.1 `usecase/slash_commands/help.rs` - Usecase ロジック
- [x] 2.1.2 `presentation/entry/slash_commands/help.rs` - イベントハンドラー

### [x] 2.2 /check-reads コマンド → 実装形式を変更

> **設計変更**: `/check-reads` スラッシュコマンドではなく、コンテキストメニューコマンド
> 「既読状況確認」として実装。DBに保存済みのメンション情報をもとに既読状況を表示。

- [x] `presentation/entry/slash_commands/view_read_status.rs` - 既読状況コンテキストメニュー
- [x] `usecase/slash_commands/view_read_status.rs` - Usecase ロジック

## Phase 3: メッセージハンドリング

### [x] 3.1 メッセージイベントハンドラー実装

- [x] 3.1.1 `usecase/on_message/auto_add_read_reaction.rs`
- [x] 3.1.2 `presentation/entry/on_message.rs`

### [x] 3.2 リアクション監視（MVP では任意 → 実装済み）

- [x] 3.2.1 `presentation/entry/on_reaction_add.rs` - KIDOKU/DONE リアクション記録

## Phase 4: Presentation層の完成 & 環境設定

### [x] 4.1 Presentation層の完成

- [x] 4.1.1 `presentation/discord_exec.rs` - Serenityクライアント初期化
- [x] 4.1.2 `presentation/entry/on_message.rs` - メッセージハンドラー実装

### [x] 4.2 インフラストラクチャ層

- [x] 4.2.1 `infrastructure/config/mod.rs` - 環境変数読み込み
- [x] `infrastructure/db.rs` - PostgreSQL接続・CRUD実装（Phase 2以降として設計していたが前倒し実装）

### [x] 4.3 Botエントリーポイント設定

- [x] 4.3.1 `src/main.rs` - 完全実装

## Phase 5: テスト・検証

### [x] 5.1 ユニットテスト（Domain + Usecase層）

- [x] 5.1.1 `domain/policy/mention_detection.rs` - テスト実装済み
- [x] 5.1.2 `usecase/on_message/auto_add_read_reaction.rs` - テスト実装済み
- [x] 5.1.3 既読状況確認ロジック - `usecase/slash_commands/view_read_status.rs` はテスト対象（未実装）

### [ ] 5.2 E2Eテスト（手動/Discord テストサーバー）

未実施。

## Phase 6: ドキュメント・デプロイ準備

### [ ] 6.1 ドキュメント

- [ ] `docs/development/setup.md` - 開発環境セットアップ
- [ ] `docs/usage/commands.md` - ユーザードキュメント

### [ ] 6.2 デプロイ準備

- [ ] Dockerfile
- [ ] docker-compose.yml

## Phase 7: 最終確認・リリース

未着手。

---

## 設計との乖離（MVP → 実装）

| 設計 | 実装 | 備考 |
|------|------|------|
| DB なし（MVP は永続化不要） | PostgreSQL に全メンションを保存 | 機能要件の拡張により必要になった |
| `/check-reads` スラッシュコマンド | コンテキストメニュー「既読状況確認」 | UX上こちらの方が自然なため変更 |
| 未実装（Phase 2以降） | `/通知一覧` スラッシュコマンド（ページネーション付き） | 追加機能 |
| 未実装 | 週次バッチ通知（毎週月曜8時 JST、DM送信） | 追加機能 |
| 未実装 | 月次バッチ通知（毎月第1月曜、期限切れメンション通知） | 追加機能 |
| 未実装 | 延命・無視ボタン（コンポーネントインタラクション） | 追加機能 |
