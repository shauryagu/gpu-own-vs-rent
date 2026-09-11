# gpu-own-vs-rent

Own-vs-rent calculator for a GPU (binary: `chi`). Given public rental prices
and declared capital inputs, it reports leftover rent \(L\) and break-even
salvage \(R^{\star}\). If you also declare salvage, it prints fair rent
\(F(\theta)\).

Fair means the discrete NPV of buy-and-earn-rent is zero. Not an optimum, not
an index, not an exchange. The program **consumes** the Ornn Compute Price
Index (OCPI). It does not publish prices.

Invert \(S\) is OCPI **daily-index** only (teaching token
`2.879583333333333`). Not hourly current, not history `2.88`. Leftover \(L\)
and salvage \(R^{\star}\) are two named inverses — never one “implied
residual.” `--residual-cents` is required to print \(F(\theta)\); `0` is
declared, not omitted. Accounting \(T=6\), \(R=0\) is a labeled overlay.

Do not implement [docs/project-plan.md](docs/project-plan.md) (seed 7-day
reservation simulator).

## Commands

| Command | What it does |
|---|---|
| `chi invert` | Frozen daily-index fixture → leftover \(L\) and salvage \(R^{\star}\). No HTTP. Does not read the event log. |
| `chi collect` | Writes timestamped raw bodies. `--series current` also appends hourly JSONL (`ocpi.current`). Lost hours cannot be reconstructed. |
| `chi replay` | Folds a log directory (`events.jsonl` + `cas/{sha256}`) to an ingest catalog. Fixture series is hourly current (`ocpi.current`, `"2.63"`), not invert \(S\). |

Invert always requires `--purchase-cents --life-years --utilization --discount-rate`. There are no silent defaults for \(P\), \(T\), \(u\), or \(r\).

## Run locally

Cargo is not on the default PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --workspace
```

Teaching invert (H100 SXM, declared \(\theta\), not a product default):

```bash
cargo run -p chi -- invert --gpu "H100 SXM" --fixture-dir fixtures \
  --purchase-cents 2500000 --life-years 5 --utilization 0.60 \
  --discount-rate 0.10 --format text
```

Collect hourly current (needs network; writes under `data/`):

```bash
cargo run -p chi -- collect --data-dir data --series current
```

After a release build on the **main** checkout, `scripts/install-ocpi-hourly.sh`
installs launchd. Do not install launchd from a worktree.

Replay the committed fixture log (no network):

```bash
cargo run -p chi -- replay --log-dir fixtures/log/v1 --format json
```

## Run with Docker

Optional packaging of the same CLI. Image build context is the repo root;
the Dockerfile, entrypoint, and Compose file live under `art/`. Compose
names three one-shot jobs. It does not replace launchd and does not invent
invert inputs.

From the repo root:

```bash
docker compose -f art/compose.yaml run --rm invert
docker compose -f art/compose.yaml run --rm replay
mkdir -p data
docker compose -f art/compose.yaml run --rm collect
```

`invert` in Compose is the teaching H100 example (same flags as above). Override
or extend args after `--`, for example `-- --residual-cents 0`. `collect` mounts
`./data` at `/data` and needs network. `replay` prints the fixture catalog of
hourly current, not leftover or salvage.

Image default with no command is `chi --help`.

## Docs

- [docs/vocab.md](docs/vocab.md) — names, units, and what each term is not
- [docs/status.md](docs/status.md) — what the code does today
- [docs/open-questions.md](docs/open-questions.md) — still open or deferred
- [docs/positioning.md](docs/positioning.md) — vs Bandi & Su and vs Ornn
- [docs/designs/2026-08-22-gate-0-1-mvp.md](docs/designs/2026-08-22-gate-0-1-mvp.md) — binding Gate 0+1 spec
- [docs/execution-plan.md](docs/execution-plan.md) — gate DAG
