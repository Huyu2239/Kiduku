# Kiduku - Discord既読確認Bot

## 概要
Discordサーバー内のメンション確認状況を可視化し、チーム内での既読状況を把握するボットです。

- このREADME.mdは人間向けにかかれています。
- AI向けのドキュメントは、`.ai/AGENTS.md` に記載します。

## ディレクトリ構成

```bash
$ tree -a -L 1
./
├── .ai/             # AI用ディレクトリ
├── .envrc          # direnvファイル
├── .envrc.example
├── .git/
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── Makefile
├── README.md       # ルートドキュメント
├── docs/           # プロジェクトドキュメント
├── mise.toml
├── src/             # ソースコード
└── target/          # rustによる生成物

6 directories, 8 files
```

## ドキュメント構成

- **ルートドキュメント**
    - `README.md` (このファイル) : プロジェクト概要
    - `.ai/AGENTS.md` : AI向けの規約・制約

- **プロジェクトドキュメント** (`docs/` 配下)
    - [`docs/index.md`](./docs/index.md) : ドキュメント索引
    - [`docs/onboarding/`](./docs/onboarding/index.md) : 開発環境セットアップ
    - [`docs/architecture/`](./docs/architecture/index.md) : アーキテクチャ設計
    - [`docs/features/`](./docs/features/index.md) : 機能仕様・PRD・DD
    - [`docs/rules/`](./docs/rules/index.md) : 開発規約
    - [`docs/todo.md`](./docs/todo.md) : 実装ロードマップ

- **AI向けドキュメント** (`.ai/docs/` 配下)
    - AI専用の補助ドキュメント・内部メモ
