# An Introduction to Revenue Management

> INFORMS TutORials in Operations Research chapter by Garrett J. van Ryzin (Columbia) and Kalyan T. Talluri (Universitat Pompeu Fabra), published 2005 and reissued 2014, excerpted from their book *The Theory and Practice of Revenue Management* (Springer, 2004). It is the canonical statement of how sellers of perishable capacity decide whether to accept an offer. For this project it governs the **shape of the R3/R4 accept rule**: it establishes that the accept threshold is a displacement cost measured as a difference of value functions, that this threshold must be a function of remaining capacity and remaining time, and that booking limits, protection levels, and bid-price tables are three presentations of one underlying policy. It also supplies the assumption list against which the compute problem's departures are documented.

**Source:** `docs/source/van-ryzin-talluri-2014-an-introduction-to-revenue-management.pdf`

**Covers:** Single-resource capacity control (control types, displacement cost, static and dynamic models, EMSR heuristics, customer choice) and dynamic pricing (demand modeling, single- and multi-product, price skimming). **Does not cover network capacity control** — no DLP, RLP, DP decomposition, or virtual nesting. For those, see `network_revenue_management.md`.

## Table of contents

```
1.   Introduction
1.1    Demand-Management Decisions
1.2    What's New About RM?
1.3    The Origins of RM
1.4    Consequences of the Airline History
1.5    A Conceptual Framework for RM
1.6    Industry Adopters Beyond the Airlines
1.7    Overview of Topics

2.   Single-Resource Capacity Control
2.1    Types of Controls
2.1.1    Booking Limits                                          ✓ (vocabulary)
2.1.2    Protection Levels                                       ✓ (vocabulary)
2.1.3    Standard vs. Theft Nesting                              — skip (authors call it folklore)
2.1.4    Bid Prices                                              ✓
2.2    Displacement Cost                                         ✓
2.3    Static Models                                             ✓ (assumption list only)
2.3.1    Littlewood's Two-Class Model                            ✓
2.3.2    n-Class Models                                          ~ skim
2.3.3    Heuristics (EMSR-a, EMSR-b)                             ~ skim
2.4    Dynamic Models                                            ✓
2.4.1    Formulation and Structural Properties                   ✓
2.5    Customer-Choice Behavior
2.5.1    Buy-Up Factors                                          — skip
2.5.2    Discrete-Choice Models                                  — skip

3.   Dynamic Pricing
3.1    Price-Based vs. Quantity-Based RM                         ✓
3.2    Industry Overview (Retailing / Manufacturing / E-business) — skip
3.3    Examples of Dynamic Pricing                               — skip
3.4    Modeling Dynamic Price-Sensitive Demand                   — skip
3.5    Basic Single-Product Dynamic Pricing Without Replenishment — skip
3.6    Multiproduct, Multiresource Pricing
3.6.1    A Basic Deterministic Model Without Replenishment       ✓
3.6.2    Action-Space Reductions                                 — skip (overkill)
3.7    Finite-Population Models and Price Skimming               — skip

4.   Summary and Conclusions
     Appendix: Notes and Sources                                 ~ skim (bibliography)
     References
```

---

## Relevant content

### §2.1.1–2.1.2 — Booking limits and protection levels (pp. 149–151)

Two descriptions of one policy, from opposite ends. A **booking limit** `b_j` caps total sales to class *j and below*. A **protection level** `y_j` reserves capacity for class *j and above*. They convert exactly:

```
b_j = C − y_(j−1),   j = 2,…,n      with b_1 = C, y_n = C
```

Worked example from Figure 1 — capacity 30, classes at $100/$75/$50:

| | Class 1 | Class 2 | Class 3 |
|---|---|---|---|
| Booking limit `b_j` | 30 | 18 | 8 |
| Protection level `y_j` | 12 | 22 | 30 |

**Partitioned** controls carve capacity into non-shared blocks and can close a high-revenue class while low-revenue inventory sits unsold. **Nested** controls let higher classes reach into capacity reserved for lower ones, so leftovers flow upward. Nested is standard practice. Note that partitioned booking limits and partitioned protection levels are trivially identical — the distinction only carries information in the nested case.

> **Project note.** These are *class-based* controls: the decision depends on which bucket a request falls into. Compute bids carry an actual price per GPU-hour and vary simultaneously on price, node count, and term, so there is no total order to nest along. Use as a business-rule layer ("cap spot at 60% of a deployment"), not as the pricing engine.

### §2.1.4 — Bid prices (p. 152)

A bid-price control is **revenue-based rather than class-based**: set a threshold, accept if the request's revenue exceeds it. Storage is one value rather than a vector of limits.

Two points that matter operationally:

1. **The threshold must be state-dependent.** The common objection — that a bid price will sell unlimited capacity to anything above the threshold — holds only for a *static* threshold. Made a function of remaining capacity, π(x) behaves exactly like a nested booking limit, closing successive classes as capacity drains. In the Figure 1 example: above 22 units remaining, π(x) < $50 and all classes open; between 13 and 22, π(x) sits between $50 and $75 so only classes 1–2 open; at 12 or fewer, π(x) exceeds $75 and only class 1 opens.
2. **Bid prices discriminate within a class.** Where a class-based control must accept or reject an entire class, a bid price can take only the higher-revenue requests inside it — provided actual revenue is observable at request time.

> **Project note.** Point 2 is why R3/R4 use thresholds rather than limits: compute bids carry real prices, and class-based control would discard that information. Point 1 is a hard requirement — a fixed floor is exactly the unsafe static-threshold case.

### §2.2 — Displacement cost (p. 152)

The whole logic in two rules:

1. Allocate capacity to a request **iff its revenue exceeds the value of the capacity required to satisfy it**.
2. That value is the **expected displacement (opportunity) cost** — the expected loss in future revenue from using the capacity now rather than holding it.

Formally, with `V(x)` the optimal expected revenue given remaining capacity `x`:

```
displacement cost = V(x) − V(x − 1)
```

Most of the theory is analysis of this value function; the decision rule itself is just revenue vs. displacement cost.

> **Project note.** This is the project thesis, stated by the authors and citable. The three-cost decomposition (direct / opportunity / stranding) is a decomposition *of this quantity* for a setting where the capacity consumed is a bundle rather than a single unit.

### §2.3 — Static model assumptions (pp. 153–154)

Six assumptions, listed here because each is a documented departure point:

1. Class demands arrive in **non-overlapping intervals, low revenue before high**.
2. Class demands are **independent** random variables.
3. Demand for a class **does not depend on the controls** (no buy-up, no diversion).
4. Within-period demand detail is suppressed — aggregate arrival, single accept decision.
5. **Either no group bookings, or groups can be partially accepted.**
6. **Risk neutrality.**

> **Project note.** Assumption 5 is the one to cite. A compute bid with a whole-node constraint and a min/max range is precisely a group booking that cannot be split arbitrarily — and the authors flag it as an assumption rather than a result. Assumption 1 also fails: bids arrive in arbitrary order, which is what pushes the project to §2.4's dynamic model.

### §2.3.1 — Littlewood's rule (p. 154)

Two classes, prices `p_1 > p_2`, class 2 arriving first. With `x` units left, holding the marginal unit for class 1 is worth `p_1·P(D_1 ≥ x)`. So accept class 2 iff:

```
p_2 ≥ p_1 · P(D_1 ≥ x)
```

The right side decreases in `x`, giving an optimal protection level `y*_1 = F_1^(−1)(1 − p_2/p_1)`.

Under normal demand this becomes `y*_1 = μ + zσ` with `z = Φ^(−1)(1 − p_2/p_1)`. The lower the ratio `p_2/p_1`, the more you protect — take very low prices only when the chance of selling high is small.

### §2.3.3 — EMSR-a and EMSR-b (pp. 159–160)

Both due to Belobaba; both reduce the *n*-class problem to repeated two-class comparisons.

- **EMSR-a** applies Littlewood pairwise and **sums the resulting protection levels**.
- **EMSR-b** **aggregates future demand** into one pseudo-class at a weighted-average revenue, then applies Littlewood once.

EMSR-b generally performs better and is more common. Reported performance: EMSR-b consistently within about 0.5% of optimal, EMSR-a occasionally off by roughly 1.5%; a later Lufthansa study found neither dominating.

The authors' discussion of *why heuristics persist* despite optimal controls being cheap to compute — inertia, familiarity, "approximately right beats precisely wrong" — is worth reading as a direct parallel to the R1-vs-R3 comparison.

### §2.4 / §2.4.1 — Dynamic models (pp. 160–163)

Relaxes the low-to-high arrival ordering. Time runs forward over `T` periods, at most one arrival per period, class `j` arriving in period `t` with probability `λ_j(t)`, `Σ_j λ_j(t) ≤ 1`. Requires Markovian arrivals, which restricts how demand variability can be modeled, and needs a booking-curve estimate.

Bellman equation, with `ΔV_(t+1)(x) = V_(t+1)(x) − V_(t+1)(x−1)`:

```
V_t(x) = V_(t+1)(x) + E[ max_{u∈{0,1}} { (R(t) − ΔV_(t+1)(x)) · u } ]
```

Accept a class-*j* request iff `p_j ≥ ΔV_(t+1)(x)`.

**Proposition 2** — the marginal value:
- (i) **decreases in remaining capacity** `x`
- (ii) **decreases as time elapses** (fewer remaining chances to sell)

**Theorem 2** — the optimal control is implementable as time-dependent nested protection levels, time-dependent nested booking limits, **or a bid-price table** `π_t(x) = ΔV_t(x)`. All three are equivalent.

Practical note from the authors: the value function changes slowly, so periodic re-solving rather than continuous updating is usually near-optimal.

> **Project note.** This is the closest model in the document to the compute problem, and Proposition 2 derives the behaviour the project needs — opportunity cost rising as inventory tightens, falling as the window closes. Theorem 2(iii) is the implementation. What it lacks is the *bundle*: a single resource, one unit per request. Bids spanning multiple time buckets at multiple nodes need the network treatment.

### §3.1 — Price-based vs. quantity-based RM (pp. 170–171)

Which lever a firm uses comes down to where it has flexibility. Airlines commit to published fares in advance (advertising, distribution, administrative simplicity) but have near-perfect supply flexibility across fare products sharing one cabin — so they manage **quantity**. Apparel retailers commit to stock levels far ahead but change prices cheaply — so they manage **price**.

The authors note that where a firm genuinely has both levers, price-based RM dominates: rationing reduces sales by limiting supply, whereas raising price reduces sales *and* raises revenue simultaneously.

> **Project note.** Operators publish floors and buy-now rates in advance, then manage the accept decision per bid. That is the airline pattern, and it is the justification for framing this as quantity-based RM. Worth stating explicitly in the write-up, since a reader may ask why the project isn't about dynamic pricing.

### §3.6.1 — Basic deterministic multiproduct, multiresource model (pp. 182–184)

The structural substitute for the network DLP, in price-setting form.

`n` products indexed `j`, `m` resources indexed `i`, `T` periods. Product `j` uses quantity `a_ij` of resource `i`; the matrix `A = [a_ij]` is the bill of materials. Capacities `C = (C_1,…,C_m)`. Revenue rate `r(t,d)` assumed bounded and jointly concave in the demand vector `d`.

```
max    Σ_t r(t, d(t))
s.t.   Σ_t A·d(t) ≤ C
       d(t) ≥ 0
```

KKT conditions, with `J(t,d) = ∇_d r(t,d)` the marginal-value vector and `π*` the dual on the capacity constraints:

```
(35)   J(t, d*(t)) = A·π*        marginal revenue = marginal opportunity cost of resources used
(36)   π* · (C − Σ_t A·d(t)) = 0  positive shadow price only on binding capacity
(37)   π* ≥ 0
```

`π*` is the vector of per-resource shadow prices. Example 4 works a six-node, two-hub airline network where itineraries span multiple legs.

> **Project note.** Same Lagrangian structure as the accept/reject DLP, different control variable — here you choose `d` and prices follow; in the compute problem prices arrive exogenously and you choose accept/reject. Translate `π*` directly: resource `i` = a (deployment, time-bucket) pair, `C_i` = node count in that bucket, `a_ij` = nodes required by bid archetype `j` in bucket `i`, `π*_i` = shadow price per node-hour. Opportunity cost of a bid = `n × Σ_(buckets spanned) π*_i × hours`.
>
> Note `a_ij` is integer-valued here, not binary — the model already tolerates a product consuming several units of a resource.
