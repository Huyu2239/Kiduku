#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path

REQUIRED_BY_KIND = {
    "bug": ["現象", "期待", "再現", "受け入れ条件"],
    "feature": ["課題", "提案", "受け入れ条件"],
    "task": ["目的", "スコープ", "完了条件", "検証"],
}

SECRET_PATTERNS = [
    re.compile(r"ghp_[A-Za-z0-9]{20,}"),
    re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"-----BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY-----"),
]


def main() -> int:
    parser = argparse.ArgumentParser(description="Check required issue fields")
    parser.add_argument("--file", required=True)
    parser.add_argument("--kind", choices=["bug", "feature", "task"], required=True)
    args = parser.parse_args()

    text = Path(args.file).read_text(encoding="utf-8")
    missing = []

    for keyword in REQUIRED_BY_KIND[args.kind]:
        if keyword not in text:
            missing.append(keyword)

    if re.search(r"- \[ \] ", text) is None:
        missing.append("acceptance_checkbox")

    secret_hits = [p.pattern for p in SECRET_PATTERNS if p.search(text)]

    if missing or secret_hits:
        print("CHECK FAILED")
        if missing:
            print("Missing keywords:")
            for item in missing:
                print(f"- {item}")
        if secret_hits:
            print("Potential secrets:")
            for item in secret_hits:
                print(f"- {item}")
        return 1

    print("CHECK OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
