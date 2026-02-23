.PHONY: check help db.up db.down db.migrate db.reset

# デフォルトターゲット
.DEFAULT_GOAL := help

# ヘルプ
help:
	@echo "使用方法: make [command]"
	@echo ""
	@echo "commands:"
	@echo "  help         このヘルプを表示"
	@echo "  check        フォーマット・Lint・テストを実行"
	@echo "  db.up        PostgreSQL コンテナを起動"
	@echo "  db.down      PostgreSQL コンテナを停止"
	@echo "  db.migrate   スキーマを既存 DB に適用（冪等）"
	@echo "  db.reset     DB を完全リセット（データ削除）してスキーマを再適用"

# 開発用チェック（fmt + lint + test）
check:
	@cargo fmt --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo test -- --test-threads=1
	@echo ""
	@echo "✓ すべてのチェックが完了しました"

# PostgreSQL コンテナを起動
db.up:
	@docker compose up -d postgres

# PostgreSQL コンテナを停止
db.down:
	@docker compose down

# スキーマを既存 DB に適用（CREATE TABLE IF NOT EXISTS なので冪等）
db.migrate:
	@docker exec -i kiduku-postgres psql -U kiduku -d kiduku < docker/init/01_schema.sql
	@echo "✓ マイグレーション完了"

# DB を完全リセット（ボリューム削除 → 再起動 → スキーマ自動適用）
db.reset:
	@docker compose down -v
	@docker compose up -d postgres
	@echo "✓ DB をリセットしました（スキーマは自動適用されます）"
