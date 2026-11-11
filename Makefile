# Rebuild everything from scratch: tests, all experiments, the summary and the web app.
#   make all        tests + experiments + results/SUMMARY.md and SUMMARY.pt-BR.md + web build
#   make rebuild    delete results/ first, then `make all`
#   make data       re-download the public Pix aggregates (network access)
#   make verify     what CI checks: tests, lints, golden results, bilingual documents
#   make golden     regenerate results-golden/ after an intended change of results
CARGO ?= cargo
PYTHON ?= python
EXPERIMENTS = calibration baseline potential-vs-realizable microcycle-sweep stress finland-replication robustness bias-calibration heterogeneity funding-windows regimes

.PHONY: all rebuild test experiments summary web wasm data clean-results golden verify lint

all: test experiments summary web

rebuild: clean-results all

test:
	$(CARGO) test --workspace

experiments:
	$(CARGO) build --release -p loopsettle-cli
	for e in $(EXPERIMENTS); do $(CARGO) run --release -q -p loopsettle-cli -- experiment $$e || exit 1; done
	$(CARGO) run --release -q -p loopsettle-cli -- topology data/samples/triangle.csv

summary:
	$(PYTHON) -I scripts/summarize.py

wasm:
	cd web && npm run wasm

web:
	cd web && npm install --no-audit --no-fund && npm run wasm && npm run build
	node scripts/wasm-smoke.mjs

data:
	$(PYTHON) -I data/fetch_pix.py --month 202609 --keys-date 2026-09-30

clean-results:
	rm -rf results/calibration results/baseline results/potential-vs-realizable results/microcycle-sweep results/stress results/finland-replication results/robustness results/bias-calibration results/heterogeneity results/funding-windows results/regimes results/topology-triangle results/SUMMARY.md results/SUMMARY.pt-BR.md

# Small, fast run of every experiment, compared byte for byte in CI (specs/09).
golden:
	$(PYTHON) -I scripts/golden.py

lint:
	$(CARGO) clippy --workspace --all-targets -- -D warnings -A clippy::needless_range_loop -A clippy::neg_cmp_op_on_partial_ord

verify: test lint
	$(PYTHON) -I scripts/golden.py --check
	$(PYTHON) -I scripts/check_twins.py
