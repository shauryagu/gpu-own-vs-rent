# Open questions through PR 5

Tracks questions that still matter after PRs 1–5: workspace scaffold, hourly collector, domain newtypes, NPV identity, and daily-index / Epoch ingest.

The Gate 0+1 design ([`docs/designs/2026-08-22-gate-0-1-mvp.md`](designs/2026-08-22-gate-0-1-mvp.md)) marks its five numbered Open Questions **Resolved** or **Resolved: deferred**. Those resolutions are binding. Invert CLI (PR 6) still has to print leftover \(L\) and salvage \(R^{\star}\) under distinct names.

Do not invent \(P\), \(r\), primary \(T\), or primary \(u\). Do not collapse \(L\) and \(R^{\star}\). Do not add a paid Ornn key.

**Code after PR 5.** `collect --series current|daily|epoch` all write raw bodies. Invert still `bail!("not implemented")`. `domain` has identity algebra. Ingest emits `ObservedSpot` from `fixtures/ocpi/daily-index/{slug}.json`. Epoch release price is an annotation only.

Companion map of what the code does: [`docs/status.md`](status.md).

## Status legend

| Status | Meaning |
|---|---|
| **open** | Live operational or implementation gap. |
| **partially addressed** | Design and/or code exist; named work remains. |
| **deferred** | Deliberately unanswered. Required flags or a later gate; no silent default. |
| **closed-for-now** | Decision is binding. Later PRs implement it. Do not reopen without a design change. |

## Contents

1. [Product / theory](#1-product--theory)
2. [Data / Ornn](#2-data--ornn)
3. [Types / serde](#3-types--serde)
4. [Collector operations](#4-collector-operations)
5. [Parameters we refuse to invent](#5-parameters-we-refuse-to-invent)
6. [Explicitly later](#6-explicitly-later)

---

## 1. Product / theory

### 1.1 Two inverses: leftover \(L\) vs NPV salvage \(R^{\star}\) (OQ 1)

- **Status:** closed-for-now (decision). Algebra is in `domain`. CLI not built.
- **Why it is open.** They have opposite signs in \(S\) and different units. Collapsing them under “implied residual” fails Gate 1 or lies about the algebra (design OQ 1; Key Decision 6; Alternatives #10).
- **What PRs 1–5 did.** PR 4 implements `leftover` (`UsdPerGpuHour`) and `implied_salvage` (`Usd`) with proptest P1a/P1b. PR 5 does not re-derive them.
- **Still to track.** Distinct JSON keys on invert stdout: `leftover_usd_per_gpu_hour` / `implied_salvage_usd`. Gate 4 defines \(\Theta_L(S)\) and \(\Theta_{R^{\star}}(S)\) separately.
- **Must answer by.** PR 6 (CLI never prints one name for both). Before publishing any number.

### 1.2 What “fair deal” means

- **Status:** closed-for-now.
- **Why it is open.** Fair means discrete NPV of buy-and-earn-rent is zero, not an optimum and not “minimize basis risk until buy wins.” Many \(\theta\) fit one \(S\) (`docs/positioning.md`; design identity section).
- **What PRs 1–5 did.** PR 4 has `Theta`, `fair_rent`, leftover, implied salvage. Invert CLI still `bail!("not implemented")`.
- **Still to track.** Accounting overlay is a labeled scenario \(T=6\), \(R=0\), not a hidden default. Forward \(F(\theta)\) prints only with `--residual-cents`.
- **Must answer by.** PR 6 (CLI contract). Before publishing any number.

### 1.3 Bandi & Su: cite, do not re-derive

- **Status:** closed-for-now.
- **Why it is open.** Cash-and-carry from spot to futures fails: a GPU-hour unused today cannot be delivered next month (`docs/source/AiComputeAssetPricing.md` §5.1; arXiv:2607.12156). This repo asks about the durable chip, not the perishable service flow. Re-deriving that result would change the stated positioning.
- **What PRs 1–3 did.** Nothing in code. Positioning and the Gate 0+1 design already cite the paper.
- **Still to track.** Do not treat \(F(\theta)\) as a stand-in for their \(F_t(T)\). Do not build a synthetic-futures strip here.
- **Must answer by.** PR 4 comments and PR 6 copy (cite, do not re-derive). Never before first invert as a new theory claim.

### 1.4 Calculator, not an exchange or competing index

- **Status:** closed-for-now.
- **Why it is open.** We consume OCPI. We do not publish a benchmark, operate a CLOB, or price residual-value swaps (`docs/positioning.md`).
- **What PRs 1–3 did.** Collector is a local pull of the free current endpoint. No server, no published series.
- **Still to track.** List prices (Lambda, RunPod, AWS) stay out until Gate 8, and then in a separate panel. Repo GitHub name is `gpu-own-vs-rent`; binary stays `chi`.
- **Must answer by.** Every public-facing string (PR 6, README). Gate 8 if venue language returns.

### 1.5 Bid-price / R1–R4 stays out

- **Status:** closed-for-now.
- **Why it is open.** Bid-price schemes are a near-optimal heuristic for network/bundle requests, not globally optimal (`AGENTS.md`; network RM paper §3). The seed 7-day reservation simulator (`docs/project-plan.md`) is a different product.
- **What PRs 1–3 did.** Nothing. No occupancy grid, no ledger, no R1–R4.
- **Still to track.** Do not pull Talluri / Littlewood / R1–R4 into Gate 1. Optional seller simulator only after Gate 4, and only if Gate 10 says the cost stack is real enough.
- **Must answer by.** Not a Gate 1 question. Optional module after Gate 4 / Gate 10.

---

## 2. Data / Ornn

### 2.1 Three series: current vs daily-index vs history

- **Status:** closed-for-now (parsers). PR 6 must still refuse the wrong file.
- **Why it is open.** Teaching fixture (2026-08-21): daily-index `2.879583333333333`, history last `2.88`. Live 2026-08-25: daily-index `2.9679166666666665`, history last `2.97`. Mixing them is a bug (Key Decision 3; Alternatives #11).
- **What PRs 1–5 did.** PR 5 parses daily-index (invert \(S\)), daily-history (refuse as \(S\)), and caches daily-index-all. Extra fields do not fail parse. Invert fixture keeps the teaching token; live collect fixture is today’s all-GPU capture.
- **Still to track.** PR 6 must name `ocpi.daily-index` and refuse a current-only fixture dir.
- **Must answer by.** PR 6.

### 2.2 Invert \(S\) is daily-index only

- **Status:** closed-for-now (decision). Parser built. Invert CLI not built.
- **Why it is open.** Invert \(S\) is the per-GPU latest daily-index, not history’s 2-decimal last point, not `/api/daily-index/all`, not hourly current (Key Decisions 3, 17).
- **What PRs 1–5 did.** Wrapper parser at `parse_daily_index_wrapper`. Path is `{fixture_dir}/ocpi/daily-index/{slug}.json`. Inner-only JSON is `Err`. Token `2.879583333333333` survives parse.
- **Still to track.** PR 6 opens only that path.
- **Must answer by.** PR 6 (invert opens only that path).

### 2.3 Free-tier GPU list

- **Status:** partially addressed.
- **Why it is open.** Ornn docs: fetch `/api/gpu-types-free` rather than hard-coding the five names. An empty list is not a successful no-op.
- **What PRs 1–3 did.** PR 2 fetches the list, parses names, fails on empty (`IngestError::EmptyGpuList`). Fixture lists A100 SXM4, B200, H100 SXM, H200, RTX 5090.
- **Still to track.** A 401 on a “free” path is an allow-list bug, not a prompt for a key. Unmapped GPU + energy is `Err` (Epoch map). First live `chi collect` should confirm the live list still matches fixtures.
- **Must answer by.** First live `chi collect` (confirm the live list still matches fixtures).

### 2.4 A100 SXM4 fail-closed (OQ 5)

- **Status:** closed-for-now (decision). Epoch adapter fails closed.
- **Why it is open.** Epoch has 40 GB vs 80 GB A100 rows. Guessing is forbidden (design OQ 5; Key Decision 14). RTX 5090 TDP is not invented.
- **What PRs 1–5 did.** `row_for_gpu(A100Sxm4 | Rtx5090, energy)` is `Err`. Invert mapping is H100 SXM → `NVIDIA H100 SXM5 80GB` only. H200 / B200 may parse.
- **Still to track.** PR 6 must not guess a TDP for unmapped SKUs.
- **Must answer by.** Before first invert of any SKU other than H100 SXM.

### 2.5 Fixtures vs live captures

- **Status:** closed-for-now (PR 5). Invert still must not read `data/`.
- **Why it is open.** Invert is fixture-only in this MVP (`--fixture-dir`). Promoting a live capture is wrap + commit, not “read `data/`” (Key Decision 17).
- **What PRs 1–5 did.** Invert wrapper `fixtures/ocpi/daily-index/H100_SXM.json` keeps the teaching close. `daily-index-all.json` is a live 2026-08-25 collect fixture. History teaching last point `2.88`. Epoch excerpt committed. `collect --series daily|epoch` writes raw envelopes under `data/raw/`.
- **Still to track.** PR 6 golden stdout produced by the binary, not hand-copied from the design sample. Invert never opens `data/` or `--data-dir`.
- **Must answer by.** PR 6.

### 2.6 No paid Ornn key

- **Status:** closed-for-now.
- **Why it is open.** Free public data only (`docs/llms.txt`; design Security). Excluded: paid Ornn history, Silicon Data, private venue tape.
- **What PRs 1–3 did.** `LiveHttp` User-Agent `chi-collector/0.1 (research; OCPI current; no key)`. No key env var, no key flag, no `Authorization` header.
- **Still to track.** Any new host reviewed against `docs/llms.txt`. OTPI endpoints on the same host are not called.
- **Must answer by.** Every ingest PR. Before launchd is trusted (a key must not appear in the plist).

---

## 3. Types / serde

### 3.1 Display scale vs source token vs invert `round_dp(12)`

- **Status:** partially addressed.
- **Why it is open.** Three layers must stay distinct. Domain money serde is the printed quantity (USD scale 2, USD/GPU-hour scale 4), not a lossless OCPI token. Ingest stores `index_value` as source token text. Invert JSON owns `round_dp(12)` on computed fields; `s_usd_per_gpu_hour` stays the source token.
- **What PRs 1–3 did.** PR 2: `parse_index_value` uses `serde_json` `arbitrary_precision` + `Decimal::from_str_exact`; JSONL stores the token; test keeps `2.879583333333333`. PR 3: `Usd` / `UsdPerGpuHour` serialize as `"25.00"` / `"2.8796"`; a JSON number is a schema error; test documents that domain serde is not the ingest wire.
- **Still to track.** Invert must not serialize \(S\) through domain `Serialize`. Computed JSON fields: `round_dp(12)`, never more than 28 significant digits. Declared cents stay scale 2.
- **Must answer by.** PR 6 invert JSON. Do not “fix” domain serde to be lossless in PR 4.

### 3.2 `FetchedAt` vs `ValidOn` vs `AsOf`

- **Status:** partially addressed.
- **Why it is open.** Transaction time, settlement date, and query time are different clocks (`docs/execution-plan.md` Gate 6; design Clock section).
- **What PRs 1–5 did.** Daily-index `valid_on` is the UTC date of `body.data.date` (`2026-08-21T20:00:00.000Z` → `2026-08-21`). Wrapper `fetched_at` is on `DailyIndexRecord`, not on `ObservedSpot`. Parser never uses `mtime` or `now()`. `AsOf` unused.
- **Still to track.** PR 6 prints both clocks. `--as-of` is Gate 6.
- **Must answer by.** PR 6 (print both). Gate 6 (query).

### 3.3 Domain has no I/O; money is `Decimal`; `f64` is physics

- **Status:** partially addressed.
- **Why it is open.** Illegal unit joins must be unrepresentable. `domain` depends on nothing else in this repo.
- **What PRs 1–3 did.** Allowed deps only. Legal `Mul`/`Div` only. rustdoc `compile_fail` for the listed illegal joins. Physics constructors reject non-finite `f64`. `Utilization` is \((0,1]\); `Years` rejects 0.
- **Still to track.** `energy_per_gpu_hour` is the only `f64 → Decimal` conversion (`from_f64_retain`, `Err` on NaN/Inf). Reject \(u=0\), \(T=0\), \(A=0\). Do not parse current prices into `UsdPerGpuHour` (already refused in PR 2).
- **Must answer by.** PR 4 (`energy.rs`, identity rejects).

---

## 4. Collector operations

### 4.1 Missed hours are not invented

- **Status:** open (ops). Code will not backfill.
- **Why it is open.** Hourly current cannot be reconstructed from the free 3-month daily window (Gate 0). A missed hour is a permanent hole.
- **What PRs 1–3 did.** One transport retry, then fail that GPU. Non-200 does not append JSONL. Attempt remaining GPUs; exit non-zero if any failed.
- **Still to track.** No “hour \(N\) is missing” detector. launchd does not retry a failed :05 run. Do not interpolate from daily-index into `ocpi.current`.
- **Must answer by.** First live `chi collect` + launchd (start the series). Gate 2 if a hole report becomes a log event. Before the hourly file is trusted as a complete panel.

### 4.2 Collect lock

- **Status:** partially addressed.
- **Why it is open.** An overlapping job must not tear JSONL.
- **What PRs 1–3 did.** `RawCache::try_lock` on `data/collect.lock` via `File::try_lock`; `WouldBlock` → `IngestError::AlreadyRunning`. Test: overlapping collect fails without writing JSONL.
- **Still to track.** launchd `KeepAlive` stays false. Crash releases the kernel lock (documented; not soak-tested).
- **Must answer by.** Before launchd is trusted.

### 4.3 Atomic raw writes and JSONL fsync

- **Status:** partially addressed.
- **Why it is open.** Raw bodies are the only irrecoverable bytes. Gate 2 will content-address them via `raw_sha256`.
- **What PRs 1–3 did.** Raw write is tmp + `sync_all` + rename. JSONL append is `write_all` + newline + `sync_all`. Nanosecond filenames. Tests: no leftover `.tmp`; second collect appends.
- **Still to track.** `collect --series daily|epoch` reuses this writer. Gate 2 must not change the collector record shape when it switches to content-addressed payloads.
- **Must answer by.** Gate 2 (hash store). Before launchd is trusted.

### 4.4 Name-match on `gpu_name`

- **Status:** partially addressed.
- **Why it is open.** A swapped body must not become an H100 current print.
- **What PRs 1–5 did.** Current: `gpu_name` mismatch → `GpuNameMismatch`. Daily-index: `gpu_type` mismatch → `GpuTypeMismatch`. Raw still kept.
- **Still to track.** Invert `--gpu "H100 SXM"` must open that SKU’s fixture only.
- **Must answer by.** PR 6 (path is a function of `--gpu`).

### 4.5 First live `chi collect` and launchd

- **Status:** open.
- **Why it is open.** Gate 0 is not done until a file grows every hour (`docs/execution-plan.md` Gate 0).
- **What PRs 1–3 did.** `scripts/ocpi-hourly.plist` and `scripts/install-ocpi-hourly.sh` ship. Minute 5, `KeepAlive` false. Collector lockfile and launchd logs are gitignored.
- **Still to track.** One hand run succeeds. `data/raw/` and `data/ocpi-hourly.jsonl` grow. Agent actually fires. Rollback is `launchctl unload`; do not truncate JSONL. Do not uninstall the collector while building Gate 1.
- **Must answer by.** First live `chi collect` + launchd install. Before launchd is trusted. **This is the next operational step, not PR 4.**

### 4.6 Snapshot-pull HTTP, not sequenced market data

- **Status:** closed-for-now (this collector).
- **Why it is open.** Ornn has no sequence number, gap-fill, or tape. Treating JSONL as a complete hourly panel would invent missed hours.
- **What PRs 1–3 did.** Sequential GETs, ~200 ms pause, 15 s timeout, `HttpGet` + `FixtureHttp`. Allow-listed URLs only.
- **Still to track.** Gate 2 events point at content hashes of snapshots. Do not “fix” Gate 0 by adding Tokio (Key Decision 11).
- **Must answer by.** Gate 2 (event shape). Not a reason to backfill.

---

## 5. Parameters we refuse to invent

### 5.1 Purchase \(P\) (OQ 2)

- **Status:** deferred.
- **Why it is open.** Purchase is a declared point inside a band, never an MSRP, never Epoch “Release price” \(33600\) (OQ 2; Alternatives #9).
- **What PRs 1–3 did.** `Usd::from_cents` exists. No CLI flag, no default literal.
- **Still to track.** Required `--purchase-cents`. Band sourced later (quote, filing, used listing as an *anchor*). Epoch release price is annotation only (PR 5).
- **Must answer by.** PR 6 (required flag). Band itself is not required before first invert.

### 5.2 Discount rate \(r\) (OQ 3)

- **Status:** deferred.
- **Why it is open.** No WACC will be invented (OQ 3).
- **What PRs 1–3 did.** `DiscountRate::try_new` rejects negative; zero is allowed (annuity factor becomes \(T\)). No CLI flag.
- **Still to track.** Required `--discount-rate`. \(r=0\) uses \(A=T\) (PR 4 unit test).
- **Must answer by.** PR 4 (algebra). PR 6 (required flag).

### 5.3 Primary \(T\) and \(u\) (OQ 4)

- **Status:** deferred.
- **Why it is open.** Do not silently use \(T=6\) or \(u=1\) as primary \(\theta\). Accounting overlay remains \(T=6\), \(R=0\) as a labeled scenario (OQ 4).
- **What PRs 1–3 did.** `Years` rejects 0. `Utilization` is \((0,1]\). No invert flags.
- **Still to track.** Required `--life-years` and `--utilization`. Accounting block is a second evaluation, not the default \(\theta\).
- **Must answer by.** PR 6 invert CLI. Gate 3 may quote a 10-K useful life as a *label*, not a silent primary.

### 5.4 Residual \(R\) and `--residual-cents`

- **Status:** closed-for-now (decision). No CLI yet.
- **Why it is open.** Residual is never silently 0 except the labeled accounting scenario. Evaluating \(F\) at \(R=R^{\star}(S)\) is the tautology \(F=S\) (Key Decision 7).
- **What PRs 1–3 did.** Nothing.
- **Still to track.** `--residual-cents` required to print \(F(\theta)\). Omitted ⇒ print \(S\), declared \((P,T,u,e,r)\), \(L\), \(R^{\star}\), accounting point; `"forward": null`.
- **Must answer by.** PR 6 invert CLI.

### 5.5 Half-life is swept, never estimated

- **Status:** closed-for-now.
- **Why it is open.** ~90 daily points cannot identify mean reversion in a demand-driven window (`docs/project-plan.md` §5.3; execution-plan non-negotiables). Half-life is not a parameter of this MVP.
- **What PRs 1–5 did.** No half-life type, flag, or estimator. History is parsed so we can refuse it as \(S\), not to fit an OU.
- **Still to track.** A later sweep, if any, is Gate 4+. Never as a point value in invert.
- **Must answer by.** Never as a point value in invert.

### 5.6 Decided constants: \(H=8760\), discrete annual, \(\pi=0\), PUE \(=1.0\)

- **Status:** closed-for-now (decision). Coded in PR 4.
- **Why it is open.** These were decided so Gate 1 is learnable on paper (design Open Questions, last paragraph). They are not silent *money* defaults.
- **What PRs 1–5 did.** `HOURS_PER_YEAR`, discrete annuity, `energy_per_gpu_hour` at \(\pi=0\) (product still evaluated). MVP PUE is `1.0`.
- **Still to track.** Gate 3 sweeps PUE and names an LMP snapshot.
- **Must answer by.** Gate 3 if PUE / LMP change.

---

## 6. Explicitly later

### 6.1 Event log and replay (Gate 2)

- **Status:** deferred (named stub only).
- **Why it is open.** Every published number is `event log ⊕ parameter vector ⊕ code version` (`docs/positioning.md`). The MVP only lays types and the raw cache so Gate 2 is not a rewrite.
- **What PRs 1–3 did.** `crates/chi_log` is an unlinked stub. JSONL carries `raw_sha256`. `chi` does not depend on `chi_log`.
- **Still to track.** `SourceFetched` / `SeriesParsed`, content-addressed payloads, `chi replay` byte-identical twice.
- **Must answer by.** Gate 2. Before publishing any number as a research artifact.

### 6.2 Energy, PUE, LMP (Gate 3)

- **Status:** deferred (types only).
- **Why it is open.** Energy is `kW · h · USD/kWh`, computed not invented. Leftover \(L\) is the cost-stack residual; the stack *panel* is Gate 3.
- **What PRs 1–3 did.** `Kilowatt`, `Hours`, `Pue`, `UsdPerKwh` exist. `Kilowatt * Hours -> f64` exists for the energy function. No `energy.rs`, no PJM client.
- **Still to track.** PR 4 implements `energy_per_gpu_hour`. Gate 3 adds PUE sweep and an LMP snapshot. Do not fetch PJM in PR 4–6.
- **Must answer by.** PR 4 (product at \(\pi=0\)). Gate 3 (real energy panel).

### 6.3 \(\Theta_L(S)\) and \(\Theta_{R^{\star}}(S)\) (Gate 4)

- **Status:** deferred.
- **Why it is open.** The object is a feasible set, not a point estimate. Non-uniqueness is the result (`docs/positioning.md`).
- **What PRs 1–3 did.** `crates/project` is a thin stub. No sweep.
- **Still to track.** Two labeled surfaces, not one residual axis. Sweep is a pure function in `project`.
- **Must answer by.** Gate 4. Do not start in PR 4–6 beyond keeping \(L\) and \(R^{\star}\) separate.

### 6.4 H100e maps beyond identity (Gate 5)

- **Status:** partially addressed (hole only).
- **Why it is open.** Bandi & Su §4.1: contract unit is 1 GPU-hour; conversion-factor analogy. \(N=5\) cannot identify a hedonic model.
- **What PRs 1–3 did.** `H100eHour` + identity map for H100 SXM. Every other `GpuModel` is `NotInMvp`.
- **Still to track.** FLOPS / memory / TDP maps from Epoch, versioned in the log. No OLS on five points.
- **Must answer by.** Gate 5. PR 4 proptest identity round-trip only.

### 6.5 As-of query (Gate 6)

- **Status:** deferred.
- **Why it is open.** `latest` is an explicit alias. Hourly current and daily settle stay distinct series.
- **What PRs 1–3 did.** `AsOf` exists and is unused. Invert has no `--as-of`.
- **Still to track.** `chi invert --as-of YYYY-MM-DD` uses the daily settle *known as of that date*. Two fixture fetches can return different \(S\).
- **Must answer by.** Gate 6. PR 5–6 must not pretend `mtime` is as-of.

### 6.6 SQLite, live marks, venue, client

- **Status:** deferred.

| Later piece | Hook already left | Answer by |
|---|---|---|
| SQLite | JSONL + raw cache; no schema | When as-of queries hurt (Gate 6) |
| Live marks / long-lived process | Hourly collector; no server | Gate 7 (Tokio allowed then) |
| Venue basis / Bandi §5.2 | `HttpGet` + parse adapter | Gate 8 |
| Thin client | None | Gate 9 |
| Used-market anchors, OTPI, Sobol, seller simulator | Explicitly after Gate 4 | Optional modules |

- **Must answer by.** The named gate. Not before first invert.

---

## Answer-by index

| When | Questions that block it |
|---|---|
| First live `chi collect` + launchd | 2.3, 2.6, 4.1, 4.2, 4.5 |
| PR 6 (invert CLI) | 1.1–1.2, 1.4, 2.2, 3.1–3.2, 5.1–5.4 |
| Before publishing any number | 1.1–1.2, 6.1 |
| Gate 2 / 3 / 4 / 5 / 6+ | §6 |

PR 4 and PR 5 are done. PR 6 needs both. Hours lost while invert is unfinished still cannot be reconstructed — install the hourly collector.
