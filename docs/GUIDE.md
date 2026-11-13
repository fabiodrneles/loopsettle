# LoopSettle user guide

**English** · [Português](GUIDE.pt-BR.md)

This guide explains, step by step, what LoopSettle does, how it works, how to use it and how to
read what it produces. It assumes no prior knowledge of payment systems or of Rust. The formal
definitions are in [`METHODOLOGY.md`](METHODOLOGY.md).

## Contents

1. [What the simulator does](#1-what-the-simulator-does)
2. [The ideas you need](#2-the-ideas-you-need)
3. [Three ways to use it](#3-three-ways-to-use-it)
4. [Install](#4-install)
5. [Use the browser interface](#5-use-the-browser-interface)
6. [Run the ready-made experiments](#6-run-the-ready-made-experiments)
7. [Analyse your own payments](#7-analyse-your-own-payments)
8. [Choose and write settlement designs](#8-choose-and-write-settlement-designs)
9. [Change a scenario](#9-change-a-scenario)
10. [Common problems](#10-common-problems)
11. [Going further](#11-going-further)

## 1. What the simulator does

When someone sends a Pix to a person at **another institution**, the payer's institution
transfers money to the payee's institution at once, through an account each one keeps at the
central bank (the PI account). For that, each institution must keep money idle in that account:
**liquidity**. Idle money is expensive. (A Pix between two accounts at the same institution is
settled in its own books and does not use that account.)

During the day, part of the money goes round in circles: bank A pays B, B pays C, C pays A. In
principle these loops could be offset, and banks would need less liquidity. LoopSettle answers
three questions:

1. **How much** of the money flowing between institutions goes round in loops?
2. **How much of it can be used in practice**, under real rules (each payment settled in full,
   no negative balance, within a few seconds)?
3. **What does using it cost in delay**?

To answer, it:

- **generates synthetic payments** that look like real Pix, calibrated on public central bank
  data (count, mean value, value distribution, concentration across institutions);
- **settles them in several ways** (the "designs" or "engines": real-time gross like Pix,
  netting at fixed times, micro-cycles and others) and measures the liquidity each needs and the
  delay it imposes;
- **measures the loops** with three different mathematical lenses;
- **computes a bound** that holds for any mechanism: how much, at most, a system that settles
  each payment within `d` seconds can save.

It also runs **on your own data**: give it a payments file and it returns the same numbers,
without the data leaving the computer.

## 2. The ideas you need

| Term | Meaning |
|---|---|
| **Participant** | an institution with a settlement account (bank, fintech, payment institution). |
| **Real-time gross settlement** | each payment is settled on its own, on arrival, if the balance allows. This is Pix. In the code: `gross`. |
| **Netting** | adding up what each owes and is owed and transferring only the difference. It requires waiting. |
| **Cycle** | a loop of payments (A→B→C→A). A cycle can be settled at once, transferring only the difference. |
| **UB (upper bound)** | the liquidity a participant needs so that no payment is refused under real-time gross settlement: the largest "hole" its outflows open against its inflows during the day. |
| **LB (lower bound)** | the theoretical minimum liquidity: the net debit at the end of the day. It is what would suffice if everything were netted once, at the end of the day. |
| **Liquidity saving** | how much a design reduces liquidity against UB, in %: `1 − design liquidity ÷ UB`. |
| **Liquidity floor** | the least liquidity that **any** mechanism settling each payment within `d` seconds can need. If the floor for 10 s is almost equal to UB, no 10 s mechanism saves almost anything. |
| **Delay p50, p95** | half of the payments (p50) or 95% of them (p95) settled within that time. |
| **2.8 s target** | Pix's timing rules require 50% of payments to settle within 2.8 s. |
| **ρ (rho)** | the Bank of Finland delay indicator: 0 = everything on arrival, 1 = nothing before the end of the day. |
| **π (pi)** | the liquidity usage indicator: liquidity used ÷ total value paid. |
| **Seed** | the number that fixes the random draw of payments. The same seed always gives the same result, on any computer. |
| **Calibration** | tuning the generator to reproduce public figures. Each parameter is listed in [`PARAMETERS.md`](PARAMETERS.md) as `SOURCED`, `CALIBRATED` or `ASSUMPTION`. |

**A worked example.** Three banks, zero balances. A pays B 10; then B pays C 5; then C pays A 5.

- Under real-time gross settlement, A needs 10 in its account to make the first payment:
  **UB of A = 10**. B receives 10 before paying 5, so it needs nothing; C receives 5 before
  paying 5, likewise.
- At the end of the day A paid 10 and received 5: it owes 5. **LB of A = 5**.
- The loop A→B→C→A of value 5 could be offset: only A's other 5 to B remain. That is the gap
  between UB (10) and LB (5).

## 3. Three ways to use it

| You want to... | Use | You need |
|---|---|---|
| see the simulator working and play with the parameters | the **browser interface** (section 5) | Node.js and Rust, or only a browser if someone publishes the page |
| reproduce the study's numbers | the **ready-made experiments** (section 6) | Rust |
| analyse your institution's payments | the **`bound` and `settle`** commands (section 7), or the "Analyse your file" panel of the interface | Rust (or the interface) |

## 4. Install

The full step-by-step, for Windows and Linux, from a computer with nothing installed, is in the
README, section [Installation from scratch](../README.md#installation-from-scratch). In short:

1. Install **Git**, **Rust** (with `rustup`) and, for the interface, **Node.js LTS**.
2. Get the code:

```bash
git clone https://github.com/fabiodrneles/loopsettle.git
```

3. Enter the folder and check that everything works (the first build takes a few minutes):

```bash
cd loopsettle
```

```bash
cargo test --workspace
```

Every command in this guide is run **from the `loopsettle` folder**.

## 5. Use the browser interface

To open it:

```bash
cd web && npm install && npm run wasm && npm run dev
```

Then open `http://localhost:5173`. The simulation runs in the browser itself; nothing is sent to
a server.

**The controls**

- **Participants, payments:** the size of the scenario. On a phone the interface reduces it by
  itself.
- **Receiver imbalance:** how much some participants receive more than they pay. It is the
  model's main assumption; moving it shows whether the result depends on it.
- **Cycle search window** (0.5 to 10 s) and **maximum delay** (2 to 10 s): the micro-cycle
  parameters.
- **Seed:** change it to see another day drawn with the same rules.
- **Language:** the button in the corner, or `?lang=pt` / `?lang=en` in the address.

**The panels, and how to read them**

| Panel | What it shows | How to read it |
|---|---|---|
| **The network** | participants in a circle (size = activity; dashed = pays more than it receives) and the flows between them | in green, the part of the flow that goes round in loops (maximum circulation); in grey, the part that only goes one way |
| **Potential vs realisable** | the "ladder" of measures: from what could be netted on paper to the liquidity actually saved | the bigger the drop from one step to the next, the bigger the gap between what looks possible and what is achieved |
| **The day in 15 minutes** | the value paid in each interval and the sum of debit positions | if the dark line rises for hours, liquidity is driven by long imbalances that seconds of waiting cannot fix |
| **Liquidity by design** | the liquidity each settlement design needs, from gross to the theoretical minimum (LB) | the lower the bar, the less idle money |
| **Saving vs window** | the micro-cycle saving for each window | close to zero means that waiting a few seconds does not help |
| **Constrained liquidity** | with less balance than needed: value refused, delay and the 2.8 s target | compares designs when money is short |
| **Liquidity floor** | the floor for each deadline `d`, with each design's result on top | no design can go below the floor; the distance to it is what could still be gained |
| **Analyse your file** | the same numbers for a CSV of yours (format in section 7) | the file never leaves the computer |

Browser scenarios are smaller than the study's (up to 50 participants and 100,000 payments), so
the numbers on screen are not the paper's. In small scenarios the micro-cycle saving may come out
slightly negative: the queue changes the order of settlements. It is not a bug.

## 6. Run the ready-made experiments

Every number of the study comes from a command. To see the list:

```bash
cargo run --release -p loopsettle-cli -- list
```

To run one (for example, the central comparison):

```bash
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable
```

Add `--quick` for a version ten times smaller, in seconds. To run everything and rebuild the
summary:

```bash
make experiments summary
```

**Where results go.** Each experiment writes to `results/<name>/`:

| File | Content |
|---|---|
| `*.csv` | the tables, one row per case; they open in any spreadsheet |
| `summary.json` | the main numbers, the seed, the code version and the command that produced everything |
| `run.json` | time and memory of the run |
| `*.svg` | the figures; they open in a browser |

The easiest way to read them is the summary of all experiments, in tables, in
[`results/SUMMARY.md`](../results/SUMMARY.md). It is regenerated by command at every run.

**How to read a number with uncertainty.** Experiments repeat the day with several seeds and show
the mean with a 95% confidence interval, like this: `1.42% [0.57; 2.26]`. Read it as "the mean
was 1.42%, and the model's true value is probably between 0.57% and 2.26%". If the interval of a
difference includes zero (for example `−0.02 [−0.05; +0.01]`), you cannot say there was an effect.

## 7. Analyse your own payments

**File format.** A CSV with one row per payment and these columns:

| Column | Content | Example |
|---|---|---|
| `time` or `t_ms` | time: seconds, `HH:MM:SS`, or milliseconds (`t_ms`) | `40168` |
| `from` | who pays (any identifier) | `P003` |
| `to` | who receives | `P000` |
| `amount` | value in currency units, up to 2 decimals | `43.20` |
| `status` (optional) | only settled rows are used (`settled`, `ACSC`, `liquidado` and equivalents) | `settled` |

Rows must be in time order. If they are not, sort them first (it works on files larger than
memory):

```bash
cargo run --release -p loopsettle-cli -- sort raw.csv payments.csv
```

To practise without real data, use the example file `data/samples/payments_example.csv` (5,123
payments, 20 participants).

**Step 1: the bounds.**

```bash
cargo run --release -p loopsettle-cli -- bound data/samples/payments_example.csv
```

The output starts like this:

```text
UB (liquidity for real-time gross settlement): 588035.67
LB (one netting at the end of the file's period): 401155.75  (LB/UB = 0.6822)
```

Read it as: under real-time gross settlement the whole system needs 588,035.67 of liquidity; with
a single netting at the end of the day, 401,155.75 (68% of that). The difference is the most that
**any** mechanism can save.

Then comes a table, one deadline `d` per row:

```text
max_delay_s,delay_bound,delay_bound_pct_of_ub,liquidity_floor,tight_bound,tight_bound_pct_of_ub
10,33458.2,5.689825,588035.67,0,0
900,94397.33,16.052994,557184.66,30851.01,5.246452
3600,193333.8,32.877903,513820.7,74214.97,12.620828
```

The column that matters is `tight_bound_pct_of_ub`: the largest saving of any mechanism that
settles each payment within `max_delay_s` seconds. In this file, for 10 s, **0%**: no 10 s
mechanism saves anything. With 15 minutes (900 s), up to 5.2%; with one hour, up to 12.6%.
`liquidity_floor` is the floor (UB − largest saving). `delay_bound` is a simpler, looser form of
the same bound.

**Step 2: the designs.**

```bash
cargo run --release -p loopsettle-cli -- settle data/samples/payments_example.csv
```

It adds a table, one design per row:

```text
engine,liquidity,saving_vs_ub_pct,delay_p50_ms,delay_p95_ms,share_within_2800ms,value_share_in_cycles
gross_real_time,588035.67,0,0,0,1,0
micro_cycle_all_w500_d10000,588035.67,0,9984,9984,0.008198,0.029111
```

`saving_vs_ub_pct` is the saving; `delay_p50_ms` and `delay_p95_ms`, the delays in
milliseconds; `share_within_2800ms`, the share settled within 2.8 s (the Pix target requires 0.5
or more); `value_share_in_cycles`, the share of value settled in cycles. Choose the designs with
`--engines` (section 8) and the deadlines with `--delays 1,10,60`.

**Step 3: a report to share.**

```bash
cargo run --release -p loopsettle-cli -- bound payments.csv --report
```

It writes `report.md`, `report.pt-BR.md` and a figure, with the numbers, the code version, a
fingerprint of the file (not its content) and a fixed paragraph of interpretation and limitations.

**Where the output goes.** To `loopsettle-out/<file name>/`, or to the folder given with `--out`.

**Privacy.** By default the output has system totals only. With `--per-participant` there is one
row per participant, with names replaced by `P001`, `P002`... and it is refused when there are
fewer than 5 participants (`--min-participants`). Original identifiers appear only with
`--keep-names`.

## 8. Choose and write settlement designs

Each design is written as a short text. Times in milliseconds.

| Text | Design |
|---|---|
| `gross` | real-time gross: settle on arrival if the balance allows (Pix) |
| `deferred:96` | net everything at 96 fixed times a day (every 15 min) |
| `micro:all:500:10000` | micro-cycles: every payment waits; every 0.5 s cycles are searched; whatever waited 10 s goes gross |
| `micro:shortfall:500:10000` | micro-cycles only when the balance is short |
| `queue:fifo:10000` | plain queue: wait up to 10 s for incoming money, no netting |
| `hold:10000` | hold every payment 10 s and settle gross (separates the effect of waiting from the effect of netting) |
| `gridlock:fifo:60000:3600000` | Bech & Soramäki (2001) gridlock resolution, every minute |
| `netq:fifo:60000:3600000` | Koponen & Soramäki (1998) complete netting of queues, every minute |
| `bilateral:all:500:10000` | pairwise offsetting only (A↔B) |

Example:

```bash
cargo run --release -p loopsettle-cli -- settle payments.csv --engines gross,deferred:96,micro:all:500:10000
```

Which real system uses which design (Pix, FedNow, UPI...): see
[`INSTANT_PAYMENTS.md`](INSTANT_PAYMENTS.md).

## 9. Change a scenario

Experiments read text files in `experiments/<name>.toml`. To test another hypothesis, copy one,
edit it and run it with `--config`:

```bash
cargo run --release -p loopsettle-cli -- experiment baseline --config my-scenario.toml
```

The most useful keys, in the `[scenario]` section:

| Key | What it changes |
|---|---|
| `seed` | the seed |
| `participants` | number of participants |
| `payments_per_day` | payments per day (the sample; the scale factor converts to real size) |
| `days` | number of days |
| `receiver_bias_sigma` | receiver imbalance (0 = balanced) |
| `calibration` | file with the public figures (`data/calibration/pix.toml`) |
| `amounts_fit` | `percentiles` (default: value shape fitted to the central bank's percentiles) or `mean` (previous method) |
| `hourly_profile` | `default` or `spi2024` (time-of-day profile) |

The full list, with the source of each value, is in [`PARAMETERS.md`](PARAMETERS.md).

## 10. Common problems

| Symptom | What to do |
|---|---|
| `cannot read experiments/....toml` | run the command from the `loopsettle` folder |
| the file is refused for being out of order | sort it with `loopsettle sort` (section 7) |
| `--per-participant refused` | the file has fewer than 5 participants; use the totals only, or change `--min-participants` knowing that few rows can identify payments |
| rows skipped (`rows skipped (invalid)`) | check separator, header and values; the command reports how many were skipped |
| the first build is slow | it is normal (minutes); the next ones are fast |
| slow computer or little memory | use `--quick`; each experiment uses a few tens of MiB |
| interface numbers differ from the study's | normal: the browser scenario is smaller |

## 11. Going further

| To understand... | Read |
|---|---|
| each metric, with formula and example | [`METHODOLOGY.md`](METHODOLOGY.md) |
| where each parameter comes from | [`PARAMETERS.md`](PARAMETERS.md) |
| what the study can and cannot claim | [`CLAIMS.md`](CLAIMS.md) |
| the model's limits | [`LIMITATIONS.md`](LIMITATIONS.md) |
| all the numbers | [`results/SUMMARY.md`](../results/SUMMARY.md) |
| the data format | [`data/README.md`](../data/README.md) |
| using the simulator as a Rust library | the `README.md` of each folder in `crates/` |
