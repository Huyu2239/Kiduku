# テスト方針

## 目的

変更による振る舞いの崩れを早期に検知する。

## 基本ルール

- 変更点に対応するテストを追加・更新する
- 可能な限り小さく、決定的なテストにする
- 外部サービスへの依存は避ける

## 層別テスト方針（MVP）

### Domain層 → 単体テスト推奨

- 純粋関数のテスト（メンション検出、既読判定など）
- 外部依存なし、決定的なテスト
- 例: `tests/domain/policy/mention_detection_test.rs`

### Usecase層 → 単体テスト推奨

- ビジネスロジック（DTO 入出力）のテスト
- 外部API呼び出しなし
- 例: `tests/usecase/on_message/auto_add_read_reaction_test.rs`

### Interface層 → ユニットテスト推奨

- DTO ↔ Domain 変換のテスト
- 例: `tests/interface/mapper_test.rs`

### Presentation層 → E2E テスト（手動）

- Discord テストサーバーでの手動確認推奨
- 自動テストは困難（Serenity API の実行環境が必要）
- テスト項目: コマンド実行、リアクション付与、エラーハンドリング

### Infrastructure層 → テスト不要（MVP）

- 環境設定・ロギング初期化のみなので、テスト不要

## 実行

- ユニットテスト: `cargo test` でローカル実行
- E2E テスト（Presentation）: Discord テストサーバーで手動確認
- CI/CD: 前述のチェックコマンド(`make check`)を実行

## カバレッジ

- MVP ではカバレッジ目標は設定しない
- Domain + Usecase で 80% 以上のカバレッジを目指す（将来）
