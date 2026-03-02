#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path

COMMON_SECTIONS = [
    "## 1. 依頼の背景",
    "## 2. 人間との対話要約",
    "## 3. 事実と証拠",
    "## 4. 期待する結果",
    "## 6. 安全性チェック",
]

KIND_HINTS = {
    "bug": ["再現手順", "実測結果", "本来の挙動"],
    "feature": ["要望の要点", "受け入れ条件", "合意済みの範囲"],
    "task": ["受け入れ条件", "検証", "合意済みの範囲"],
}

SECRET_PATTERNS = [
    re.compile(r"ghp_[A-Za-z0-9]{20,}"),
    re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"-----BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY-----"),
    re.compile(r"(?i)discord[_-]?bot[_-]?token\s*[:=]\s*\S+"),
]


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate issue body markdown")
    parser.add_argument("--file", required=True)
    parser.add_argument("--kind", choices=["bug", "feature", "task"], required=True)
    args = parser.parse_args()

    text = Path(args.file).read_text(encoding="utf-8")
    errors = []

    for section in COMMON_SECTIONS:
        if section not in text:
            errors.append(f"missing section: {section}")

    for hint in KIND_HINTS[args.kind]:
        if hint not in text:
            errors.append(f"missing {args.kind} hint: {hint}")

    if re.search(r"- \[ \] ", text) is None:
        errors.append("missing acceptance checkbox: '- [ ] ...'")

    for pattern in SECRET_PATTERNS:
        if pattern.search(text):
            errors.append(f"possible secret detected by pattern: {pattern.pattern}")

    if errors:
        print("VALIDATION FAILED")
        for err in errors:
            print(f"- {err}")
        return 1

    print("VALIDATION OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
