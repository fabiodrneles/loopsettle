#!/usr/bin/env python3
"""Golden results: a small, fast run of every experiment, versioned in results-golden/.

    python -I scripts/golden.py            regenerate results-golden/
    python -I scripts/golden.py --check    run into a temporary directory and compare

CI runs `--check`: a change to an engine, the generator or an experiment that moves any number
fails until results-golden/ is regenerated and committed, so no result changes unnoticed. The
same check on Linux, Windows and macOS is the test of bit-for-bit reproducibility across
platforms (specs/09, E3 and E4).

The golden scenarios are the real experiment files (experiments/*.toml) with the overrides
below: fewer participants, payments and seeds. They test the code, not the study's numbers.
Standard library only.
"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXPERIMENTS = ["calibration", "baseline", "potential-vs-realizable", "microcycle-sweep", "stress", "finland-replication",
               "robustness", "bias-calibration", "heterogeneity", "funding-windows", "regimes", "liquidity-frontier", "echo-sweep", "echo-scale", "circularity-horizon"]
# (section, key) -> value, applied to every experiment file that has the section.
OVERRIDES = {
    ("scenario", "participants"): "40",
    ("scenario", "payments_per_day"): "30_000",
    ("robustness", "seeds"): "[1, 2, 3]",
    ("robustness", "payments_per_day"): "[30_000, 90_000]",
    ("heterogeneity", "payments_per_day"): "[30_000, 90_000]",
    ("windows", "payments_per_thursday"): "[30_000]",
    ("regimes", "wholesale_payments_per_day"): "[500, 5_000]",
    ("calibration", "seeds"): "[1, 2]",
    ("finland", "seeds"): "[1, 2]",
    ("frontier", "seeds"): "[1, 2, 3]",
    ("echo", "seeds"): "[1, 2, 3]",
    ("echo", "share"): "[0.2]",
    ("echo_scale", "seeds"): "[1, 2]",
    ("horizon", "seeds"): "[1, 2]",
    ("echo_scale", "payments_per_day"): "[30_000, 90_000]",
    ("frontier", "wholesale_payments_per_day"): "2_000",
    ("frontier", "alphas"): "[0.5, 0.9, 1.0, 1.5]",
}
CONFIG_DIR = os.path.join("target", "golden-config")  # relative: it is recorded in summary.json


def small_config(name):
    out, section, seen = [], None, set()
    with open(os.path.join(ROOT, "experiments", name + ".toml"), encoding="utf-8") as f:
        lines = f.read().split("\n")

    def flush():
        for (s, k), v in OVERRIDES.items():
            if s == section and (s, k) not in seen and s != "scenario":
                out.append(f"{k} = {v}")

    for line in lines:
        m = re.match(r"\s*\[([a-z_]+)\]\s*$", line)
        if m:
            flush()
            section = m.group(1)
        else:
            k = re.match(r"\s*([a-z_]+)\s*=", line)
            if k and (section, k.group(1)) in OVERRIDES:
                seen.add((section, k.group(1)))
                line = f"{k.group(1)} = {OVERRIDES[(section, k.group(1))]}"
        out.append(line)
    flush()
    os.makedirs(os.path.join(ROOT, CONFIG_DIR), exist_ok=True)
    path = os.path.join(CONFIG_DIR, name + ".toml")
    with open(os.path.join(ROOT, path), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out))
    return path.replace(os.sep, "/")


def generate(dest):
    env = dict(os.environ, LOOPSETTLE_GOLDEN="1", LOOPSETTLE_RESULTS_DIR=dest)
    cargo = os.environ.get("CARGO", "cargo")
    subprocess.run([cargo, "build", "--release", "-q", "-p", "loopsettle-cli"], cwd=ROOT, check=True)
    for name in EXPERIMENTS:
        cmd = [cargo, "run", "--release", "-q", "-p", "loopsettle-cli", "--", "experiment", name, "--config", small_config(name)]
        r = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"experiment {name} failed:\n{r.stderr}")
        print(f"  {name}: ok")


def files(root):
    return sorted(os.path.relpath(os.path.join(d, f), root).replace(os.sep, "/") for d, _, fs in os.walk(root) for f in fs)


def main():
    golden = os.path.join(ROOT, "results-golden")
    if "--check" not in sys.argv:
        shutil.rmtree(golden, ignore_errors=True)
        generate(golden)
        print(f"wrote {golden} ({len(files(golden))} files)")
        return
    with tempfile.TemporaryDirectory() as tmp:
        generate(tmp)
        want, got = files(golden), files(tmp)
        bad = sorted(set(want) ^ set(got))
        bad += [f for f in want if f in got and not filecmp.cmp(os.path.join(golden, f), os.path.join(tmp, f), shallow=False)]
        if bad:
            print("golden results differ:")
            for f in bad:
                print("  " + f)
            keep = os.path.join(ROOT, "target", "golden-got")
            shutil.rmtree(keep, ignore_errors=True)
            shutil.copytree(tmp, keep)
            sys.exit(f"compare results-golden/ with {keep}; if the change is intended, run `python -I scripts/golden.py` and commit")
        print(f"golden results identical ({len(want)} files)")


if __name__ == "__main__":
    main()
