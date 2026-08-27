# Design so far (through PR 5)

Status of the calculator after the workspace scaffold (PR 1), the hourly OCPI collector (PR 2), domain newtypes (PR 3), the NPV identity (PR 4), and daily-index / Epoch ingest (PR 5). These are on `main` at [github.com/shauryagu/gpu-own-vs-rent](https://github.com/shauryagu/gpu-own-vs-rent) once PR 5 merges. The binary is still `chi`.

Open questions, resolutions, and answer-by dates live in [`docs/open-questions.md`](open-questions.md). The binding spec is [`docs/designs/2026-08-22-gate-0-1-mvp.md`](designs/2026-08-22-gate-0-1-mvp.md). This note is a map of what the code actually does.

---

## 1. What this is

Given public GPU rental prices and declared capital inputs, find which assumptions about a chip’s life, use, and resale value make **owning** it a fair deal versus **renting** — and how far that is from standard accounting (\(T = 6\,\mathrm{y}\), \(R = 0\)).

“Fair” means discrete NPV of buy-and-earn-rent is zero. It is **not** an optimum and **not** “minimize basis risk until buy wins.” Observed rental \(S\) is given. \(\Theta(S)\) is every \((T, u, R)\) consistent with that \(S\). Many \(\theta\) fit one \(S\); that set is the result.

We **consume** Ornn OCPI. We do not publish an index, operate a venue, or price residual-value swaps.

Bandi & Su already showed a GPU-hour is not storable, so cash-and-carry from spot to futures fails ([`docs/source/AiComputeAssetPricing.md`](source/AiComputeAssetPricing.md) §5.1; arXiv:2607.12156). We cite that. We do not re-derive it. Their object is the perishable *service flow*. Ours is the durable *hardware stock* that produces it.

---

## 2. Decisions that are locked

### Product

| Decision | Why it stays |
|---|---|
| Two named inverses forever: leftover \(L\) and salvage \(R^{\star}\) | Opposite signs in \(S\), different units. Do not call either “implied residual.” Gate 4 will have two surfaces, \(\Theta_L(S)\) and \(\Theta_{R^{\star}}(S)\). |
| Invert \(S\) is OCPI **latest daily-index** only | Teaching fixture (2026-08-21 close): daily-index `2.879583333333333`, history last point `2.88`. Live 2026-08-25 H100 daily-index is `2.9679166666666665`, history last `2.97`. Those are still different series. |
| Hourly current is collected anyway | Updates hourly and cannot be reconstructed from the free 3-month daily window. Lost hours are gone. |
| No silent defaults for \(P\), \(r\), primary \(T\), primary \(u\) | Unknowns are declared or later swept. Accounting \(T=6, R=0\) is a *labeled overlay*, not the primary \(\theta\). |
| `--residual-cents` required to print \(F(\theta)\) | Residual is never silently 0 except the accounting scenario. |
| A100 SXM4 fail-closed on invert | Do not guess 40 vs 80 GB. |
| H100 SXM → H100e is the identity map only | FLOPS maps for other SKUs are Gate 5. |
| Free public data only | No Ornn key, no paid history, no Silicon Data. A 401 on a “free” path is a bug. |
| Half-life is never estimated | Not a parameter of this MVP. Sweep later if at all. |
| Discrete annual NPV, \(H = 8760\) | Learnable on paper. No root finder. No continuous-time veneer. |
| Energy default \(\pi = 0\), PUE \(= 1.0\) | Product \(e\) is still evaluated. Gate 3 sweeps PUE / LMP. |
| Do not implement `docs/project-plan.md` | That is the seed 7-day reservation simulator (R1–R4, occupancy grid). Out of scope. |

### Architecture

| Decision | Why it stays |
|---|---|
| Crate graph: `chi` → `ingest` + `project`; both → `domain`. `chi_log` unlinked stub | `domain` has no I/O. Replay is Gate 2. \(\Theta\) sweep is Gate 4. |
| Binary package name `chi`, log crate `chi_log` | Positioning already documents `-p chi`. Crate name `log` collides on crates.io. |
| One trait in the MVP: `ingest::HttpGet` | Collector tests never hit the network. No trait objects in `domain`. |
| Blocking `reqwest`, 15 s timeout, no Tokio | Six sequential GETs from a cron process. A hung Ornn must fail, not overlap the next hour. |
| `FetchedAt` injected; `now()` only at the `chi collect` edge | Tests pass a frozen timestamp. |
| Gate 0 does not construct `UsdPerGpuHour` | Couples the irrecoverable write to the type model and invites using current as invert \(S\). |
| Storage is JSONL + timestamped raw files | As-of queries that would justify SQLite are Gate 6. |
| Internal newtypes, not `uom` | Compile-fail doctests are the lesson. |
| Money is `rust_decimal`; `f64` only for physics | OCPI `index_value` never transits `f64`. |
| Three serde scales, three jobs | Ingest stores **source token text**. Domain serde is **display scale** (USD 2, USD/GPU-hour 4). Invert JSON later uses **`round_dp(12)`**. They are not interchangeable. |

### Collector operations (PR 2 hardening)

Ornn is a public REST **snapshot**, not a sequenced market-data session. There is no sequence number, gap-fill, or replay stream. We do not invent one.

What we did instead:

- Exclusive `data/collect.lock` (`File::try_lock`); kernel-released on crash.
- Raw bodies via `*.tmp` + `rename` after `sync_all`.
- Each JSONL line `write_all` + newline + `sync_all`.
- Reject a current body whose `gpu_name` does not match the requested GPU (raw still kept).
- Non-200 and parse failures do not append JSONL.
- Attempt every GPU; exit 1 if any failed or if the free list is empty.
- One transport retry, then fail that GPU. Do not invent a missed hour.
- Sequential fetches, 200 ms pause, no API key, `User-Agent: chi-collector/0.1 …`.

---

## 3. Functionality that exists today

| Command / API | Behavior |
|---|---|
| `chi --help` | Lists `collect` and `invert`. |
| `chi collect` / `chi collect --series current` | Live: `GET /api/gpu-types-free`, then `GET /api/gpu/{name}` for each free GPU. Writes raw envelopes and appends `ocpi.hourly.v1` JSONL. |
| `chi collect --series daily` | Fetches free-list, `/api/daily-index/all`, per-GPU `/api/daily-index?gpu=`, and history. Raw envelopes only. No hourly JSONL. |
| `chi collect --series epoch` | Fetches Epoch `ml_hardware.csv` into `data/raw/epoch/`. |
| `chi invert …` | `bail("not implemented")` — PR 6. |
| Domain arithmetic | Legal `Mul`/`Div` only. `Usd + UsdPerGpuHour` does not type-check. |
| Domain money serde | Decimal **strings**. A JSON number is a schema error. Round-trip is **not** identity for long tokens (`2.879583333333333` → `"2.8796"`). |
| H100e conversion | `to_h100e` is identity for `GpuModel::H100Sxm`; every other SKU is `ConvertError::NotInMvp`. |
| Tests | Fixture HTTP only. No live network in CI. |
| launchd | `scripts/ocpi-hourly.plist` + `scripts/install-ocpi-hourly.sh` shipped. **Not installed by the merge.** Hours are still being lost until a human runs one collect and loads the agent. |

What is **not** here: invert CLI (PR 6), event log, SQLite, server, UI, Kalshi.

---

## 4. Key files and functions

### Crate graph

```
chi (binary)  -->  ingest  -->  domain
              -->  project -->  domain
chi_log (stub, unlinked)
```

`domain` may depend on `rust_decimal`, `thiserror`, `time`, `serde` only. Forbidden in `domain`: `tokio`, `reqwest`, `std::fs`, `clap`, `anyhow`.

### Binary — `crates/chi`

| File | Role |
|---|---|
| `src/main.rs` | `enum Cmd { Collect, Invert }`. Do not re-route in later PRs. |
| `src/collect.rs` | `--data-dir` (default `data`), `--series` (`current` default). Injects `OffsetDateTime::now_utc()`, `LiveHttp`, `RawCache`. |
| `src/invert.rs` | Stub. PR 6 fills this file only. |

### Ingest — `crates/ingest`

| Symbol | File | Role |
|---|---|---|
| `HttpGet` / `LiveHttp` / `FixtureHttp` | `src/http.rs` | The only MVP trait. 15 s timeout. One transport retry. Fixtures map URL → `fixtures/ocpi/…`. |
| `RawCache` | `src/cache.rs` | Timestamped raw writes, JSONL append, 200 ms pause, `try_lock`. |
| `CollectLock` | `src/cache.rs` | Exclusive lock on `data/collect.lock`. |
| `write_raw` | `src/cache.rs` | `*.tmp` → `sync_all` → `rename`. Bodies bit-identical to the HTTP bytes. |
| `append_hourly_line` | `src/cache.rs` | One JSON object + newline + `sync_all`. |
| `collect_current` | `src/ocpi_current.rs` | Lock, fetch list, loop GPUs, parse, append. |
| `parse_gpu_types_free` | `src/ocpi_current.rs` | Ornn envelope: `success` + `data[].gpu_name`. Names are not hard-coded. |
| `parse_current_body` / `parse_index_value` | `src/ocpi_current.rs` | `success` + `data.{gpu_name,index_value,last_updated}`. Token via `arbitrary_precision` + `Decimal::from_str_exact`. Shared with daily-index. |
| `CurrentQuote` | `src/ocpi_current.rs` | Ingest-local. `index_value: String`. Not `UsdPerGpuHour`. |
| `parse_daily_index_wrapper` / `DailyIndexRecord` | `src/ocpi_daily.rs` | Wrapper `{fetched_at, source_url, body}` → `ObservedSpot` (`OcpiDailyIndex`). Inner-only JSON is `Err`. |
| `parse_daily_history` | `src/ocpi_daily.rs` | Ingest-local points. Last point `2.88` is not invert \(S\). No `SpotSeries::OcpiDailyHistory`. |
| `collect_daily` | `src/ocpi_daily.rs` | Lock, free-list, all, per-GPU daily-index + history. Reuses `write_raw`. |
| `parse_ml_hardware_csv` / `EpochRow` | `src/epoch.rs` | TDP + annotation `release_price_usd`. Never `Theta.purchase`. |
| `row_for_gpu` | `src/epoch.rs` | H100 SXM → `NVIDIA H100 SXM5 80GB` (TDP 700). A100 / RTX 5090 + energy → `Err`. |

On-disk after a successful collect:

```
data/
  collect.lock                         # advisory; gitignored
  ocpi-hourly.jsonl                    # append-only; gitignored
  raw/ocpi/gpu-types-free/{fetched_at}.json
  raw/ocpi/current/{gpu_slug}/{fetched_at}.json
  raw/ocpi/daily-index/{gpu_slug}/{fetched_at}.json
  raw/ocpi/daily-index-all/{fetched_at}.json
  raw/ocpi/daily-history/{gpu_slug}/{fetched_at}.json
  raw/epoch/ml_hardware/{fetched_at}.csv
```

JSONL schema `ocpi.hourly.v1`: `series = ocpi.current`, `index_value` as source token string, `valid_on: null`, `fetched_at` (ms), `raw_sha256`, `source_last_updated`. Raw filenames use nanoseconds so two collects in the same UTC second do not collide.

### Domain — `crates/domain`

| Symbol | File | Role |
|---|---|---|
| `Usd`, `UsdPerGpuHour`, `UsdPerKwh` | `src/money.rs` | `from_cents` / `TryFrom<Decimal>`. Display/serde scales 2 / 4 / raw. |
| `GpuHour`, `Years`, `Utilization`, `DiscountRate` | `src/qty.rs` | Validated constructors. `u ∈ (0,1]`, \(T \ge 1\), \(r \ge 0\). |
| `Kilowatt`, `Hours`, `Pue` | `src/qty.rs` | Physics; reject non-finite `f64`. |
| `GpuModel`, `H100eHour` | `src/gpu.rs` | Five free SKUs. `H100eHour` inner is crate-private. |
| `FetchedAt`, `ValidOn`, `AsOf` | `src/time.rs` | Transaction / settlement / query time. `AsOf` unused until Gate 6. |
| `SpotSeries::OcpiDailyIndex`, `ObservedSpot` | `src/spot.rs` | No `OcpiCurrent` / `OcpiDailyHistory` variants. `FetchedAt` is not on this type. |
| `to_h100e`, `from_h100e_as_h100` | `src/convert.rs` | Identity for H100 SXM only. |
| Legal ops | `money.rs`, `qty.rs` | `UsdPerGpuHour * GpuHour → Usd`, `Usd / GpuHour → UsdPerGpuHour`, `Utilization * GpuHour → GpuHour`, `Kilowatt * Hours → f64`. |

PR 4 added `theta.rs`, `identity.rs`, `energy.rs`, `tests/proptest_identity.rs`. `leftover` / `implied_salvage` take `ObservedSpot`.

### Fixtures and scripts

| Path | Role |
|---|---|
| `fixtures/ocpi/gpu-types-free.json` | Live free-list snapshot. |
| `fixtures/ocpi/current/*.json` | Live current-price snapshots (not invert \(S\)). |
| `fixtures/ocpi/daily-index/H100_SXM.json` | Invert \(S\) wrapper. Teaching token `2.879583333333333`, date `2026-08-21T20:00:00.000Z`. |
| `fixtures/ocpi/daily-index-all.json` | Collect fixture (live 2026-08-25 capture). Invert must not open it. |
| `fixtures/ocpi/daily-history/H100_SXM.json` | Teaching last point `2.88`. Not invert \(S\). |
| `fixtures/epoch/ml_hardware.excerpt.csv` | Header + H100 / H200 / B200 rows. |
| `scripts/ocpi-hourly.plist` | launchd job, minute 5, `KeepAlive` false. |
| `scripts/install-ocpi-hourly.sh` | Substitutes binary / repo / data dir and `launchctl load`. |

---

## 5. Theory the next PRs must implement, not invent

PR 4 writes these formulas in `domain`. They are already specified. Do not “improve” them from first principles.

Owner cash flows for one GPU:

- \(t = 0\): pay \(P\).
- \(t = 1,\ldots,T\): receive \((S - e)\, h\), with \(h = u \cdot 8760\).
- \(t = T\): receive salvage \(R\).

Fair deal \(\Leftrightarrow\) NPV \(= 0\):

\[
P = (S - e)\, h\, A(r,T) + R\, (1+r)^{-T}
\]

\[
A(r,T) =
\begin{cases}
\dfrac{1 - (1+r)^{-T}}{r} & r \neq 0 \\[8pt]
T & r = 0
\end{cases}
\]

**Forward** (only when \(R\) is declared):

\[
F(\theta) = e + \frac{P - R\,(1+r)^{-T}}{h\, A(r,T)}
\qquad [\mathtt{UsdPerGpuHour}]
\]

Capital-recovery rent at scrap-zero (not a second name for \(F\)):

\[
F_{\mathrm{capital}}(P,T,u,r) = \frac{P}{h\, A(r,T)}
\]

**Leftover** (rises with \(S\); unit `UsdPerGpuHour`):

\[
L(S) = S - F_{\mathrm{capital}} - e, \qquad \partial L/\partial S = 1
\]

**Salvage inverse** (falls with \(S\); unit `Usd` at year \(T\); algebra, not a solver):

\[
R^{\star}(S) = (1+r)^{T}\bigl(P - (S - e)\, h\, A(r,T)\bigr), \qquad \partial R^{\star}/\partial S < 0
\]

Negative \(R^{\star}\) is valid: even scrap-zero ownership beats renting at that \(S\). Do not clamp.

Round-trip tests (PR 4): \(F(T,u,R^{\star}(S),\ldots) = S\) and \(R^{\star}(F(\theta)) = R\) up to `Decimal` epsilon. Do not print the tautology \(F(R^{\star})=S\) as a “second direction” when `--residual-cents` is omitted.

**Units check.** \((S-e)\) is `UsdPerGpuHour`; \(h\) is `GpuHour`/year; \(A\) is years; product is `Usd`. That is why the newtypes exist.

**What this is not.** \(F(\theta)\) is not a futures quote. Bandi & Su \(F_t(T)\) is a rental-index future. Cash-and-carry \(S_t \mapsto F_t(T)\) does not exist for a GPU-hour. Cite §5.1; do not build a carry engine.

Energy (still PR 4, even when \(\pi = 0\)):

\[
e = \mathrm{TDP}\,[\mathrm{kW}] \cdot 1\,[\mathrm{h}] \cdot \pi\,[\mathrm{USD/kWh}] \cdot \mathrm{PUE}
\]

`f64` for kW; money stays `Decimal`. The only `f64 → Decimal` on the path is inside `energy_per_gpu_hour` via `Decimal::from_f64_retain` (`Err` on NaN/Inf). Epoch release price \(33600\) is **not** \(P\).

---

## 6. Storage, disk I/O, and memory

The workload is **five small HTTP snapshots per hour**, not a scan-heavy query engine. Design for *not losing an hour* and *not rounding a price*, not for throughput.

### What we sized

| Stream | Volume |
|---|---|
| Hourly JSONL | 5 records/hour × ~200 B ≈ 9 KB/day |
| Hourly raw | ~1–2 KB × 5 ≈ 10 KB/hour → ~90 KB/day |
| A year of hourly raw | tens of MB |

Daily-index and 3-month history (PR 5) are a few tens of KB. Epoch CSV is cached whole and parsed from disk; we commit an excerpt fixture.

### I/O pattern (keep this)

1. **Write bytes first, parse later.** The raw file is the source of truth if JSONL parse rules change.
2. **Append-only JSONL.** A second collect cannot rewrite the first line.
3. **One process writer** (`collect.lock`). Two overlapping writers can tear a line; two *sequential* collects are two honest observations.
4. **fsync the JSONL line.** At 5 lines/hour this is free. Do not batch-fsync to “save I/O.”
5. **Atomic raw replace** (`tmp` + rename). A crash must not leave a truncated file under the final name.
6. **Whole response in a `Vec<u8>`.** Bodies are kilobytes. Streaming JSON would add code and no I/O win.
7. **No database, no mmap, no compaction.** `grep` on a year of JSONL is fine until Gate 6 as-of queries hurt. Then SQLite, not sooner.
8. **SHA-256 per body** is for Gate 2 content addressing, not a hot path.

### What not to add for “performance”

- Concurrent fetches or a Tokio runtime. The 200 ms pause is politeness on an unauthenticated, IP-rate-limited API.
- Connection pooling across hours. Each `chi collect` is a new process.
- Parsing current into `UsdPerGpuHour` to “save a later pass.”
- Reading the hourly JSONL in `invert`. Invert \(S\) is one committed daily-index fixture (PR 5/6).
- Sequence numbers, replay buffers, or gap-fill. Ornn has no session.

### Memory / numeric access

- Money stays on the stack as `Copy` newtypes around `Decimal` (28 significant digits). No heap per quantity.
- Do not enable `rust_decimal` float serde.
- Domain `Serialize` **rounds**. Putting OCPI tokens through it loses digits. Ingest keeps `String`.
- Physics (`Kilowatt`, `Hours`, `Pue`) may be `f64`; constructors reject NaN/Inf. Never `as f64` on `index_value`.

### Durability still open (ops, not a new crate)

- launchd does not retry a failed :05 run. That hour is gone unless a human re-runs.
- There is no “hour \(N\) is missing” detector yet.
- Stale Ornn `last_updated` is still recorded with *our* `fetched_at` (correct bitemporal split). We do not yet flag “source did not move.”
- First live `chi collect` + `install-ocpi-hourly.sh` has not been run.

---

## 7. What the next PRs build on

DAG from the approved design. PR 4 is on `main`. PR 5 is this branch.

```
PR1 scaffold ──► PR2 collector ──► PR5 daily-index / Epoch adapters ──┐
             └► PR3 newtypes   ──► PR4 identity + proptest ──────────┴► PR6 invert CLI
```

### PR 6 — invert CLI (depends on PR 4 + PR 5)

`chi invert --gpu "H100 SXM" --fixture-dir fixtures` with required `--purchase-cents --life-years --utilization --discount-rate`. Load **only** `fixtures/ocpi/daily-index/{slug}.json`. Always print \(L\), \(R^{\star}\), and the accounting point. Print \(F(\theta)\) only with `--residual-cents`. JSON computed money at `round_dp(12)`. No HTTP.

Build on: `leftover` / `implied_salvage` / `fair_rent`, daily-index parser, clap router already in `main.rs`. Fill `invert.rs`. Thin DTO in `project` is allowed; algebra stays in `domain`.

### Do not start yet

Gate 2 event log, Gate 3 PJM/PUE sweep, Gate 4 \(\Theta\) surfaces, Gate 5 FLOPS maps, Gate 6 `--as-of`. The hooks (`raw_sha256`, `chi_log` stub, `AsOf`, `H100eHour`, `Pue`) are already typed so those gates are not a rewrite.

### Learn-before still on the owner (Gate 1)

Before PR 4/6 feel cheap: Rust Book ch. 1–6 and 10; `rust_decimal`; discrete NPV on paper (write \(R^{\star}\) without a computer); Epoch H100 row; which series invert reads (daily-index, not current).

---

## 8. How to verify this snapshot

```bash
cargo test --workspace
cargo run -p chi -- collect --help
cargo run -p chi -- invert   # expected: not implemented
```

Live collect (irrecoverable data; do this on a machine you will leave on):

```bash
cargo build -p chi --release
cargo run -p chi -- collect --data-dir data --series current
# then: scripts/install-ocpi-hourly.sh
```
