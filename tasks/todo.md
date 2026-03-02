# Issue作成フロー整備タスク

## 計画
- [x] 1. 現行ルールと既存ドキュメントを確認する（`docs/`・`.ai/docs/`・`AGENTS.md`）
- [x] 2. GitHub Issue テンプレート仕様（Issue forms / `config.yml`）を確認する
- [x] 3. 人間向け Issue フォームを `.github/ISSUE_TEMPLATE/` に追加する
- [x] 4. AIエージェントが対話ベースで起票するための運用フローを `docs/` と `.ai/docs/` に作成する
- [x] 5. 索引（`docs/rules/index.md`, `.ai/docs/index.md`）を更新する
- [x] 6. 構文確認（YAML）と差分レビューを実施する

## 実装前確認
- [x] 計画内容を確認し、この順で実装する

## 作業レビュー（完了後に記入）
- [x] 変更の妥当性確認（テンプレート表示・入力項目・運用手順）
- [x] 検証ログ（実行コマンドと結果）を記録
- [x] 残課題・次アクションを記録

### 検証ログ

- `ruby -ryaml -e "YAML.load_file(...)"` で `.github/ISSUE_TEMPLATE/*.yml` と `config.yml` の構文を確認（すべて `OK`）
- `git status --short` で追加・更新ファイルを確認

### 残課題・次アクション

- GitHub 側でテンプレート選択画面の表示確認（Web UI）
- 必要に応じてラベル運用（`bug` / `feature` / `task` 等）をリポジトリ設定で追加
