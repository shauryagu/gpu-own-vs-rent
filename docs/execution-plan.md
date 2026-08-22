# GPU-hour identity — execution plan

A deliverable-gated plan, not a calendar. Each gate has something you can run
or show, learning you must do **before** that gate, and learning you get
**during** it. Later gates may be reordered, split, or dropped. Earlier gates
should not be rewritten after they have tests.

This file is the product roadmap. It is **not** a writing-plans TDD task list
and **not** an execute-plan PR DAG. See [Formats](#formats--do-not-convert-the-whole-project)
for when those get written, one gate at a time.

**Question this system answers.** Given public rental indices (OCPI) and public
capital, spec, and energy inputs, what feasible set of (economic life,
utilization, residual) makes owning a GPU a fair deal versus renting it — and
how far is that set from accounting practice?

**Feasible set (not geography).** \(\Theta(S)\): the set of
\(\theta = (T, u, R)\) such that a forward GPU-hour identity \(F(\theta)\)
matches observed OCPI \(S\) within a declared \(\varepsilon\). Many \(\theta\)
fit one \(S\). That non-uniqueness is the result.

**Computer-science object.** Compute and keep that inverse correct over a
streaming, heterogeneous, unit-incompatible ingest log. Every published number
is `event log ⊕ parameter vector ⊕ code version`.

**MVP.** Gate 0 (collector running) + Gate 1 (`invert` CLI on frozen fixtures).
Everything after that is a structured improvement on a running system.

---

## Formats — do not convert the whole project

Three documents, three jobs. Mixing them is how learning gets skipped and how
a 7-day simulator gets rebuilt by accident.

| Document | Job | When to write |
|---|---|---|
| **This file** (`docs/execution-plan.md`) | What to build, in what order, what “done” means, what to learn | Already written. Evolve the tail only. |
| **writing-plans TDD plan** (`docs/superpowers/plans/YYYY-MM-DD-<gate>.md`) | Exact files, failing tests, commits for *one* gate | After that gate’s **Learn before** is done, immediately before coding it |
| **execute-plan PR DAG** (`## PR Plan` inside a `/design` doc) | Parallel PRs in worktrees for an agent orchestrator | Only if a *single* gate has independent PRs *and* you want agents to implement them. Never for Gates 0–10 as one DAG. |

**Do not** turn Gates 0–10 into one writing-plans document. The writing-plans
skill’s own scope check says multi-subsystem specs become separate plans. This
project is eleven gates plus optional modules.

**Do not** turn Gates 0–10 into one `/execute-plan` DAG. That skill launches
implementer subagents in isolated worktrees and is designed so *you* do not
write the code. That fights the point of this project: you learn Rust and the
identity by building the MVP.

**Do** convert **one gate** into a TDD plan when that gate is next and its
pre-reading is done. Parallelism inside a gate is allowed (for example Gate 1
types before ingest). Parallelism *across* gates is almost never allowed on
the critical path.

---

## Gate DAG

Solid arrows are hard dependencies. Dashed arrows are “can start after, not
on the critical path.”

```
                    ┌──────────── Gate 0 collector ────────────┐
                    │                 │                        │
                    │                 ▼                        │
                    │            Gate 1 MVP invert             │
                    │           /     |      \                 │
                    │          /      |       \                │
                    │         ▼       ▼        ▼               │
                    │    Gate 2     Gate 3    Gate 5           │
                    │    log/replay cost      conversion       │
                    │         │       │        │               │
                    │         │       ▼        │               │
                    │         │    Gate 4 Θ    │               │
                    │         │       │        │               │
                    │         ▼       │        │               │
                    │    Gate 6 as-of │        │               │
                    │         │       │        │               │
                    │         └───────┼────────┘               │
                    │                 ▼                        │
                    │            Gate 7 live marks ◄───────────┘
                    │                 │
           Gate 8 venue ──────────────┤  (needs Gate 1; can run beside 3–6)
                    │                 ▼
                    │            Gate 9 thin client
                    │                 │
                    │                 ▼
                    │            Gate 10 write-up
```

| Path | Gates | Meaning |
|---|---|---|
| **Critical path to MVP** | 0 → 1 | First working product |
| **Critical path to live system** | 1 → 2 → 6 → 7 | Replayable marks |
| **Research object** | 1 → 3 → 4 | \(\Theta(S)\) surface |
| **Exchange kernel extras** | 1 → 5; 1 → 8 | Conversion factor; offer-vs-index basis |
| **Presentation** | 4 + 7 + 8 → 9 → 10 | UI then findings |

Optional later modules (used-market anchors, OTPI, SQLite, Sobol, seller
simulator, Bandi synthetic-futures time series) attach **after Gate 4**, never
before.

---

## Non-negotiables

- Free, public sources only (`docs/llms.txt`). No paid Ornn history, no
  Silicon Data, no private venue tape.
- Domain logic has no I/O. Ingest, clock, and storage sit behind traits.
- Money and physical quantities never share an untyped `f64`.
- Half-life / mean reversion of OCPI is **not** estimated. Unknowns are swept.
- Bid-price / reservation engine is out of MVP and out of v1 unless a later
  gate explicitly pulls it in.
- The hourly OCPI collector starts at Gate 0 and never stops. Lost hours
  cannot be reconstructed.
- You do the **Learn before** block of a gate *before* that gate’s TDD plan
  is written or coded. Skipping it is how the old 7-day simulator sneaks back
  in (Talluri, Schwartz, R1–R4 are not Gate 1).

---

## Tech stack (locked)

Rust workspace + thin TypeScript client later. Rust is the system language
because the CS thesis is making illegal unit joins unrepresentable.

| Layer | Choice |
|---|---|
| Language | Rust (stable), cargo workspace |
| Domain numerics | `rust_decimal` for USD; integer cents internally; `f64` only for physics |
| Units | Internal newtypes first (`GpuHour`, `UsdPerGpuHour`, `H100eHour`) |
| Time | One crate crate-wide (`time` or `chrono`) + `AsOf` / `FetchedAt` from day one |
| Ingest HTTP | `reqwest` + `tokio` |
| CLI | `clap` |
| Serialization | `serde` + JSONL event log in MVP |
| Errors | `thiserror` in libraries, `anyhow` at the binary edge |
| Tests | `cargo test` + `proptest` on the identity |
| Lint | `clippy -D warnings`, `rustfmt` |
| Storage (MVP) | Append-only JSONL + content-addressed raw cache |
| Storage (later) | SQLite via `sqlx` when as-of queries hurt |
| API (later) | `axum` + SSE |
| Client (later) | React + TypeScript + Vite |

```
crates/
  domain/     # quantities, F and inverse, GPU ids — NO tokio, NO fs, NO reqwest
  ingest/     # fetchers, raw cache, source adapters → domain events
  log/        # append-only event log, replay
  project/    # projections: cost stack, Θ surface, basis panel
  cli/        # clap binary: fetch, invert, sweep, replay
  api/        # (later) axum
```

`domain` depends on nothing else in this repo.

---

## How to use a gate

For every gate below:

1. Finish **Learn before**. Do not start a TDD plan or open a PR until you can
   explain the “check you understand” item out loud.
2. Then write a *one-gate* writing-plans file if the implementation is more
   than an afternoon. Gate 0 does not need one (small, already specified).
   Gate 1 does.
3. Build until **Done when** is true. `cargo test` and `clippy` green.
4. The **During** list is what the work is supposed to teach. If you finish
   the deliverable without touching those ideas, the gate was implemented
   wrong (usually: Python notebook, untyped `f64`, or estimated half-life).

Do **not** pre-read the next gate’s list while building the current one,
except the items marked *also unblocks later*.

The old `docs/reading-list.md` is mapped to the *seed* 7-day reservation
simulator. Ignore its Day-2 Talluri / Schwartz / PJM-estimator track until a
later gate or the optional seller simulator explicitly pulls it in.

---

## Gate 0 — Collector + positioning

**Status.** Positioning note exists (`docs/positioning.md`). Collector is
**not** running. This gate is not done.

**Deliverable.** (1) Positioning note — done. (2) Hourly OCPI current-price
poller that appends to a local file for the five free GPUs, with `fetched_at`.
Running, not designed.

**Done when.** A `curl`-equivalent fetch succeeds; a file grows every hour;
the note states the question, the free-data constraint, and what we will not
estimate.

**Not this gate.** Parsing into domain types; a database; a web server.

### Learn before Gate 0

| Item | Why | Check you understand |
|---|---|---|
| Install Rust (`rustup`) and confirm `rustc --version`, `cargo --version` | Collector is a tiny Rust binary. You cannot schedule what you cannot compile. | Both commands print a version. |
| `curl` the OCPI current-price endpoint once; save the raw body | You must see hourly current as a JSON blob before wrapping it. | You can point at `price` vs any date field in the response. |
| Bandi & Su §4.1 (local extract `docs/source/AiComputeAssetPricing.md`) | Why we do **not** try to turn spot into a future via cash-and-carry. | “A GPU-hour unused today cannot be delivered next month.” |
| Ornn, *The Basis Gap* (20 min) | Vocabulary: spot, utilization, residual. Those are \(S\), \(u\), \(R\). | Basis = what the contract guarantees − what the position actually is. |
| Ornn, *Compute Futures* settlement section | Hourly current ≠ daily settle ≠ Asian monthly average. | Name those three series and which one we collect at Gate 0. |
| `docs/positioning.md` | The question this repo answers, vs Ornn, vs Bandi. | We consume OCPI; we do not publish an index. |

**Skip until much later:** Talluri & van Ryzin, Littlewood, Schwartz (1997),
Lucia & Schwartz, Hull hedge-ratio chapter, the seed plan’s R1–R4 regimes.

### During Gate 0

HTTP + cron/launchd; treating a paper section as a spec; why a missed hour
cannot be reconstructed.

---

## Gate 1 — MVP: typed identity, one inverse, one CLI

The first thing that is the product.

**Deliverable.** `cargo run -p cli -- invert --gpu "H100 SXM"` prints latest
daily OCPI \(S\), a declared \(\theta\), implied residual **or** implied rent
\(F(\theta)\) both directions, and the accounting point \((T=6\text{y}, R=0)\)
versus that \(S\).

**Done when.**

- `domain` inverts the NPV identity for \(R\) in closed form and evaluates
  \(F(\theta)\).
- GPU names and quantities are newtypes. Adding `Usd` to `UsdPerGpuHour`
  does not compile (or a typed test fails).
- OCPI + Epoch ingest is cached raw and parsed once.
- `proptest`: increasing \(S\) increases implied \(R\); H100e conversions
  round-trip; energy term is `kW · h · USD/kWh`.
- CLI is deterministic given a frozen cache directory.

**Not this gate.** Sweeps, UI, AWS, PJM, event log, live hourly marks, OTPI.

### Learn before Gate 1

| Item | Why | Check you understand |
|---|---|---|
| Rust Book ch. 1–6 and 10 (ownership, structs, enums, `Result`, traits, modules) | Gate 1 *is* the Rust course. You need enough to read a three-crate graph. | You can explain why `domain` cannot take a `reqwest::Client`. |
| `rust_decimal` README (15 min) | Money is not `f64`. | When `f64` is allowed (FLOPS, kW) and when it is not (USD). |
| Newtypes / “make illegal states unrepresentable” (one short note is enough) | `Usd` + `UsdPerGpuHour` must not type-check. | You can name three quantities that look like floats and must not mix. |
| Discrete NPV algebra, on paper | Inverse for \(R\) is closed form. Do not binary-search it. | Given \(S, T, u, P, r\), you can write \(R\) without a computer. |
| Epoch ML hardware CSV: open it, find the H100 row | Specs, TDP, conversion, launch price if present. | You know which columns we will read and which we will ignore. |
| OCPI **daily** history free window (3 months, 5 GPUs) | Gate 0 collected *hourly current*. Gate 1 inverts *daily settle*. | You can say which series `invert` reads. |

**Skip:** event sourcing, axum, React, AWS Price List, hedonic regression,
Sobol, CoreWeave 10-K (that is Gate 3).

### During Gate 1

Ownership on a small crate graph; newtypes; `Result` vs panic; `rust_decimal`;
NPV as an operator; closed-form inverse vs root-finding.

---

## Gate 2 — Event log and replay

**Deliverable.** Append-only log of ingest events (`SourceFetched`,
`SeriesParsed`). `cli replay` twice on the same log directory produces
byte-identical projection JSON.

**Done when.** A test copies a fixture log, replays twice, asserts equality.
Cache files are payloads the events point at (content hash).

**Not this gate.** User command events, SSE, forking runs.

### Learn before Gate 2

| Item | Why | Check you understand |
|---|---|---|
| Fowler, *Event Sourcing* (or equivalent one-pager) | Log is the source of truth; projections are functions of the log. | Why overwriting a `prices` table is forbidden. |
| `serde` internally/externally tagged enums | Events are an enum. Tag mistakes silently drop variants. | You can sketch the JSON for `SourceFetched`. |
| Content-addressed storage (hash the raw body, store once) | Replay must find the same bytes. | Event points at a hash, not at “the latest file.” |

### During Gate 2

Event sourcing in the small; fixture-driven tests; why the CLI becomes a
projection runner, not a place that “has state.”

---

## Gate 3 — Cost stack and energy

**Deliverable.**

```
S  ≈  capital_recovery  +  energy  +  unexplained
```

Energy = Epoch TDP × swept PUE × a named public LMP snapshot (or a documented
constant with the source named). Purchase is a **band**. Accounting
\(T=6\text{y}, R=0\) is a labeled scenario, not a hidden default.

**Done when.** Energy is computed, not invented, and is visibly small or not
next to OCPI (that comparison is the finding).

**Not this gate.** Full LMP history as a trading signal; residual from used
listings.

### Learn before Gate 3

| Item | Why | Check you understand |
|---|---|---|
| Dimensional analysis: `kW · h · USD/kWh = USD` | Energy term is a unit check, not a model. | You can write the energy line with units on every factor. |
| What PJM LMP is (one explainer page) | Named energy price, not a trading signal. | LMP is \$/MWh at a node, not a GPU price. |
| One CoreWeave (or hyperscaler) 10-K footnote on useful life | Accounting scenario comes from a filing, not a vibe. | You can quote the useful-life number and the filing date. |
| Why opex (opex besides energy) is omitted or swept | We will be asked “where is staff / building?” | Because it is unobserved; sweeping it is honest, inventing it is not. |

### During Gate 3

EDGAR as a *parameter source*, not a time series; reading a footnote;
separating computed energy from unexplained residual.

---

## Gate 4 — Feasible set \(\Theta(S)\): the sweep

**Deliverable.** Grid over life × utilization × residual (optionally purchase
band, PUE). CSV/JSON of cells marked consistent with \(S\) within
\(\varepsilon\). Static chart of an iso-OCPI slice with the accounting point
overlaid.

**Done when.** Sweep is a pure function `params × S → surface` in `project`,
cancellable, cache key = snapshot id + code version + grid spec. A tiny-grid
test is enough. No job queue.

**Not this gate.** Distributed jobs, live streaming of dots, claiming a unique
implied residual.

### Learn before Gate 4

| Item | Why | Check you understand |
|---|---|---|
| Ill-posed inverse / level set (Wikipedia is enough) | Many \(\theta\) fit one \(S\). That is the result. | You will not report “the” implied residual. |
| Why we sweep instead of estimate | ~90 OCPI points cannot identify half-life or \(u\). | Name two parameters that look estimable and are not. |
| `rayon` (optional, 20 min) | Embarrassingly parallel map. Not in `domain`. | Parallelism lives in `project`, not in the identity. |

**Skip:** Sobol / Morris (optional module after Gate 4).

### During Gate 4

Level sets; cancellable batch work; cache keys that include code version.

---

## Gate 5 — Quality-adjusted units (hedonic as conversion)

**Deliverable.** Same surface in `/H100e-hour`, `/PFLOP-hour`, `/GB-hour`.
Cross-GPU table: OCPI ratio vs FLOPS vs memory vs TDP. Leftover = scarcity,
not a regression.

**Done when.** Conversion factors live in `domain`, sourced from Epoch,
versioned with the log. No OLS on five points. H100 → H100e is identity.

### Learn before Gate 5

| Item | Why | Check you understand |
|---|---|---|
| Epoch’s H100-equivalent definition | Our conversion factor, not a new index. | Peak 8-bit ≠ delivered training. |
| Bandi & Su §4.1 contract-sizing paragraph (Treasury conversion-factor analogy) | Why a common unit exists at all. | An H100 future and a B200 future cannot net without a map. |
| Why \(N=5\) cannot identify a hedonic model | Temptation to OLS the five free GPUs. | Degrees of freedom: five points, many attributes. |
| Buckingham-style ratios (one page) | Ratios of like quantities, not a fitted hedonic. | OCPI_H200 / OCPI_H100 vs FLOPS_H200 / FLOPS_H100. |

### During Gate 5

Conversion as a typed function; leftover as a finding, not a residual to
absorb into a regression.

---

## Gate 6 — Bitemporal as-of

**Deliverable.** `cli invert --as-of 2026-06-15` uses the daily settle *known
for that date*. Hourly current and daily settle are distinct series.
`fetched_at` ≠ `valid_on`.

**Done when.** A fixture with two fetches on different days returns different
\(S\) for different `--as-of`. `latest` is an explicit alias.

### Learn before Gate 6

| Item | Why | Check you understand |
|---|---|---|
| Valid time vs transaction time (bitemporal data, one explainer) | Overwriting “the” price destroys the research. | `valid_on` is the market date; `fetched_at` is when we learned it. |
| As-of join (the idea, not a SQL dialect) | `--as-of` is a query, not a file overwrite. | “What did we know on date D about date D?” vs “what do we know now about date D?” |

### During Gate 6

Bitemporal fields that already existed as names become a query. Fixture
discipline.

---

## Gate 7 — Live marks (still no SPA)

**Deliverable.** After each successful hourly fetch, recompute \(\Theta\) for
the five GPUs and append a mark. `cli serve` exposes `GET /surface?gpu=...`
and an SSE endpoint. `curl` is enough.

**Done when.** Clock is server-side. A mark can be replayed from the log after
restart.

### Learn before Gate 7

| Item | Why | Check you understand |
|---|---|---|
| `tokio` tasks (the chapter on spawning, not the whole runtime book) | Collector and server share a process. | The client never advances time. |
| `axum` “hello world” + one JSON route | Tiny API, not a framework tour. | Handler is a projection reader. |
| What SSE is (one page) | Marks are a stream of events. | SSE is server → client. It is not a clock. |
| Idempotent hourly fetch | Re-running the same hour must not double-count. | Same `fetched_at` hour → same log position or an explicit duplicate rule. |

### During Gate 7

Server-authoritative clock; backpressure only if invert is slower than one
hour (it will not be — do not overbuild).

---

## Gate 8 — Venue basis panel

**Deliverable.** For H100 (and whatever else is public): OCPI vs Lambda list
vs RunPod list vs AWS on-demand per-GPU-hour vs AWS 3y reserved implied.
Separate series, never averaged. `basis = venue - OCPI`, with the caveat that
OCPI is trades and lists are offers.

**Done when.** AWS Price List extract works **or** is explicitly cut with a
recorded reason. HTML list prices cached raw. Table exists in CLI/JSON.

**Not this gate.** Trading the wedge; Silicon Data.

### Learn before Gate 8

| Item | Why | Check you understand |
|---|---|---|
| Bandi & Su §5.2, equation (3) — synthetic futures from adjacent term rentals | Implement the formula, even on three points. | Synthetic \(F\) is a difference of adjacent term rates, not a scraped “forward.” |
| Offer vs trade | OCPI is cleared trades; Lambda/RunPod pages are offers. | You will never average them into one “price.” |
| Target site `robots.txt` + ToS, 15 min per source | Scraping ethics is part of the gate. | You know whether we may fetch, and we cache raw either way. |
| AWS Price List API overview, timeboxed to one hour | Seed-plan cut line 4: if SKU filtering wins, cut it and write why. | You either have a p5 on-demand / 1y / 3y triple or a written cut. |

### During Gate 8

Adapter pattern for ugly sources; schema drift; limits to arbitrage as a
table, not a trade.

---

## Gate 9 — Thin client

**Deliverable.** React + TS page: cost-stack bars, \(\Theta\) heatmap with
accounting overlay, live OCPI mark, venue-basis table. Reads the API. Does
not advance time. Does not invert in the browser.

**Done when.** A stranger can understand “implied residual vs 6-year books”
from the screen without a README paragraph.

### Learn before Gate 9

| Item | Why | Check you understand |
|---|---|---|
| React + Vite “fetch JSON and render a table” | Client is dumb. | No identity code in TypeScript. |
| EventSource / SSE on the client (one page) | Live marks already exist on the server. | Page renders marks; it does not produce them. |
| The four panels’ *sentence* each | If you cannot say it, the UI will not. | (1) stack (2) surface (3) live S (4) offer vs index. |

### During Gate 9

API design for projections; keeping the client a renderer.

---

## Gate 10 — Robustness write-up

**Deliverable.** Findings doc: surface vs accounting point; energy share of
OCPI; cross-GPU ratio leftovers; venue basis; which conclusions survive the
sweep and which flip; what paid data would sharpen; how to replay.

**Done when.** Limitations are stated in our terms (bands not MSRPs; residuals
not a tape; OCPI already aggregated; utilization unobserved).

### Learn before Gate 10

| Item | Why | Check you understand |
|---|---|---|
| Elasticity \(\partial R / \partial S\) as a finite difference on the grid | Sensitivity without a new model. | Which conclusions move when \(S\) moves 10%. |
| How to write a limitation (seed plan §10, restated in our terms) | Overclaim is the failure mode. | You can list what paid data would change vs what it would not. |

Optional: Morris screening on the grid. Not required to close the gate.

### During Gate 10

Writing a result that does not overclaim. Replay instructions that someone
else can follow.

---

## Optional later modules (only after Gate 4)

| Module | Unlocks | Learn before that module |
|---|---|---|
| Used-market residual **anchors** | Dots on \(\Theta\), not a fitted curve | Treat listings as constraints, not a tape |
| OTPI convertibility | Sequel question, different good | Why \$/MTok is not a GPU-hour |
| SQLite/Postgres projections | Faster as-of | `sqlx`, migrations |
| Sobol indices | Which params matter | Global sensitivity, one tutorial |
| Seller simulator (old R1–R4) | Consumes this cost stack | **Then** Talluri ch. 2–3, Littlewood, network RM §3 (bid-price is a heuristic) |
| Synthetic futures vs OCPI series | Bandi wedge empirically | Bandi §5.2 already required at Gate 8; now the time series |

Do not start the seller simulator because it is in `docs/project-plan.md`.
Start it if Gate 10 says the cost stack is real enough to decide on.

---

## Data used, by gate

| Source | First gate | Role |
|---|---|---|
| Ornn OCPI free (5 GPUs, daily + hourly) | 0–1 | \(S\) |
| Epoch ML hardware CSV | 1 | Specs, TDP, conversion, launch price if present |
| CoreWeave / hyperscaler 10-K useful life | 3 | Accounting scenario (manual extract) |
| PJM (or documented LMP snapshot) | 3 | Energy |
| AWS Price List + optional Spot | 8 | Term structure, venue basis |
| Lambda / RunPod / Vast public prices | 8 | Offer vs index |
| OTPI | optional | Not MVP |

---

## Evolution rule

When something new appears (a source, a chart, a simulator):

1. Does it change \(F\) or \(\Theta\)? If yes, it lands in `domain` + tests,
   then a projection.
2. Is it another dirty feed? It lands in `ingest` as an adapter that only
   emits log events.
3. Is it a view? It lands in `project` or the client. It may not reimplement
   the identity.
4. If it cannot be replayed from the log, it is not done.

The plan is allowed to grow at the tail. It is not allowed to rewrite
Gates 1–2 after they have tests.

---

## Next slice

1. Finish **Learn before Gate 0** (Rust toolchain + one manual `curl` + the
   three short readings).
2. Implement the Gate 0 collector. No TDD plan file required.
3. Finish **Learn before Gate 1**, then write
   `docs/superpowers/plans/YYYY-MM-DD-gate-1-invert.md` and build until
   `invert` works on frozen fixtures.

That is the MVP. Do not `/design` or `/execute-plan` the rest of the DAG.
