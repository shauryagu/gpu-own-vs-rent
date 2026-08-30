# gpu-own-vs-rent

Own-vs-rent calculator for a GPU. Given public rental prices and declared
capital inputs, it reports leftover rent \(L\) and break-even salvage
\(R^{\star}\) — and, if you declare salvage, fair rent \(F(\theta)\).

It **consumes** the Ornn Compute Price Index (OCPI). It does not publish an
index, run an exchange, or price residual-value swaps. The binary is `chi`.

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --workspace

cargo run -p chi -- invert --gpu "H100 SXM" --fixture-dir fixtures \
  --purchase-cents 2500000 --life-years 5 --utilization 0.60 \
  --discount-rate 0.10 --format text
```

Invert \(S\) is OCPI **daily-index** only (teaching token
`2.879583333333333`). Not hourly current, not history `2.88`.
`--residual-cents` is required to print \(F(\theta)\); `0` is declared, not
omitted. Accounting \(T=6\), \(R=0\) is a labeled overlay.

```bash
cargo run -p chi -- collect --help
```

`collect --series current` appends hourly JSONL. Lost hours cannot be
reconstructed. After a release build, run one collect and
`scripts/install-ocpi-hourly.sh` if this machine should keep the series.

## Docs

- [docs/vocab.md](docs/vocab.md) — names, units, and what each term is not
- [docs/status.md](docs/status.md) — what the code does today
- [docs/open-questions.md](docs/open-questions.md) — still open or deferred
- [docs/positioning.md](docs/positioning.md) — vs Bandi & Su and vs Ornn
- [docs/designs/2026-08-22-gate-0-1-mvp.md](docs/designs/2026-08-22-gate-0-1-mvp.md) — binding Gate 0+1 spec
- [docs/execution-plan.md](docs/execution-plan.md) — gate DAG

Do not implement [docs/project-plan.md](docs/project-plan.md) (seed 7-day
reservation simulator).
