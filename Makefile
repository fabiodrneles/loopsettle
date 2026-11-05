# Rebuild everything from scratch: tests, all experiments, the summary and the web app.
#   make all        tests + experiments + results/SUMMARY.md and SUMMARY.en.md + web build
#   make rebuild    delete results/ first, then `make all`
#   make data       re-download the public Pix aggregates (network access)
CARGO ?= cargo
PYTHON ?= python
EXPERIMENTS = calibration baseline potential-vs-realizable microcycle-sweep stress finland-replication

.PHONY: all rebuild test experiments summary web wasm data clean-results

all: test experiments summary web

rebuild: clean-results all

test:
	$(CARGO) test --workspace

experiments:
	$(CARGO) build --release -p loopsettle-cli
	for e in $(EXPERIMENTS); do $(CARGO) run --release -q -p loopsettle-cli -- experiment $$e || exit 1; done

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
	rm -rf results/calibration results/baseline results/potential-vs-realizable results/microcycle-sweep results/stress results/finland-replication results/SUMMARY.md results/SUMMARY.en.md
