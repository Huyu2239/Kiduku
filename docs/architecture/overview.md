# アーキテクチャ概要

## 目的

Discord Bot の機能追加や運用に耐える構造を保ちつつ、過剰な抽象化を避ける。

## 背景

- ドメイン知識と外部依存（Discord API / 音声 / I/O）を分離したい
- 変更の影響範囲を小さくし、テストしやすい形にしたい

## 採用方針（クリーンアーキテクチャ）

内側ほど安定し、外側ほど変化しやすい層とする。依存は外側から内側への一方向とする。

## 層の責務（厳密）

### Domain

コアとなる業務知識を表現する層。

- ドメインモデル（構造体・列挙）と不変条件
- バリデーション（値/状態の妥当性）
- ポリシー（業務ルール）とドメインサービス（複数モデル横断の操作）
- 他レイヤへの依存は禁止

### Usecase

アプリケーションの振る舞いを表現する層。

- 入力（DTO）→ ドメイン操作 → 出力（DTO）の流れを定義する
- **すべての入出力は DTO 形式で統一される**
- ビジネスロジックのみを扱う（外部API操作は行わない）
- 起点は「コマンド」「イベント」「定期処理」を含む
- 外部依存は「ポート（trait）」として定義するが、MVP ではポートを極力避ける設計

### Interface Adapters

境界の変換を行う層。

- DTO とドメインモデルの変換
- Usecase の入力整形
- リポジトリやゲートウェイの実装は置かない

### Infrastructure

環境設定・初期化・ロギング。

- 環境変数読み込み（DISCORD_TOKEN, LOG_LEVEL など）
- ロギングの初期化（tracing-subscriber 設定）
- **注意: MVP では Serenity API アクセスは Presentation 層で行う**

### Presentation

Discord イベント・コマンド受信と操作の入口。

- Serenity クライアントの初期化とイベントハンドラー登録
- Discord のイベント受信（MessageCreate, InteractionCreate など）
- イベントデータを DTO に変換
- Usecase 呼び出し
- **出力DTO に基づいて Discord API 操作を実行**（リアクション付与、メッセージ返信など）
- Discord API エラー処理（権限不足、レート制限など）

## 依存関係ルール

- 依存方向は **Presentation/Infrastructure → Interface Adapters → Usecase → Domain**
- Domain は他レイヤに依存しない
- Usecase は Domain とポート定義のみに依存する
- Infrastructure は Usecase が定義したポートを実装する

## データの流れ（MVP での基本形）

1. Presentation が Serenity から Discord イベントを受け取る
2. Presentation が イベントデータを DTO に変換
3. Usecase が Domain ロジックを実行
4. Usecase が 結果を DTO で返す
5. Presentation が 出力DTO に基づいて Discord API を操作実行

**ポイント**: Usecase は Discord 操作を行わない。すべて DTO の入出力で完結する。

## エラー方針

- Domain と Usecase では文脈が分かるエラーを返す
- Infrastructure 由来のエラーは境界で変換する
- Presentation はユーザー向けの短いメッセージに変換する

## 想定モジュール構成（MVP）

必要になるまで分割を増やさない。初期は単純に保つ。

- `src/domain/` : ドメインロジック
- `src/usecase/` : ビジネスロジック
- `src/interface/` : 層間のマッピング
- `src/infrastructure/` : 環境設定・初期化
- `src/presentation/` : Discord 操作・イベント受付

## 具体的なディレクトリ構成（MVP）

初期から固定する構成。ファイル数が増えた場合のみ拡張する。

- `src/domain/`
  - `model/` : エンティティ（Message, ReadState など）
  - `policy/` : 業務ルール（メンション検出、既読判定など）
- `src/usecase/`
  - `dto/`
    - `input/` : Usecase 入力DTO
    - `output/` : Usecase 出力DTO
  - `slash_commands/` : スラッシュコマンド（/check-reads, /help）
  - `on_message/` : メッセージ受信起点のユースケース
- `src/interface/`
  - `mapper/` : DTO ↔ Domain 変換
- `src/infrastructure/`
  - `config/` : 環境変数読み込み、設定構造体
  - `logging.rs` : ロギング初期化（非MVP では削除可能）
- `src/presentation/`
  - `discord_exec.rs` : Serenity クライアント初期化・ハンドラー登録
  - `entry/` : イベント・コマンド受付と Discord 操作
    - `on_message.rs` : メッセージ受信イベント処理
    - `on_error.rs` : エラーハンドリング
    - `slash_commands/` : スラッシュコマンドハンドラー
  - `mod.rs` : プレゼンテーション層の公開API

## 命名と配置の規約

- Usecase は「動詞 + 対象」の命名とする（例: `play_voice`, `join_channel`）
- イベント起点のモジュールは `on_*` を基本とする
- Usecase の入口関数は `execute` を基本とする
- インタラクション起点は `on_interaction_*` を基本とする
- DTO は `Input` / `Payload` / `Plan` など役割が分かる命名にする
- ポート（trait）は `Port` 接尾辞を使う（例: `VoiceGatewayPort`）
- バリデーションは `validate_*`、ポリシーは `can_*` / `is_*` を基本とする
- Presenter は `*_presenter`、Mapper は `*_mapper` を基本とする
- Presentation の入口処理は `handle_*` を基本とする

## ポート（ports）の位置づけ（Phase 2 以降）

Ports は **Usecase が外部依存に要求するインターフェース（trait）** を置く場所である。

- MVP では不要（すべてDTO で表現）
- 将来、永続化やキャッシュが必要になったら導入
- 例（将来）: `MessageRepositoryPort`, `CachePort`
- 目的: Usecase から永続化やキャッシュなどの具体実装を隔離する

### 出力の扱い（MVP の方針）

**Usecase は Discord 操作を一切行わず、結果を DTO で返す。**

- Usecase: 処理結果を DTO 形式で返す（例: `AddReadReactionOutputDto`, `CheckReadsOutputDto`）
- DTO には実行するべき操作や結果データが含まれる
- Presentation: 出力DTO を解析して、必要な Discord API 操作を実行

このため、**Usecase にはポートが不要** （現在の MVP 設計では）。
ポートが必要になるのは、永続化やキャッシュが必要になった Phase 2 以降。

## MVP での実装上の具体的な制約

### 外部依存を避ける

**Usecase層 内では、以下を呼び出さない：**
- `async fn` 呼び出し（Serenity API など）
- I/O 操作（ファイル読み書き、DB アクセス）
- 時間取得（現在時刻操作は Presentation で行い、DTO に含める）

**Presentation層 では、以下を実行：**
- Serenity イベント受信・処理
- Serenity API 操作（message.react(), channel.messages() など）
- エラーハンドリング（Discord API エラー捕捉・ログ出力）

### 設計の単純性を保つ

- **ポート（trait）は不要**: MVP では入出力が明確なため、trait で抽象化する必要がない
- **ドメインサービスは最小限**: ビジネスロジックは policy に集中させる
- **DTO は機械的変換**: Mapper は単純な構造変換のみ

## テスト方針（層別・MVP）

- **Domain**: 純粋関数の単体テスト（メンション検出、既読判定など）
- **Usecase**: 純粋ロジックの単体テスト（DTO 入出力のみ）
- **Presentation**: E2E テスト推奨（Discord テストサーバーでの手動確認）
  - Serenity API 操作は実環境でのテストが必須
- **Interface**: Mapper のテスト（不可視の変換のため）

## 設計方針

- 単純で直線的な構造を維持する
- 境界をまたぐデータは変換を明示する
- 最初から抽象化を増やさず、必要になった時点で導入する

## 変更方針

- 小さな単位で段階的に分離する
- 実装が薄い層は統合して運用コストを下げる
