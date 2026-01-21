# Sample DiscordBot(Prototype)

## 概要
RustでつくるDiscordbotプロジェクトです。

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

- ルートドキュメントは `README.md` と `.ai/AGENTS.md` です。
    - ざっくりと説明します。
    - `.ai/AGENTS.md`はそれに加え、AIに対しての汎用的な取り決めを記述します。
- プロジェクトドキュメントは `docs/` に格納します。
    - 設計方針や全体構造を説明します。
    - AIに対する詳細な取り決めは`.ai/docs`記述します。
