# Pricing Perishable Capacity — Project Plan

**A reservation-pricing study of compute markets, delivered as a working full-stack simulator.**

Duration: 7 days · Self-directed · Compute markets as live case, mechanics as the subject

---

## 0. Framing

**Research question.** An operator holds capacity that expires. A bid arrives for part of it. Accepting is irrevocable and better bids may still come. What is the lowest price they should accept — and how does that number move once the risk of holding capacity can be laid off in a financial market?

**Thesis.** The reservation price for perishable capacity is not a property of the capacity. It is a property of the seller's ability to hedge holding it. The spread between the unhedged and hedged floor is the economic value of the financial layer, and it is measurable in simulation.

**Why compute.** Non-storable, indivisible, fragmentable in both time and space, imperfectly fungible but actively being indexed, and still mostly bilateral. No existing playbook — airline revenue management, power markets, charter shipping — covers that combination.

**Grounding case.** Ornn, chosen because its mechanism is unusually public and it spans physical and financial sides at once: an agency capacity venue, a cleared-trade index (OCPI) on the Bloomberg Terminal, and announced cash-settled GPU futures with ICE. Nothing in this project uses non-public information or characterizes any venue's internal practice; mechanism details are parameters.

**Success condition.** Two artifacts. A defensible empirical finding about how compute spot prices behave, and a running system where a person can make the accept/reject decision themselves and see what it cost them.

**Standing constraint: free, publicly available data only.** No paid feeds, no purchased history, no private venue data. This is a scope condition, not an apology — it shapes the method (sweep parameters rather than estimate them) and it means anyone can reproduce the result. Section 5 lists every source and what each is used for.

**Prior work.** Bandi & Su, *(Early) AI Compute Asset Pricing* (arXiv 2607.12156, July 2026) establishes the theoretical spine: compute is non-storable, so storage-based no-arbitrage fails and compute sits closer to electricity than to oil; term rentals can be converted into synthetic futures that upper-bound true futures prices; and futures should carry a risk premium reflecting provider hedging pressure. This project does not re-derive any of that. It cites it and asks the complementary operator-side question the paper leaves open.

---

## 1. Deliverables

| # | Deliverable | Evidence of |
|---|---|---|
| D1 | Capacity ledger with enforced invariants | Correctness under concurrency |
| D2 | Seeded, replayable event log | Reproducibility, systems design |
| D3 | Reservation-price engine, 4 regimes | Domain modeling |
| D4 | Price-process study: estimator validated on power data, applied to OCPI | Research capability, statistical honesty |
| D5 | Occupancy grid with bid shadow | Visualization judgment |
| D6 | Human-in-the-loop review mode | Product thinking, real-time architecture |
| D7 | Hedge blotter marked to live index | Financial modeling, external integration |
| D8 | Parameter sweep as background jobs | Async architecture |
| D9 | Write-up with frontier + basis sensitivity | Synthesis |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────┐
│  Web client (React + TS)                            │
│  Occupancy grid · Bid queue · Blotter · Sweeps      │
└────────────┬───────────────────────┬────────────────┘
             │ REST                  │ SSE (clock, bids, marks)
┌────────────▼───────────────────────▼────────────────┐
│  API layer                                          │
│  Command handlers · Query projections · Job control │
└────────────┬───────────────────────┬────────────────┘
             │                       │
┌────────────▼──────────┐  ┌─────────▼────────────────┐
│  Simulation core      │  │  Job runner              │
│  Clock · Bid gen      │  │  Sweeps, cancellable,    │
│  Ledger + invariants  │  │  streamed partials       │
│  Pricing engine       │  └──────────────────────────┘
└────────────┬──────────┘
             │
┌────────────▼────────────────────────────────────────┐
│  Postgres — event log (append-only) + projections   │
└─────────────────────────────────────────────────────┘
             ▲
┌────────────┴────────────────────────────────────────┐
│  Ingest (all free sources, cached to disk, offline   │
│  after first pull)                                   │
│  OCPI daily+hourly · PJM/CAISO LMP · EC2 Spot        │
│  history · AWS Price List (term structure)           │
└─────────────────────────────────────────────────────┘
```

**Stack.** TypeScript end to end, or Python/FastAPI backend with a TS client if the numerics push that way. Postgres for the log and projections. SSE over WebSockets — the stream is one-directional and SSE is materially less to get wrong.

**Key decision: the simulation clock is server-authoritative.** The client renders state and issues commands; it never advances time. This is what makes the human-in-the-loop mode honest and prevents the client from racing or replaying decisions.

---

## 3. Data model

**Event log** (append-only, the source of truth)

```
Event
  id           bigserial
  run_id       uuid
  seq          int          -- unique per run
  sim_time     timestamptz
  type         enum
  payload      jsonb
  UNIQUE (run_id, seq)
```

Event types: `RunSeeded`, `DeploymentListed`, `BidArrived`, `BidAccepted`, `BidRejected`, `BidExpired`, `HedgePlaced`, `MarkApplied`, `WindowExpired`.

A run is fully determined by its seed plus its decision events. Replay reconstructs any state.

**Core entities**

```
Deployment
  id, gpu_type, node_count
  window_start, window_end
  cost_basis_per_gpu_hour      -- parameter, swept
  floor_price, buy_now_price
  divisible: bool              -- venue parameter
  window_granularity_hours     -- venue parameter

Bid
  id, deployment_id
  min_node_count, max_node_count
  start_at, end_at
  price_per_gpu_hour
  arrived_at, expires_at

Allocation
  id, bid_id, deployment_id
  node_count, start_at, end_at
  version                      -- optimistic lock

HedgePosition
  id, run_id, gpu_type
  notional_gpu_hours, entry_price, tenor
  basis_error_bps              -- swept
```

**Ledger invariants** — enforced in the database, not just application code:

1. For every deployment and every instant, allocated nodes ≤ total nodes.
2. No allocation extends outside its deployment's window.
3. Allocations are whole-node when `divisible = false`.
4. `min_node_count ≤ allocated ≤ max_node_count` for each accepted bid.
5. Accepted bids are immutable.

Implement 1 with an exclusion constraint over `(deployment_id, tstzrange)` plus a node-count check, or a serializable transaction with optimistic versioning. Either way, write the test that fires N concurrent overlapping accepts and asserts no overcommit.

---

## 4. The pricing engine

For a pending bid, decompose:

- **Direct cost** — GPU-hours consumed × cost basis
- **Stranding cost** — value of capacity the fill renders unsellable (time gaps below minimum viable term; node remainders below typical minimum bid)
- **Opportunity cost** — expected value of the foreclosed capacity

Four regimes, run on identical seeded bid streams:

| Regime | Accept rule |
|---|---|
| R1 Floor-only | price ≥ published floor |
| R2 Price-greedy | highest $/GPU-hour that fits |
| R3 Inventory-aware, unhedged | price ≥ direct + stranding + *forecast* opportunity |
| R4 Inventory-aware, hedged | price ≥ direct + stranding + *curve-implied* opportunity, degraded by basis error |

**Headline number:** R4 floor minus R3 floor, as a function of basis error. This locates the tracking error at which the financial layer stops paying for itself.

**Basis error is measured, not guessed.** Two free empirical anchors: cross-zone dispersion in EC2 Spot for identical instance types, and cross-provider dispersion in public price pages for identical GPUs. Sweep around the observed range rather than an invented one.

**Fill recommendation** is a search over the min/max node range, scoring each candidate fill by net contribution.

---

## 5. Data and the price-process study (the research half)

### 5.1 What's free, and what each source is for

| Source | Access | Used for |
|---|---|---|
| **Ornn OCPI free tier** (`api.ornnai.com`) | No key. 3 months daily, 5 GPUs: H100 SXM, H200, B200, A100 SXM4, RTX 5090. Current price updates hourly. | Primary compute price series. Level, volatility, cross-GPU correlation. |
| **PJM Data Miner 2 / CAISO OASIS** | Free, registration only. Years of hourly LMP. | **Estimator validation.** Known mean-reverting-with-spikes reference. |
| **AWS EC2 Spot price history** | Free with an AWS account. 3 months, every GPU instance type × availability zone. | Wide cross-section of a real non-storable compute price. Cross-zone dispersion = empirical basis proxy. |
| **AWS Price List API** | Free, public, machine-readable. On-demand + 1yr + 3yr reserved rates. | **Real term structure.** Synthetic forward curve via Bandi & Su's construction. |
| **CoreWeave SEC filings** (EDGAR) | Free. | Cost-basis range: depreciation schedules, contract structure, utilization commentary. |
| **Public provider price pages** (Vast.ai, RunPod, Lambda) | Free. | Cross-provider dispersion for identical hardware — second basis measurement. |
| **Epoch AI GPU dataset** | Open. | H100-equivalent conversion factors for cross-generation comparability. |
| **Bandi & Su Table 1** | Open access paper. | External anchor: Ornn series start dates and end-of-sample values, to check whether the 3-month window looks representative. |

Everything is pulled once and cached to disk. The simulator runs offline after the first ingest.

*Deliberately excluded: Ornn's paid historical tier, Silicon Data term curves, and any private venue data. Where these would sharpen a result, the write-up says so and names what they'd resolve.*

### 5.2 The problem with three months, stated plainly

Roughly 90 daily observations per GPU. Enough for level and volatility. **Not** enough to identify a mean-reversion half-life longer than a few weeks — and the available window is a bad one, since H100 and B200 turned sharply upward in late 2025 into 2026 on agentic-inference demand. A naive fit will read that trend as "no reversion" for the wrong reason.

Three responses, all free:

**Validate the estimator elsewhere first.** Fit the GBM / Ornstein-Uhlenbeck / Lucia–Schwartz suite to PJM hourly LMP, where the answer is known. That tests the code, not the market. Only then apply it to compute, and report confidence intervals honestly.

**Widen the panel.** Pool the five OCPI series with a shared reversion parameter rather than fitting each in isolation. Add EC2 Spot's much larger cross-section as a second, independent panel.

**Accumulate your own history.** The current-price endpoint updates hourly. Start a cron polling it on day one of reading, and by the end of the build you have several hundred hourly observations of your own — useful for short-horizon dynamics even if not for long-run reversion.

### 5.3 The method: sweep, don't estimate

**The half-life is a swept parameter, not a fitted constant.** Run the simulator across a plausible range (say 3 days to 6 months) and report which conclusions hold across the whole range and which flip.

- If the R4−R3 floor spread is positive throughout, that's a robust finding independent of data you can't get.
- If it flips inside the range, you've located the parameter that matters and named exactly what data would resolve it.

This converts the data limitation from a weakness into a stated scope condition, and it is better practice than a point estimate from 90 observations would have been regardless of budget.

### 5.4 The forward curve, built not invented

AWS publishes on-demand, 1-year, and 3-year reserved rates. Differences between adjacent term rentals give implied forward prices for the intervals between them — the Bandi & Su synthetic-futures construction, run on free data.

It's coarse: three tenor points, one provider, hyperscaler rather than neocloud. Say so. But "coarse and real, limitations stated" is a stronger foundation for the hedged regime than a curve simulated from the fitted spot process, and the paper's result that synthetic futures upper-bound true futures gives you a principled adjustment to apply.

Keep the simulated curve as a fallback if the Price List extraction proves fiddly.

---

## 6. Frontend

**Occupancy grid.** Nodes × time. Accepted allocations are rectangles. On hovering a pending bid, overlay its shadow in three shades: consumed, stranded, foreclosed. SVG until it doesn't scale, then Canvas. This is the load-bearing view — build it first and build it well.

**Bid queue.** Pending bids with price, term, node range, and the recommended fill. Each row shows the four regime thresholds side by side so the disagreement between them is the visible thing.

**Review mode.** Clock runs, bids stream in via SSE, user accepts (choosing a fill) or rejects. Pause and speed control. At season end, the user's decisions are scored against R1–R4 on the same stream, and their result is plotted as a point on the margin/volume frontier.

**Blotter.** Open hedge positions, entry, current mark against real OCPI, unrealized P&L, basis drift. Sits beside the inventory view — physical and financial on one screen.

**Sweeps panel.** Launch a parameter sweep (cost basis × basis error × arrival intensity), watch results stream in as points on the frontier, cancel mid-run.

**Scrub bar.** Timeline over the event log. Scrub to any decision, fork, run a different regime from that state, diff outcomes. *Cut first if behind.*

---

## 7. Day-by-day

**Day 0 — Start the collector (do this on day one of *reading*, not building).** A cron job polling the OCPI hourly current-price endpoint into a flat file. Costs twenty minutes and every day of delay is data you don't get back.

**Day 1 — Foundation.** Repo, Postgres, migrations. Event log with seeded RNG and replay. Deployment and bid entities. Ledger with invariants 1–5 and the concurrency test. Headless.
*Gate: replay a run twice from the same seed, get byte-identical projections.*

**Day 2 — Data + engine.** Ingest and cache all free sources. Fit the model suite to PJM LMP first (estimator validation), then to the pooled OCPI panel. Write the price-process finding. Extract the AWS term structure and build the synthetic forward curve. Pricing engine with all four regimes; bid generator driven by the fitted process.
*Gate: the estimator recovers known behaviour on PJM, and R1–R4 produce different, explicable outcomes on the same stream.*

**Day 3 — The grid.** React app, REST reads, static occupancy grid over a completed run. Bid queue with regime thresholds. Bid shadow on hover.
*Gate: fragmentation is visible without explanation.*

**Day 4 — Live.** Server clock, SSE stream, command endpoints. Review mode with accept/reject/fill, pause, speed. End-of-season scoring against all regimes.
*Gate: a stranger can play a season unaided and understand their score.*

**Day 5 — Financial layer.** Hedge placement against the AWS-derived forward curve, daily marks against real OCPI. Blotter UI. Basis error injected at the empirically observed range.
*Gate: the R4−R3 spread is computed and displayed.*

**Day 6 — Sweeps + polish.** Job runner with cancellation and streamed partials. Sweep over half-life × basis error × cost basis. Frontier plot and robustness chart showing which conclusions survive the full parameter range. Scrub bar if time remains.
*Gate: frontier renders progressively, cancel works mid-sweep, robustness chart answers "does this hold everywhere?"*

**Day 7 — Write-up.** README with architecture and how to run. Findings document: price-process result with the PJM validation alongside it, frontier, basis sensitivity, the robustness sweep, where airline RM breaks on indivisibility, open questions, and a short section naming which conclusions paid data would sharpen. Screenshots or a short recording.

---

## 8. Cut lines, in order

Cut from the bottom up when behind. Do not cut upward.

1. Scrub bar and timeline forking
2. Concurrency demo panel (keep the test, drop the UI)
3. Sweeps as *background jobs* — fall back to precomputed results, but keep the sweep itself
4. AWS-derived forward curve — fall back to a curve simulated from the fitted spot process
5. Blotter live marks — fall back to a static basis sensitivity chart
6. **Floor:** ledger + invariants, price-process finding, half-life robustness sweep, occupancy grid, review mode, R1–R4 comparison

**Never cut:** the invariant tests, the price-process write-up, the half-life sweep, the grid. The sweep is non-negotiable on free data — without it, every conclusion rests on 90 observations and a hopeful point estimate.

---

## 9. Open questions to answer in the write-up

- Does compute spot reject GBM in favor of mean reversion with jumps, as non-storability predicts — and can 90 observations even tell?
- Is the hedged floor meaningfully below the unhedged floor at *observed* basis levels, or does tracking error eat the benefit?
- Which conclusions survive the full half-life sweep, and which flip? The flipping ones are the real contribution: they name what data would settle the question.
- How large is stranding cost, and does a liquid secondary market (transfer/sublet) collapse it to near zero? Model both.
- Where does EMSR-style nesting break on indivisibility?
- At what volume does price-ranking stop being adequate? A null result names that threshold.

---

## 10. Limitations to state upfront

**Free data only, by choice.** Three months of OCPI history at daily frequency; the paid tier extends to January 2024 and is not used. This limits what can be identified about long-run dynamics, which is why the half-life is swept rather than fitted.

**The sample window is unrepresentative.** It sits inside a documented demand-driven upswing. Any trend estimate from it should be read as conditional on that regime.

**The forward curve is coarse.** Three tenor points from one hyperscaler, not a neocloud term curve. Silicon Data publishes richer term curves; they are not used here.

**Cost basis is a swept parameter**, anchored to public filings rather than observed directly.

**Bid arrivals are synthetic.** Only the price process touches real data.

**No optimality claim.** This compares heuristics against an NP-hard problem.

**Opportunity cost is estimated**, as it would be in any real deployment.

The write-up should close by naming which specific conclusions paid data would sharpen — that's more useful to a reader than pretending the constraint didn't exist.
