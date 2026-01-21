# MCP ツールの概要と利用方針

## 目的

AI が利用する MCP（Model Context Protocol）ツールの内容と、使用時の判断基準を整理する。

## 導入済み MCP

- `serena`
  - リポジトリ内のファイル探索・読み取り・コード編集・シンボル解析を行う
  - 例: `list_dir`, `find_file`, `read_memory`, `apply_patch`
- `context7`
  - 外部ライブラリの公式ドキュメント検索（API 使用方法の確認）
  - 例: `resolve-library-id`, `query-docs`
- `sequential-thinking`
  - 複雑な思考プロセスの構造化（長い推論を伴うタスク）

## 利用方針

- 目的に最短で到達できるツールを選び、MCP を積極的に活用する
- 変更系の操作は最小限にし、差分が追える形で実行する
- 機密情報や認証情報に触れる操作は避ける
- リポジトリ内の確認・編集は `serena` を優先する
- 外部ライブラリの仕様確認は `context7` を使う
- 複雑な判断が必要な場合は `sequential-thinking` を使う
- 迷った場合は、まず読み取り系ツールで安全に確認する

## 注意

- `.envrc` はクレデンシャルが含まれるため参照しない
