# Git 運用ルール

## 目的

変更履歴を追いやすくする。

## ルール

- 1コミット1目的を守る
- レビュー可能な粒度でコミットする
- 生成物や一時ファイルはコミットしない

## ブランチ運用（git-flow）

- `main` は常にデプロイ可能な状態を保つ
- `develop` を日常開発の基点とする
- `feature/*` は機能追加、`bugfix/*` は不具合修正に使う
- `release/*` でリリース準備を行い、完了後に `main` と `develop` へ反映する
- `hotfix/*` は緊急修正用とし、`main` と `develop` へ反映する

### 生成・起点

- `feature/*` と `bugfix/*` は `develop` から作成する
- `release/*` は `develop` から作成する
- `hotfix/*` は `main` から作成する

### 命名

- ブランチ名は小文字の `kebab-case` を使う
- 例:
  - `feature/add-voice-command`
  - `bugfix/fix-login-timeout`
  - `release/0.2.0`
  - `hotfix/0.2.1`

### マージ方針

- `main` と `develop` への直接コミットは禁止する
- すべての変更は PR で取り込む
- `feature/*` と `bugfix/*` は `develop` へマージする
- `release/*` と `hotfix/*` は `main` と `develop` の両方へマージする
- マージ後はブランチを削除する

### リリース

- `release/*` または `hotfix/*` を `main` に取り込む際にタグを付ける
- タグは `vX.Y.Z` 形式とする

## コミット文

- 形式は `prefix: 日本語の要約` とする
- 例: `feat: 音声再生コマンドを追加`
- 主な prefix
  - `feat`: 機能追加
  - `fix`: 不具合修正
  - `del`: 削除
  - `refactor`: リファクタリング
  - `docs`: ドキュメント更新
  - `test`: テスト追加・更新
