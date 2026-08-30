# AGENTS.md

## Vocabulary

Canonical terms live in `docs/vocab.md`. When a symbol or market word is
ambiguous — leftover vs salvage, \(F(\theta)\) vs a futures price, which OCPI
series is \(S\) — read that file before writing code, comments, or docs.
Load `.grok/skills/project-vocab/SKILL.md` (`/project-vocab`).
Do not invent a synonym if the term is missing; ask.

## Project

An own-vs-rent calculator for perishable compute **capacity as a chip**:
fair means discrete NPV of buy-and-earn-rent is zero. Compute (GPU rental)
is the live case study.

Binding spec: `docs/designs/2026-08-22-gate-0-1-mvp.md` (Gates 0+1).
Roadmap: `docs/execution-plan.md`. Positioning: `docs/positioning.md`.

**`docs/project-plan.md` is out of scope** (seed 7-day reservation
simulator: R1–R4, occupancy grid, hedge blotter, client clock). Do not
implement it.

**Standing constraint: free, publicly available data only.** No paid feeds,
no purchased history, no private venue data. Unknowns are declared or swept
— do not propose paid data as a fix for a modeling gap.

## Setup / test / build

- PATH: `export PATH="$HOME/.cargo/bin:$PATH"` (cargo is not on default PATH)
- Test: `cargo test --workspace`
- Invert (teaching H100):
  `cargo run -p chi -- invert --gpu "H100 SXM" --fixture-dir fixtures --purchase-cents 2500000 --life-years 5 --utilization 0.60 --discount-rate 0.10`
- Collect: `cargo run -p chi -- collect --data-dir data --series current`
- Release collect binary: `cargo build --release -p chi`
- Do not install launchd from a worktree. Plist points at
  `target/release/chi` on the main checkout.

## Architecture

```
chi (binary)  -->  ingest  -->  domain
              -->  project -->  domain
chi_log (stub, unlinked until Gate 2)
```

- **`domain` has no I/O** — no `reqwest`, `std::fs`, `clap`, `tokio`.
- **Invert \(S\)** is `{fixture_dir}/ocpi/daily-index/{slug}.json` only.
  Not hourly current, not history, not `data/`.
- **Collect** writes timestamped raw bodies + hourly JSONL (`ocpi.current`).
  Simulation clock is not this MVP. The client never advances time.
- **Event log is Gate 2.** Until then the file cache is the source of
  collect bytes; invert is fixture-deterministic.

## Non-negotiables

- Free data only — `docs/llms.txt`.
- Two named inverses forever: leftover \(L\) (`UsdPerGpuHour`, rises with
  \(S\)) and salvage \(R^{\star}\) (`Usd`, falls with \(S\), unclamped).
  Never “implied residual.”
- \(F(\theta)\) only with `--residual-cents` (zero is present, not omitted).
  Accounting \(T=6\), \(R=0\) is a labeled overlay, not primary \(\theta\).
- \(P\) is `--purchase-cents`, never Epoch release price.
- Half-life is never estimated.
- A100 SXM4 / RTX 5090 fail closed (do not guess 40 vs 80 GB).
- Whole-node / bid-price / R1–R4 are not this product.

## Do not

- Do not add paid API keys or purchased datasets.
- Do not treat hourly JSONL as invert \(S\).
- Do not collapse leftover and salvage under one name.
- Do not silently default \(P\), \(T\), \(u\), or \(r\).
- Do not re-derive Bandi & Su cash-and-carry failure; cite §5.1.
- Do not present \(F(\theta)\) as futures \(F_t(T)\).

## When context is missing

If a task needs a data source, threshold, or modeling choice not in
`docs/designs/2026-08-22-gate-0-1-mvp.md`, `docs/vocab.md`, or
`docs/execution-plan.md`, stop and ask. Do not invent \(P\), \(r\),
half-life, or a leftover/salvage synonym.
