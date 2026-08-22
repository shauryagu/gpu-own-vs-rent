# An Analysis of Bid-Price Controls for Network Revenue Management

> Kalyan Talluri (Universitat Pompeu Fabra) and Garrett van Ryzin (Columbia), *Management Science* 44(11), November 1998. The paper that put bid-price control on a theoretical footing: it formulates the network accept/reject problem as a dynamic program, proves bid-price control is **not** optimal in general, identifies exactly why it fails, proves it is asymptotically optimal at large scale, and evaluates the standard approximation schemes. For this project it governs three things: the **formulation of a whole-node bid as a group request**, the **choice of DLP as the R3 engine**, and — most importantly — the **documented argument that the stranding cost term is structurally outside what any additive bid-price scheme can express**, with a reproducible counter-example to prove it.

**Source:** `docs/source/network_revenue_management.pdf`

**Covers:** Network (multi-resource) capacity control under a general demand process — optimal DP structure, non-optimality of bid prices and why, asymptotic optimality, uniqueness of bid prices, and the DLP / PNLP / prorated-EMSR approximations. **Does not cover** single-resource control (see `van-ryzin-talluri-2014-an-introduction-to-revenue-management.md`), RLP, DP decomposition, virtual nesting, overbooking, or pricing.

## Table of contents

```
     Introduction and Overview
       Related Literature                                        — skip
       Organization of the Paper                                 — skip

1.   Formulation                                                 ✓ (group-request paragraph)
2.   Structure of the Optimal Control                            ✓ (Proposition 1)

3.   Nonoptimality of Bid-Price Controls
3.1    A Counter Example to Bid-Price Optimality                 ✓✓ load-bearing
3.2    Bid-Price Optimality and the Structure of the
       Value Function                                            ✓ (Proposition 2 + discussion)

4.   An Asymptotic Analysis of Bid-Price Controls
4.1    An Upper Bound Problem                                    ~ skim (skip proofs)
4.2    Asymptotic Analysis of Bid Prices Derived from the
       Upper Bound Problem                                       ✓ (Theorem 1 statement + scaling only)
4.3    Uniqueness of the Asymptotic Bid Prices                    ✓ (Proposition 3)
4.4    Bid Prices and Opportunity Cost                           ✓✓ load-bearing

5.   A Unified View of Bid Price Approximation Schemes
5.1    Deterministic Linear Program (DLP)                        ✓
5.2    Probabilistic Nonlinear Program (PNLP)                    — skip (know it underperforms)
5.3    Prorated EMSR                                             — skip (airline-specific)
5.4    Asymptotic Bid Prices                                     ~ skim
5.5    Numerical Examples                                        ~ skim (Figures 1–3)

6.   Conclusions                                                 ~ skim
     References
```

---

## Relevant content

### §1 — Formulation, and the group-request construction (pp. 1579–1580)

A network of `m` legs (resources) supporting `n` itineraries (products). `a_ij` = units of leg `i` consumed by itinerary `j`; `A = [a_ij]`. Network state is the capacity vector `x = (x_1,…,x_m)`; selling itinerary `j` moves it to `x − A^j`. Time is discrete, counted backwards, fine enough that at most one request arrives per period. No cancellations or no-shows.

The demand model is deliberately general: a single random vector `R_t` per period, where a positive component `R_t^j` signals a request for itinerary `j` **at that revenue**. Fares are therefore random within a class, and no assumption is made about arrival order — high-before-low, low-before-high, or interleaved.

**The construction that matters here:** `a_ij` is *not* restricted to 0/1. Group requests are modeled by **adding one column of `A` per group size**, with the nonzero entries equal to the group size and the revenue scaled to the group total. A family booking four seats is a column with `a_ij = 4` on each leg of the path. The probability model then reflects the likelihood of a request for each group size.

> **Project note.** This is the formulation for a whole-node bid. A bid for 6 nodes spanning buckets *t₁…t₂* is a column with `a_ij = 6` on every bucket in that span. A min/max node range becomes **several columns — one per admissible node count** — and the fill recommendation is the choice among them. Cleaner than treating fill as a separate search, and it comes from the source.

### §2 — Structure of the optimal control (pp. 1580–1581)

With `J_k(x)` the optimal expected revenue at time-to-go `k` and capacity `x`, the Bellman equation is

```
J_k(x) = max_{u ∈ U(x)} E[ R_k·u_k(x,R_k) + J_(k−1)(x − A·u_k(x,R_k)) ]
J_0(x) = 0
```

**Proposition 1.** An optimal control exists and has the form: accept a request for itinerary `j` at fare `r` iff capacity allows **and**

```
r ≥ J_(k−1)(x) − J_(k−1)(x − A^j)
```

Accept only when the fare exceeds the opportunity cost of the **whole bundle** of capacity consumed. Exact, and intractable — everything downstream is approximation.

> **Project note.** Note the shape: a difference of value functions evaluated over `A^j` as a unit, *not* a sum over the resources in it. That distinction is the entire subject of §3.

### §3.1 — The counter-example (pp. 1581–1582)

Two legs, **one unit of capacity each**, two periods left.

| Period | Itinerary | Fare | P(arrival) |
|---|---|---|---|
| 2 | through (1,1) | $500 | 0.4 |
| 2 | local (1,0) | $250 | 0.3 |
| 2 | local (0,1) | $250 | 0.3 |
| 1 | through (1,1) | $500 | 0.8 |
| 1 | no arrival | — | 0.2 |

Accepting either local in period 2 yields $250 and forecloses the through fare in period 1. Holding both legs gives an expected $400. So the **optimal policy rejects both locals** while still accepting the through fare in period 2.

For a bid-price control to reproduce that, it would need:

```
μ_1 > 250,   μ_2 > 250,   μ_1 + μ_2 ≤ 500
```

Impossible. The best any bid-price policy achieves is $400; the optimal policy earns **$440 — about 10% more**.

The authors note this is not an end-of-horizon artifact: counter-examples exist with arbitrarily large remaining leg capacities.

> **Project note — this is the load-bearing result.** Relabel legs as adjacent time buckets: two short bids versus one long bid spanning both. Accepting the shorts fragments the window and blocks the long. That is exactly the **stranding cost**, and this section proves no additive per-resource threshold scheme can represent it.
>
> The write-up claim sharpens from "the LP relaxes integrality" to: *stranding is structurally outside the expressive range of bid-price control, per Talluri & van Ryzin §3.1.*
>
> **Build this as a unit test.** Reproduce the two-leg instance in the simulator, assert the $440 vs. $400 gap, then show the same structure arising in a GPU deployment.

### §3.2 — Why bid prices fail (pp. 1582–1583)

**Proposition 2.** Under mild conditions, a bid-price scheme is optimal only if the value function satisfies

```
J_k(x) − J_k(x − A^j)  =  Σ_(i ∈ A^j) [ J_k(x) − J_k(x − e_i) ]
```

That is, the bundle's opportunity cost must equal the **sum of the single-resource opportunity costs**. This linearity does not hold in general.

Two named causes, both visible in §3.1:

1. **Selling a unit is a large relative change in capacity.** Simultaneous large changes across several resources cannot be expected to have the same revenue effect as the sum of the individual changes. Gradient-based reasoning fails; second-order interaction terms matter.
2. **Future revenue can depend non-linearly on remaining capacity** — in the counter-example, on the **minimum** capacity across the two legs, not the sum. The opportunity cost of using one leg then equals the cost of using both, destroying the required linearity.

The authors compare this to degeneracy in mathematical programming, and note it can appear in the optimal value function or in any approximation to it.

> **Project note.** Both causes are acute at compute scale. One node out of sixteen is a large relative change. And a multi-day bid needs the **minimum available node count across every bucket it spans** — precisely cause 2.

### §4.2 — Asymptotic optimality (pp. 1584–1586)

**Theorem 1.** Scale the problem by an integer `θ`: capacities `θx`, time-to-go `θk`, each period split into `θ` i.i.d. periods. Then the fixed-bid-price heuristic using `μ*` from the upper-bound problem satisfies

```
J^H_(θk)(θx) / J_(θk)(θx)  ≥  1 − O(θ^(−1/2))     →  1  as θ → ∞
```

Read the statement and the scaling construction. Skip the coupling argument and the Gallego bound.

Two properties worth carrying: the asymptotically optimal bid prices are **constant over time**, even under non-stationary demand, and `μ*` is the same vector for every `θ`.

> **Project note — this disqualifies the usual defense.** Asymptotic optimality holds as capacity and volume grow together. A 16-node deployment is not that regime. The standard justification for bid-price control does not apply at compute scale, and this is the same conclusion §3.2 reaches from the other direction (large relative capacity changes). Two independent, citable arguments that the problem sits in the failure region.

### §4.3 — Uniqueness (p. 1586)

**Proposition 3.** If the fare distributions have sufficiently large tails and **rank(A) = m**, the asymptotically optimal bid-price vector is unique. Otherwise multiple optimal vectors can exist — for example, two legs in series with a single itinerary traversing both gives rank(A) = 1 < m = 2, and only the sum `μ_1 + μ_2` is determined. Highly concentrated fare distributions produce the same ambiguity.

In practice the authors expect `n ≫ m`, making full rank likely.

> **Project note — actionable.** Fine time bucketing inflates `m` and can push it past the variety of bid archetypes `n`, making `A` rank-deficient. Duals then become non-unique: identical state, different opportunity cost depending on solver path. **Keep buckets coarse relative to bid variety, and assert rank(A) = m as a test.**

### §4.4 — Bid prices are not opportunity costs (pp. 1586–1587)

There is **no one-to-one correspondence** between good bid prices and true opportunity cost. A bid-price vector can produce near-optimal accept/deny decisions while badly estimating the marginal value of capacity.

The authors' example: on a single leg where high fares arrive strictly before low fares, first-come-first-serve is optimal, so a **constant bid price of zero is optimal** — while the true opportunity cost `J_k(x) − J_k(x−1)` is nowhere near zero.

Comparing revenue to true opportunity cost is *sufficient* for optimal decisions but not *necessary*; other thresholds give identical decisions. But the authors stress that in practice an accurate opportunity-cost estimate is often essential — specifically for **special-event and ad-hoc group bookings that fall outside the forecast**, where a good decision requires a real assessment rather than a decision-equivalent threshold.

The stated algorithmic challenge: construct bid prices that are near-optimal for acceptance decisions *and* good estimates of opportunity cost, so off-forecast group requests can be evaluated properly.

> **Project note.** The console **displays** opportunity cost to a human reviewer. A decision-equivalent number is not good enough for "what this bid forecloses." And bids with unusual node counts are exactly the ad-hoc group bookings the authors single out. Track decision quality and estimate quality as separate metrics.

### §5.1 — Deterministic Linear Program (p. 1587)

Bid-price schemes are usefully viewed as **approximations of the value function**, with the bid prices as the gradient (or a subgradient) of the approximation.

The DLP approximation:

```
J^LP_k(x) = max  Σ_j  E[R_j] · y_j
            s.t.  A·y ≤ x
                  0 ≤ y ≤ E[D]
```

`D_j` is demand-to-come for itinerary `j`; `y_j` is a static, non-nested allocation. Bid prices are the optimal duals on `A·y ≤ x`. If those constraints are degenerate at the optimum, multiple dual vectors exist and each is only a subgradient.

**Stated weakness:** the DLP uses **mean demand only** and discards all other distributional information. Consequence — **the dual is zero on any resource whose mean demand is below capacity.**

Despite this, Williamson's simulation studies found DLP bid prices performed well *with frequent reoptimization*, beating both PNLP and a range of leg-based EMSR heuristics.

> **Project note.** Expect long stretches where R3's opportunity cost is exactly zero and R3 collapses into price-greedy. That is the model behaving as documented, not a bug — surface it in the console rather than hiding it. "Frequent reoptimization" means re-solving on every bid arrival, which is cheap at this problem size (HiGHS via SciPy, or PuLP).

### §5.4–5.5 — Comparison and numerical behaviour (pp. 1588–1591)

**Nesting property.** The DLP and the asymptotic approximation are invariant to splitting one itinerary into two identical columns; PNLP is not, because it allocates capacity separately to each. PNLP consistently produced lower revenue than DLP in simulation.

**Variance sensitivity.** DLP and asymptotic bid prices depend only on the first moment of demand, so both go to zero when mean demand is below capacity. The asymptotic method *does* account for variability in itinerary **revenues**, which DLP and PNLP ignore entirely.

From the numerical examples (3 fare classes, single leg, 30 periods):

- All three approximations **underestimate** the optimal bid price when remaining capacity is below mean total demand (Figure 1).
- DLP and asymptotic overestimate at low capacity and underestimate above mean demand; PNLP does the opposite (Figure 2).
- Raising fare variance leaves DLP and PNLP unchanged, but the optimal and asymptotic bid prices rise substantially at low capacity — approaching ~$150 rather than ~$120 (Figure 3). With many requests to choose from, it becomes optimal to be selective and accept only the right tail.

Related closed form: with one fare class and demand well above capacity, the optimal bid price tends to `r*` satisfying `E[D]·(1 − F(r*)) = x`, which can sit well above `E[R]`.

> **Project note.** Compute bids carry real prices with real dispersion, so the Figure 3 effect is live: when a deployment is nearly full, the correct threshold is above the mean bid price, not at it. DLP will miss this. Worth flagging as a known bias in the write-up, and as the motivation for a possible R5 using the asymptotic construction.
