.PHONY: check help

# デフォルトターゲット
.DEFAULT_GOAL := help

# ヘルプ
help:
	@echo "使用方法: make [command]"
	@echo ""
	@echo "commands:"
	@echo "  help         このヘルプを表示"
	@echo "  check        フォーマット・Lint・テストを実行"

# 開発用チェック（fmt + lint + test）
check:
	@cargo fmt --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo test -- --test-threads=1
	@echo ""
	@echo "✓ すべてのチェックが完了しました"
