# Reading List — Mapped to the Project Plan

Sorted by whether it blocks a build day or not. The test for "blocking": *if I skip this, will I build the wrong thing and have to redo it?* Everything else reads better alongside the work, when you have a concrete problem to hang it on.

**Pre-read budget: about 2.5 focused days.** The blocking set is genuinely short. Resist expanding it — the alongside list is where scope creep lives.

---

## Part 1 — Read before you start

### 1.1 Framing (half a day) — unblocks Section 0, and stops you claiming a published result

**Bandi & Su, *(Early) AI Compute Asset Pricing*** (arXiv 2607.12156, July 2026)

The one genuinely non-negotiable item. Read Sections 2 and 4 carefully, then 5–6, then skim appendices.

Three things you need from it:
- **§4.1** — why storage-based no-arbitrage fails for compute. This is the theoretical spine of your project and it's already published; cite it, don't re-derive it.
- **§4.2** — the synthetic futures construction from adjacent term rentals. **You will implement this on Day 2** using AWS reserved rates. Read it as a spec.
- **§2.3, Table 1** — Ornn's actual series coverage and end-of-sample values. Your free-tier sanity check.

**Ornn, *The Basis Gap in Compute Acquisition Structures*** — 20 minutes. The vocabulary you'll write in.

**a16z crypto, *Investing in Ornn*** — 10 minutes. The thesis in its cleanest form.

*Deliverable: one page on how your project sits next to Bandi & Su. This becomes your positioning section and is the single highest-value hour of the whole pre-read.*

### 1.2 Reservation pricing (three quarters of a day) — unblocks Section 4 and Day 2

**Talluri & van Ryzin, *The Theory and Practice of Revenue Management*** — chapters 2–3, plus 3.3 on network bid prices.

This is your opportunity-cost term. Skip it and you will rebuild EMSR badly, then discover on Day 5 that your R3 regime was subtly wrong the whole time. Read it as engineering documentation, not theory.

**Littlewood (1972)** — half a page. Two-class marginal seat revenue, the ancestor of everything above. Read it first, actually; it makes Talluri & van Ryzin easier.

*Focus question while reading: where does divisibility get assumed? That assumption is exactly what your whole-node constraint breaks, and finding those spots is a write-up section.*

### 1.3 Price processes (half a day) — unblocks Section 5 and Day 2

**Schwartz (1997), "The Stochastic Behavior of Commodity Prices"** — the one-factor mean-reverting model.

**Lucia & Schwartz (2002)** — mean reversion plus jumps, fit to Nordic power.

Read for the model specifications and the estimation approach. These two are literally what you implement on Day 2 and then validate against PJM.

*Don't read the broader commodity-stochastics literature yet. Two papers is the whole requirement.*

### 1.4 Operational (half a day) — unblocks the ingest layer

Not reading so much as reconnaissance. Do this hands-on, with a terminal open:

- **Ornn API docs** (`dashboard.ornnai.com/docs`) — pull all five series, plot them. **Start the hourly collector cron today** (plan, Day 0).
- **PJM Data Miner 2** — register, pull a year of hourly LMP for one node. This is your estimator validation set.
- **AWS EC2 Spot price history** — `describe-spot-price-history` for GPU instance types. Confirm you can get the cross-section.
- **AWS Price List API** — find on-demand vs 1yr vs 3yr for a p5 instance. **Timebox to one hour.** If the SKU filtering fights you, note it and move on; it's cut line 4.
- **Ornn Compute product pages** — mechanism details: agency structure, floors, buy-now, secondary transfer and sublet.

*Gate before writing application code: every free source pulls successfully and is cached to disk.*

---

## Part 2 — Read alongside the build

Each of these attaches to a specific day. Reading them cold beforehand wastes them; reading them the evening before you need them makes them stick.

**Before Day 2 evening — CoreWeave's latest 10-K or 10-Q** (SEC EDGAR, free). Skim only: depreciation schedules, contract structure, utilization commentary. You need a defensible cost-basis range, not a financial analysis. One hour.

**Before Day 5 — Hull, derivatives textbook, cross-hedging and minimum-variance hedge ratio chapter.** This is the machinery behind basis error and the R4 regime. One chapter, the evening before you build the hedge layer.

**Before Day 5 — Eydeland & Wolyniec, *Energy and Power Risk Management*.** Forward curve construction chapters. Reference, not cover-to-cover — dip in for the specific problem you're facing.

**Before Day 7 — Stopford, *Maritime Economics*, chartering chapters.** Laycan windows, part cargoes, ballast legs. This is your closest institutional analogue for fragmentation, and it's write-up material rather than code material. Instructive precisely because shipping handles this with brokered terms rather than an algorithm.

**Before Day 7 — PJM's LMP and hub overview, plus FERC Order 888.** Half an hour total. The paper holds up market-clearing-produced benchmarks and hub aggregation as the thing compute hasn't reached; Order 888 is the institutional-sequencing lesson. Both belong in your conclusion about where compute markets are headed.

**Before Day 7 — hedonic index methods** (the S&P/Case-Shiller methodology doc is a readable worked example) **and IOSCO Principles for Financial Benchmarks.** Both Ornn's CEO and the paper reach for housing-index analogies for GPU heterogeneity. Twenty minutes each, and they give your index discussion actual grounding.

**Anytime, low cost — Dave Friedman's *Compute Derivatives Market Primer* and *How to Control Your AI Compute Budget*.** The clearest plain-English treatment of the space. Good subway reading during the build.

---

## Part 3 — Deliberately not now

These are interesting and will expand your scope past a week. Note them for later.

- **Geman, *Commodities and Commodity Derivatives*** — the fuller treatment of convenience yield. Eydeland & Wolyniec covers what you need.
- **Krishna, *Auction Theory*** / **Milgrom, *Putting Auction Theory to Work*** — you're comparing heuristics, not designing a mechanism. This is a different project.
- **AWS EC2 Spot design papers** — preemption-instead-of-commitment is a fascinating alternative path and not the one you're modeling.
- **SF Compute, NVIDIA DGX Cloud Lepton, OpenRouter** — the routing-intermediary story matters for where indices go next. Write-up footnote at most.
- **Silicon Data's methodology** — relevant, but their term curves aren't free, and reading about data you can't use invites scope creep.

---

## Sequence

| | |
|---|---|
| **Pre-day 1** | Bandi & Su §2, §4. Ornn Basis Gap. a16z post. Write the positioning page. |
| **Pre-day 2** | Littlewood. Talluri & van Ryzin ch. 2–3, 3.3. Bandi & Su §5–6. |
| **Pre-day 3** | Schwartz (1997). Lucia & Schwartz (2002). |
| **Pre-day 4** | Operational reconnaissance: pull every source, cache it, start the collector. |
| **Then** | Build. Read Part 2 items the evening before the day they attach to. |

---

## The one thing to get right

If you read nothing else: **Bandi & Su, twice, plus the positioning page.**

It changes your framing from "I discovered compute behaves like power" to "I applied an established result to the operator's decision problem, which the authors flag as open." The second is a stronger claim, better supported, and takes half a day to earn.
