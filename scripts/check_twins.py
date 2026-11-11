#!/usr/bin/env python3
"""Bilingual documentation check (DECISIONS.md, D17): every `X.md` has an `X.pt-BR.md` twin.

    python -I scripts/check_twins.py                 both files of every pair exist
    python -I scripts/check_twins.py --base <ref>    also: a pair changed since <ref> changed together

Standard library only.
"""
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIRS = ["", "docs", "paper", "results", "data"]
# Single-file documents: both languages inside one file, generated logs, or working documents.
SINGLE = {"CLAUDE.md", "CHANGELOG.md", "README.md", "data/README.md", "results/README.md", "paper/paper.pt-BR.md"}


def pairs():
    out, missing = [], []
    for d in DIRS:
        for f in sorted(os.listdir(os.path.join(ROOT, d))):
            rel = f"{d}/{f}" if d else f
            if not f.endswith(".md") or rel in SINGLE:
                continue
            twin = rel[:-9] + ".md" if f.endswith(".pt-BR.md") else rel[:-3] + ".pt-BR.md"
            if not os.path.exists(os.path.join(ROOT, twin)):
                missing.append((rel, twin))
            elif not f.endswith(".pt-BR.md"):
                out.append((rel, twin))
    return out, missing


def main():
    ps, missing = pairs()
    errors = [f"{a}: twin {b} is missing" for a, b in missing]
    if "--base" in sys.argv:
        base = sys.argv[sys.argv.index("--base") + 1]
        changed = set(subprocess.run(["git", "diff", "--name-only", f"{base}...HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.split())
        for a, b in ps:
            if (a in changed) != (b in changed):
                errors.append(f"{a} and {b}: only one of the two changed since {base}")
    if errors:
        sys.exit("bilingual documentation check failed:\n  " + "\n  ".join(errors))
    print(f"{len(ps)} document pairs ok")


if __name__ == "__main__":
    main()
