# 実装ToDo - Kiduku MVP

実装優先度順の作業リスト。

## Phase 1: ドメイン層・基盤実装

### [ ] 1.1 ドメインモデル実装

- [ ] 1.1.1 `domain/model/message.rs` - Message Entity実装
  - 構造体: `Message { id, channel_id, author_id, content, user_mentions, role_mentions, mentions_everyone }`
  - メソッド: `has_mention()` - メンション含有判定
  - 依存: Serenity 型 (MessageId, ChannelId, UserId, RoleId)

- [ ] 1.1.2 `domain/model/mod.rs` - MentionType Value Object実装
  - 列挙型: `enum MentionType { User(UserId), Role(RoleId), Everyone, Here }`
  - 不変条件: 各メンション種別は相互排他的

- [ ] 1.1.3 `domain/policy/mention_detection.rs` - メンション検出ロジック
  - 関数: `should_add_read_reaction(msg: &Message) -> bool`
    - メンション（user_mentions OR role_mentions OR mentions_everyone）を含むか判定
  - 関数: `extract_mentions(msg: &Message) -> Vec<MentionType>`
    - すべてのメンション種別を抽出
  - テストケース: ユーザーメンション、ロールメンション、@everyone、複合メンション

### [ ] 1.2 ユースケース層基盤

- [ ] 1.2.1 `usecase/mod.rs` - ユースケース基盤
  - モジュール宣言（on_message, slash_commands, dto）
  - 注記: MVP ではポート（trait）定義は不要（Phase 2以降で導入予定）
  - エラー型定義（UsecaseError）

## Phase 2: コマンド実装

### [ ] 2.0 DTO定義

- [ ] 2.0.1 `usecase/dto/input/mod.rs` - 入力DTO定義
  - `MessageInputDto` (message_id, channel_id, content, user_mentions, role_mentions, mentions_everyone)
  - `CheckReadsInputDto` (user_id, messages: Vec<MessageWithReactionsDto>, hours)
  - `MessageWithReactionsDto` (message_id, content, user_mentions, role_mentions, mentions_everyone, reaction_user_ids)

- [ ] 2.0.2 `usecase/dto/output/mod.rs` - 出力DTO定義
  - `AddReadReactionOutputDto` (message_id, should_add_reaction)
  - `CheckReadsOutputDto` (message_id, message_content, mentioned_users, read_users, unread_users)
  - `UsecaseError` enum (NoMentionedUsers, NoMessages, RoleNotFound, InvalidHoursParameter など)

### [ ] 2.1 /help コマンド実装

- [ ] 2.1.1 `usecase/slash_commands/help.rs` - Usecase ロジック
  - 関数: `execute() -> Result<HelpOutputDto, UsecaseError>`
  - 出力: HelpOutputDto { title, description, commands }
  - 注記: ビジネスロジックは最小（Presentation層で Embed 形成）

- [ ] 2.1.2 `presentation/entry/slash_commands/help.rs` - イベントハンドラー実装
  - Serenity の Interaction 受け取り
  - Usecase呼び出し
  - Embed メッセージ生成・返信

- [ ] 2.1.3 ヘルプメッセージ内容定義
  - コマンド一覧（/help, /check-reads）
  - 使用方法・例

### [ ] 2.2 /check-reads コマンド実装

- [ ] 2.2.1 `usecase/dto/mod.rs` - check-reads 専用 DTO（必要に応じて拡張）
  - 入力: CheckReadsInputDto (すでに2.0.1で定義)
  - 出力: CheckReadsOutputDto[] (すでに2.0.2で定義)

- [ ] 2.2.2 `usecase/slash_commands/check_reads.rs` - Usecase ロジック
  - 関数: `execute(input: CheckReadsInputDto) -> Result<Vec<CheckReadsOutputDto>, UsecaseError>`
  - 処理:
    - メンション対象者を特定（ロール展開：Presentation層で事前実施のため、Usecase側では user_mentions と expanded_role_members を処理）
    - リアクション追加者との差分計算
    - 未読ユーザーリスト生成

- [ ] 2.2.3 `presentation/entry/slash_commands/check_reads.rs` - イベントハンドラー実装
  - Serenity の CommandInteraction 受け取り
  - `hours` パラメータパース
  - Serenity API で過去N時間のメッセージ取得・フィルタリング
  - ロール内メンバー展開（guild.roles, role.members()）
  - CheckReadsInputDto 構築
  - Usecase呼び出し
  - 結果を Embed メッセージに整形
  - 一時的メッセージ（ephemeral）で返信

- [ ] 2.2.4 結果表示フォーマット（Embed設計）
  - メッセージURL
  - メッセージ内容（最初の100文字）
  - 既読ユーザー一覧
  - 未読ユーザー一覧
  - 既読率（%）

## Phase 3: メッセージハンドリング

### [ ] 3.1 メッセージイベントハンドラー実装

- [ ] 3.1.1 `usecase/on_message/auto_add_read_reaction.rs` - 自動リアクション付与ロジック
  - メンション検出
  - 対象メッセージの判定（Bot自身のメッセージを除外）
  - リアクション付与命令の生成
- [ ] 3.1.2 `presentation/entry/on_message.rs` - イベント受信・処理
  - MessageCreateイベント受信
  - Usecaseの呼び出し
  - 例外処理（権限不足など）

### [ ] 3.2 リアクション監視（オプション/MVP では不要）

**注記**: このフェーズは **オプション** です。MVP では実装不要です。

- [ ] 3.2.1 ReactionAddイベントハンドラー（オプション、フェーズ2以降推奨）
  - ユーザーがリアクション追加時の監視・ログ出力
  - 用途: 既読ユーザー通知機能（Phase 2予定）の前提
  - Phase 1-2 では実装スキップ推奨

## Phase 4: Presentation層の完成 & 環境設定

### [ ] 4.1 Presentation層の完成

- [ ] 4.1.1 `presentation/discord_exec.rs` - Serenityクライアント初期化
  - DiscordClient構築
  - EventHandler実装・登録
  - コマンド登録ロジック
- [ ] 4.1.2 `presentation/entry/on_message.rs` - メッセージハンドラー実装
  - ボット自身のメッセージ除外判定
  - MessageInputDto生成
  - Usecase呼び出し
  - リアクション付与実行（`message.react()`）
  - Discord API例外処理
- [ ] 4.1.3 `presentation/entry/slash_commands/check_reads.rs` - コマンドハンドラー実装
  - インタラクション受信
  - コマンド引数パース
  - Serenity APIでメッセージ取得 (`channel.messages()`)
  - CheckReadsInputDto生成
  - Usecase呼び出し
  - 結果をEmbedに整形
  - 一時的メッセージ返信

### [ ] 4.2 インフラストラクチャ層

- [ ] 4.2.1 `infrastructure/config/mod.rs` - 環境変数読み込み
  - DISCORD_TOKEN
  - DISCORD_GUILD_ID
  - RUST_LOG
- [ ] 4.2.2 `infrastructure/config/logging.rs` - ロギング初期化

### [ ] 4.3 Botエントリーポイント設定

- [ ] 4.3.1 `src/main.rs` - 完全実装
  - 環境変数読み込み
  - ロギング初期化
  - Serenityクライアント初期化
  - イベントハンドラー登録
  - ボット起動

## Phase 5: テスト・検証

### [ ] 5.1 ユニットテスト（Domain + Usecase層）

- [ ] 5.1.1 `tests/domain/mention_detection_test.rs` - メンション検出ロジック
  - ユーザーメンション判定
  - ロールメンション判定
  - @everyone/@here判定
- [ ] 5.1.2 `tests/usecase/on_message/auto_add_read_reaction_test.rs` - リアクション付与ロジック
  - メンション検出 → 出力DTO生成
- [ ] 5.1.3 `tests/usecase/slash_commands/check_reads_test.rs` - 既読状況確認ロジック
  - メンション対象者抽出
  - 既読/未読ユーザー分類

### [ ] 5.2 E2Eテスト（手動/Discord テストサーバー）

- [ ] 5.2.1 /help コマンド実行確認
- [ ] 5.2.2 /check-reads コマンド実行確認（オプション引数なし、あり）
- [ ] 5.2.3 自動リアクション付与確認
  - ユーザーメンション時
  - ロールメンション時
  - @everyone時
- [ ] 5.2.4 複数ユーザーでのリアクション反応確認
- [ ] 5.2.5 エラーケース確認（権限不足など）

## Phase 6: ドキュメント・デプロイ準備

### [ ] 6.1 ドキュメント作成

- [ ] 6.1.1 `docs/development/setup.md` - 開発環境セットアップ
  - Rust環境構築
  - Discord Developer Portal設定
  - Bot token取得
- [ ] 6.1.2 `docs/usage/commands.md` - ユーザードキュメント
  - コマンド一覧
  - 使用方法
  - 例
- [ ] 6.1.3 `docs/development/logging.md` - ログ出力仕様
  - ログレベル
  - 出力内容

### [ ] 6.2 デプロイ準備

- [ ] 6.2.1 Dockerfile作成
- [ ] 6.2.2 docker-compose.yml設定
- [ ] 6.2.3 .envrc.example更新

## Phase 7: 最終確認・リリース

### [ ] 7.1 最終テスト

- [ ] 7.1.1 ビルド確認 (`cargo build --release`)
- [ ] 7.1.2 Lintチェック (`cargo clippy`)
- [ ] 7.1.3 フォーマットチェック (`cargo fmt`)

### [ ] 7.2 リリース準備

- [ ] 7.2.1 バージョン更新 (Cargo.toml)
- [ ] 7.2.2 CHANGELOG更新
- [ ] 7.2.3 README最終確認

## 補足

### 依存関係

- Phase 1 → 必須（他のすべてのベース）
- Phase 2, 3: 並列実装可能
- Phase 4: Phase 2, 3 完了後
- Phase 5: Phase 4 完了後
- Phase 6, 7: Phase 5 完了後

### 優先度の考え方

- **高**: `/help` → `/check-reads` → 自動リアクション付与
  - 上記順で機能検証が可能
  - ユーザー体験向上の優先度順

### 実装上の注意事項

- **Presentation層**: すべてのDiscord操作（イベント受信、API操作）はここで
- **Usecase層**: 純粋なビジネスロジック、すべてDTO形式の入出力
- **Domain層**: ドメインモデル、ビジネスロジック（Usecase層から呼ばれる）
- **テスト**: Domain + Usecase層のテストは容易；Presentation層はE2E手動テスト推奨
