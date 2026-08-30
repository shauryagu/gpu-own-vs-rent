# Vocabulary

Canonical names for this repo. Everyday English first, then the symbol.
If two words look like synonyms, they are not — use the row in this file.

Do not call leftover and salvage by the same name. Do not call \(F(\theta)\) a
futures price. Invert \(S\) is daily-index only.

Binding spec: [designs/2026-08-22-gate-0-1-mvp.md](designs/2026-08-22-gate-0-1-mvp.md).
Positioning: [positioning.md](positioning.md). Bandi & Su §5.1:
[source/AiComputeAssetPricing.md](source/AiComputeAssetPricing.md).

## Objects

| Term | Symbol | Meaning in this repo | Not this |
|---|---|---|---|
| GPU / chip / hardware stock | — | Durable machine you can buy. Exists across years. | A rental hour |
| GPU-hour / service flow | — | One hour of using a chip. Unused today, gone. | The chip |
| Spot / observed rental | \(S\) | Market price of one GPU-hour from OCPI. Unit: USD / GPU-hour. Invert uses **daily-index** only. | Hourly current; history `2.88`; a futures quote |
| OCPI / the index | — | Ornn’s published rental number. We **read** it. | An index we publish |

## Own vs rent

| Term | Symbol | Meaning in this repo | Not this |
|---|---|---|---|
| Own | — | Pay \(P\), keep the chip, earn or save rent on used hours, scrap at \(T\). | Renting the hour |
| Rent | — | Pay \(S\) per GPU-hour and never hold the chip. | Owning |
| Fair deal | NPV \(= 0\) | Own and rent have the same value under declared \(\theta\). | “Owning wins”; minimized basis risk |
| User cost of capital | \(F(\theta)\) | Rental rate that makes you indifferent between owning and renting the hour. | Bandi & Su futures \(F_t(T)\) |
| Indifference | — | You would not pay to switch sides at that \(S\) and \(\theta\). | An optimum |

## Cash flows and the identity

| Term | Symbol | Meaning in this repo | Not this |
|---|---|---|---|
| Purchase / cost basis | \(P\) | Declared outlay for one GPU (`--purchase-cents`). USD. | MSRP; Epoch release price; a fitted value |
| MSRP / release price | — | Vendor or database launch price. Annotation only. | \(P\) |
| Economic life | \(T\) | Declared years the chip is in the model. Positive integer. | Silent default of 6; “how long NVIDIA supports it” unless you chose that |
| Utilization | \(u\) | Fraction of civil hours the GPU is on **and earning** \(S\). \((0,1]\). Declared, never fitted. | Occupancy we estimate |
| Civil hours | \(H\) | Clock hours per year. Here \(H = 8760\). | Utilized hours |
| Utilized hours | \(h = u H\) | Hours per year that produce rent. | Wall-clock duration (`Hours`) |
| Salvage / residual | \(R\) | USD at end of year \(T\) (resale, scrap, or disposal if negative). Declared only to print \(F(\theta)\). | Leftover \(L\); silent 0 except the accounting overlay |
| Energy cost | \(e\) | USD per utilized GPU-hour for power: TDP × 1 h × \(\pi\) × PUE. Default \(\pi = 0\) so \(e = 0\), product still evaluated. | A skipped term |
| TDP | — | Chip thermal design power, kW. Physics (`f64`). | Purchase price |
| PUE | — | Datacenter wall power ÷ IT load. MVP \(= 1.0\). | Something to invent from a 10-K in Gate 1 |
| Electricity price | \(\pi\) | USD / kWh. Declared. Default 0. | LMP we have not fetched |
| Discount rate | \(r\) | Declared annual required return, \(\ge 0\). | A WACC we estimate |
| Discounting | \((1+r)^{-t}\) | A dollar at year \(t\) in today’s money. | Continuous-time veneer |
| NPV | — | Discounted owner cash flows including \(-P\). Fair \(\Leftrightarrow\) NPV \(= 0\). | A futures P&L |
| Annuity factor | \(A(r,T)\) | PV of $1 at the end of each of \(T\) years. \(A = T\) if \(r = 0\); else \((1-(1+r)^{-T})/r\). | A solver |
| Identity / fair rent | \(F(\theta)\) | USD / GPU-hour that sets NPV \(= 0\) for fully declared \(\theta\). Printed only with `--residual-cents`. | Futures price \(F_t(T)\) |
| Capital-recovery rent | \(F_{\mathrm{capital}}\) | \(P / (h A)\). Recover purchase at scrap-zero, ignore energy. | A second name for \(F(\theta)\) |
| Leftover | \(L = S - F_{\mathrm{capital}} - e\) | Unexplained rent after capital recovery and energy. USD / GPU-hour. **Rises** with \(S\). JSON: `leftover_usd_per_gpu_hour`. | “Implied residual”; \(R^{\star}\) |
| Implied salvage | \(R^{\star}\) | \(R\) that sets NPV \(= 0\) given \(S\). USD at year \(T\). **Falls** with \(S\). May be **negative** — do not clamp. JSON: `implied_salvage_usd`. | Leftover; “implied residual” |
| Parameter vector | \(\theta\) | One declared \((T, u, R)\) (plus \(P, e, r\) when needed). | The feasible set |
| Feasible set | \(\Theta(S)\) | Every \(\theta\) consistent with this \(S\). A **set**, not a point. Later: \(\Theta_L(S)\) and \(\Theta_{R^{\star}}(S)\). | A data-center region; a unique residual |
| Non-uniqueness / identification | — | One \(S\), many \((T,u,R)\). That set is the result. | A failed point estimate |
| Accounting point / overlay | \(T=6\), \(R=0\) | Labeled scenario, same \(P,u,e,r\). How books often write chips off. | Silent primary \(\theta\); a 10-K claim before Gate 3 |
| Economic vs accounting life | — | Economic \(T\) is the NPV story you declared. Accounting life is the depreciation schedule. Report the gap. | The same number |

## Markets and Bandi & Su

| Term | Symbol | Meaning in this repo | Not this |
|---|---|---|---|
| Spot market | \(S_t\) | Pay now, use the hour now. | A futures curve |
| Forward | — | Private contract on a future hour or month. Usually not marked daily. | Our \(F(\theta)\) |
| Futures | \(F_t(T)\), \(F_t^{\mathrm{m}}(M)\) | Exchange contract, often marked to market, cash-settled on an index. | The capital-recovery rent |
| Bandi & Su futures | \(F_t(T)\) | Their price **on the service flow**. Cite §5.1; do not re-derive. | Our \(F(\theta)\) |
| No-arbitrage | — | A price relation that must hold or someone locks in riskless profit. | A relation we cannot trade |
| Cash-and-carry | — | Buy spot, store, sell forward (or reverse). Needs a **storable** good. **Fails** for a GPU-hour. | A term to add to \(S\) to get \(F(\theta)\) |
| Storability | — | Ability to warehouse the thing and deliver it later. GPU-hour: **no**. | The chip (the chip is durable; the hour is not) |
| Storage cost | \(c\) | Cost of holding inventory. Meaningless for an hour that already vanished. | Energy \(e\) |
| Convenience yield | \(y\) | Benefit of holding physical inventory. Not an MVP knob. | Leftover \(L\) |
| Term rental | \(\Pi_t(t \rightarrow T)\) | Reserved USD/hour for a multi-month window. Bandi §5.2. Not implemented here. | Spot \(S\) |
| Settlement | — | Number a futures contract pays off against. | Our invert input unless it is daily-index |
| Daily-index | `ocpi.daily-index` | OCPI latest settled daily close. **Authoritative invert \(S\)**. | Current; history last point |
| Hourly current | `ocpi.current` | Latest intra-day print. Collected because it disappears. | Invert \(S\) |
| Daily history | `ocpi.daily-history` | Free ~3-month window, often rounded to 2 decimals (`2.88`). Cached; refused as \(S\). | Daily-index token `2.879583333333333` |

## Basis (vocabulary, not an optimizer)

| Term | Symbol | Meaning in this repo | Not this |
|---|---|---|---|
| Basis | — | What the contract or index guarantees minus what the position actually is (Ornn *Basis Gap*). | A quantity we minimize |
| Cash-settled hedge | — | Hold chips; hedge with a future on generic OCPI. Tracks the **index**, not your SKU. | A perfect hedge of the physical book |
| Residual risk | — | Risk left because the chip is not the index. \(\theta\) outside \(\Theta(S)\). | Leftover \(L\) (different unit and job) |
| Haircut | — | Extra collateral a clearer might demand for that leftover risk. Not computed here. | \(R^{\star}\) |
| Minimize basis risk | — | **Not this project.** We list \(\theta\) that set the basis to zero. | The research question |

## Numbers we refuse to fake

| Term | Symbol | Meaning in this repo | Not this |
|---|---|---|---|
| Declared point / band | — | \(P\) is one number you type, understood to sit in an unsourced range. No default dollars in the binary. | Epoch \(33600\) as \(P\) |
| Half-life | — | Time for a mean-reverting shock to decay by half. **Not estimated.** Not an MVP parameter. | Something to fit on ~90 daily points |
| Mean reversion | — | Drift of a price back toward a long-run level. Sweep later if at all. | A calibrated OU on free history |
| WACC | — | Firm cost of capital from books. We will not invent one and call it \(r\). | `--discount-rate` |

## Units

| Term | Type | Meaning | Must not add to |
|---|---|---|---|
| USD | `Usd` | A lump (purchase, salvage). | `UsdPerGpuHour`, `GpuHour` |
| USD per GPU-hour | `UsdPerGpuHour` | A rate (\(S\), \(F\), \(L\), \(e\)). | `Usd`, `GpuHour` |
| GPU-hour | `GpuHour` | Quantity of service. | `Hours`, `H100eHour` |
| Wall-clock hours | `Hours` | Duration for physics (energy). | `GpuHour` |
| H100-equivalent hour | `H100eHour` | Quality-adjusted hour. Gate 1: identity for H100 SXM only. | Raw `GpuHour` of another SKU |

## Quick collisions

| If you almost wrote | Write instead |
|---|---|
| implied residual | leftover \(L\) **or** salvage \(R^{\star}\), never both under one name |
| futures price / forward rent | \(F(\theta)\) if NPV identity; \(F_t(T)\) only when citing Bandi |
| implied residual (increases with \(S\)) | leftover \(L\) |
| implied residual (USD at year \(T\)) | salvage \(R^{\star}\) |
| the OCPI price | name the series: current, daily-index, or history |
| invert \(S\) | daily-index token, not `2.88`, not hourly current |
| default life / default rate | there is none; accounting overlay is labeled \(T=6\), \(R=0\) only |
