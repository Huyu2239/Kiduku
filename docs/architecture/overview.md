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

- 入力 → ドメイン操作 → 出力の流れを定義する
- 外部依存は「ポート（trait）」として定義する
- Discord や永続化の具体実装は置かない
- 起点は「コマンド」「イベント」「定期処理」を含む

### Interface Adapters

境界の変換を行う層。

- DTO とドメインモデルの変換
- Usecase の入力整形
- リポジトリやゲートウェイの実装は置かない

### Infrastructure

外部サービス・フレームワークとの接続。

- Discord API（serenity / songbird）
- 永続化、設定、外部I/O
- Usecase が定義したポートの具体実装

### Presentation

ユーザー入力の入口。

- Discord のイベント受信とコマンド解釈
- 入力の最小限の整形と Usecase 呼び出し
- 出力は Presentation で DiscordExecPlan を実行する

## 依存関係ルール

- 依存方向は **Presentation/Infrastructure → Interface Adapters → Usecase → Domain**
- Domain は他レイヤに依存しない
- Usecase は Domain とポート定義のみに依存する
- Infrastructure は Usecase が定義したポートを実装する

## データの流れ（基本形）

1. Presentation が Discord イベントを受け取る
2. Interface Adapters が DTO を作成
3. Usecase が Domain を操作
4. Usecase が DiscordExecPlan を返す
5. Presentation が DiscordExecPlan を実行する

## エラー方針

- Domain と Usecase では文脈が分かるエラーを返す
- Infrastructure 由来のエラーは境界で変換する
- Presentation はユーザー向けの短いメッセージに変換する

## 想定モジュール構成

必要になるまで分割を増やさない。初期は単純に保つ。

- `src/domain/`
- `src/usecase/`
- `src/interface/`
- `src/infrastructure/`
- `src/presentation/`

## 具体的なディレクトリ構成（案）

初期から固定する構成。ファイル数が増えた場合のみ拡張する。

- `src/domain/`
  - `policy/` : 業務ルール
- `src/usecase/`
  - `dto/` : Usecase 入出力のデータ構造
    - `input/` : Usecase 入力
    - `output/` : Usecase 出力
  - `slash_commands/` : スラッシュコマンド
  - `on_message/` : メッセージ受信起点のユースケース
  - `ports/` : 外部依存のポート定義
- `src/interface/`
  - `mapper/` : DTO/ドメイン変換
- `src/infrastructure/`
  - `config/` : 設定読み込み
- `src/presentation/`
  - `entry/` : 受信イベント・コマンド受付
    - `on_message.rs` : メッセージ受信イベント
    - `on_error.rs` : 入口のエラーハンドリング
    - `slash_commands/` : スラッシュコマンド
  - `mod.rs` : Framework 構築とイベント配線
  - `discord_exec.rs` : DiscordExecPlan の実行

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

## ポート（ports）の位置づけ

Ports は **Usecase が外部依存に要求するインターフェース（trait）** を置く場所である。
Usecase はポートを **呼び出す側** であり、具体実装は Infrastructure に置く。

- 例: `GuildConfigRepositoryPort`, `VoiceGatewayPort`, `ClockPort`
- 目的: Usecase から Discord API や永続化などの具体実装を隔離する

### 出力の扱い（本プロジェクトの方針）

本プロジェクトでは **Usecase は送信処理を直接行わず**、出力データを返す。

- Usecase: `DiscordExecPlan` を返す（送信手順・内容・次アクションなど）
- 初回応答は専用の Usecase（例: `defer_response`, `open_modal`）で返す
- Presentation: 初回応答を先に実行し、その後に本処理 Usecase を実行する

このため、**「メッセージ送信」はポートに含めない**。  
ポートは主に「情報取得」や「外部リソース操作」のために使う。

## テスト方針（層別）

- Domain: 純粋関数の単体テスト
- Usecase: ポートをモック化した単体テスト
- Infrastructure / Presentation: 可能な範囲で結合テスト

## 設計方針

- 単純で直線的な構造を維持する
- 境界をまたぐデータは変換を明示する
- 最初から抽象化を増やさず、必要になった時点で導入する

## 変更方針

- 小さな単位で段階的に分離する
- 実装が薄い層は統合して運用コストを下げる
