# (Early) AI Compute Asset Pricing

> This is an early academic working paper (Bandi & Su, Johns Hopkins University, August 2026)
> that builds a first asset-pricing framework for AI compute — the rented GPU capacity that
> powers AI training and inference. It argues that standard futures-pricing logic (cash-and-carry,
> no-arbitrage from spot to forward) breaks down for compute because it is a non-storable service
> flow, proposes an alternative "synthetic futures" pricing approach built from existing term-rental
> contracts, and derives a risk-premium framework for how compute futures should eventually be
> priced once a formal futures market (CME/Silicon Data, ICE/Ornn) launches later in 2026.

**Source:** `docs/source/AiComputeAssetPricing.pdf`

**Covers:** The theoretical core of the paper — why storage-based no-arbitrage pricing fails for
compute, how synthetic futures are constructed from term rentals, and how a compute risk
premium should be priced. Does *not* include the compute-market/indexation background (Section 3),
the full empirical results (Section 7), or the references/appendices — flagged below for reference
but not extracted.

## Table of contents

1. Introduction
2. Related literature
3. The underlying: compute market and its indexation
   - 3.1 The compute market
   - 3.2 Compute price indexation
   - 3.3 Index data and preliminary analysis
4. The financialization of AI compute
   - 4.1 Designing early compute futures contracts ✓
5. The no-arbitrage pricing of compute futures ✓
   - 5.1 Storage-based no-arbitrage pricing ✓
   - 5.2 No-arbitrage pricing against term rentals ✓
     - 5.2.1 Limits to arbitrage ✓
6. The risk pricing of compute futures ✓
7. Empirical analysis
   - 7.1 Term rentals and synthetic futures prices
   - 7.2 Hold-to-maturity returns and risk premium
   - 7.3 Constant-maturity daily returns and risk premium
8. Conclusions ✓
   References
   A. Additional technical details
   - A.1 Synthetic futures values with discounting
   - A.2 Deriving the compute risk premium
   - A.3 The relation between Silicon Data term rates and implied forward rates
   - A.4 Implied forward rates: mapping tenors and expiration months
   - A.5 Equivalence between zero-tenor synthetic futures prices and spot prices
   - A.6 Run-off of the physical access wedge
   B. Additional calculations and empirical results
   - B.1 Compute service flow and capex to GDP
   - B.2 Full sample rolling constant-maturity returns

*(✓ = extracted below. Everything else — market background, indexation methodology, empirical
return panels, and appendices — lives only in the source PDF.)*

## Relevant content

### 4.1 Designing early compute futures contracts

Based on the current market and available data, the first working contracts will likely be
designed around the following features:

1. **The underlying: a standardized GPU rental index.** The first contracts are expected to be
   launched on a current Silicon Data on-demand rental index for the neo-cloud segment — H100,
   A100, or B200 (tickers SDH100RT, SDA100RT, SDB200RT).

2. **The settlement price and payoff.** The settlement price is the spot index value $S_T$ at
   delivery time $T$. The corresponding futures price is $F_t(T)$. The settlement payoff of a long
   futures position entered at time $t$ at price $F_t(T)$ is:

   $$S_T - F_t(T)$$

3. **Monthly settlement.** The first generation of compute futures may be settled against the
   average daily spot index over the delivery month. Let $M$ denote the delivery month and $d$ a
   specific day. The settlement price over month $M$ is:

   $$S_M^{\mathrm{m}} = \frac{1}{N_M}\sum_{d \in M} S_d$$

   with $N_M$ the number of daily observations in month $M$. The futures price quoted on day $t$
   for delivery in month $M$ is $F_t^{\mathrm{m}}(M)$, with settlement payoff $S_M^{\mathrm{m}} - F_t^{\mathrm{m}}(M)$.

4. **Contract sizing.** The paper treats the underlying as 1 GPU-hour by default (analogous to how
   Treasury futures allow "deliverable" alternative bonds via a conversion factor), noting that
   scaling to more GPUs/hours, or defining the contract in standardized "effective compute" units
   (e.g., 1 H100 GPU-hour), is straightforward.

Monthly averaging mirrors the settlement convention used in electricity futures markets, since
averaging over the delivery interval is natural for a non-storable service flow — a single
point-in-time price could contain excessive high-frequency noise. That said, the paper's
theoretical arguments below mostly use the cleaner point-in-time contract version for clarity.

**Marking-to-market.** Following standard futures convention, the marked-to-market cumulative
P&L for a long position entered on day $t$ is:

$$F_{t'}^{\mathrm{m}}(M) - F_t^{\mathrm{m}}(M) \tag{1}$$

for any marked-to-market date $t'$ between $t$ and settlement month $M$. The corresponding daily
return (assuming notional leverage) is:

$$r_{t+1} = \frac{F_{t+1}^{\mathrm{m}}(M) - F_t^{\mathrm{m}}(M)}{F_t^{\mathrm{m}}(M)}$$

---

### 5. The no-arbitrage pricing of compute futures

#### 5.1 Storage-based no-arbitrage pricing

**Core negative result: there is no natural no-arbitrage link between the current spot rental
price and the futures price for compute.** Because compute service flow is not storable,
cash-and-carry arguments cannot be conducted — a GPU-hour not used today cannot be carried
forward and delivered next month. Compute is closer to electricity than to oil, metals, or other
inventory assets.

The analysis assumes a frictionless futures market where:
- **(i)** there is no marking-to-market, and
- **(ii)** the futures contract is written on the same underlying as the forward contract (no basis risk).

If (i) and (ii) hold, futures and forward prices should coincide, and any link between forward
and spot prices would carry over to futures prices.

The standard storable-commodity no-arbitrage relation (continuous time) is:

$$F_t(T) = S_t \exp[(r_t + c_t - y_t)(T-t)] \tag{2}$$

where $S_t$ is the spot price, $r_t$ the financing rate, $c_t$ the storage cost, and $y_t$ the
convenience yield (benefit of holding inventory).

Ignoring convenience yield ($y_t = 0$), the classic cash-and-carry arbitrage works as follows:

- If $F_t(T)$ is **too high** relative to $S_t e^{(r_t+c_t)(T-t)}$: borrow, buy the commodity, pay
  storage/financing costs, and sell forward — capturing the arbitrage gain
  $F_t(T) - S_t e^{(r_t+c_t)(T-t)}$.
- If $F_t(T)$ is **too low**: short-sell the commodity, deposit proceeds at rate $r_t$, and buy
  forward — capturing the gain $S_t e^{(r_t+c_t)(T-t)} - F_t(T)$.

The first strategy is easy to implement and should erode any overpricing. The second is limited
in practice because owners of some commodities won't want to give them up even temporarily — this
asymmetry is exactly what gives rise to a positive convenience yield ($y_t > 0$) for storable goods.

**Why this fails for compute:** compute rental service has no storage technology. An unused
GPU-hour today is lost — it cannot be warehoused and delivered next month. The current spot
rental price $S_t$ is the price of *immediate access* to capacity, not the price of a durable
object that can be moved through time. As a result, Eq. (2) cannot provide a pricing restriction
linking compute spot and futures prices. A useful no-arbitrage reference must instead come from
contracts that already shift compute access across time — term rentals and reservation contracts.

#### 5.2 No-arbitrage pricing against term rentals

The physical compute market already has a mechanism for setting forward compute prices before a
formal futures market exists: **term rental (Reserved) contracts**, which commit to capacity over
a fixed term (a couple of months to 3 years).

Let $\Pi_t(t \rightarrow T)$ denote the fixed physical term rental rate (dollars per GPU-hour),
quoted at $t$, for compute access over $[t,T]$. Because a futures contract is a marginal claim at
a single future date/month, translating between term rental rates and futures prices requires a
transformation analogous to the zero-coupon-bond-to-forward-rate translation in fixed income.

**Synthetic futures prices are differences of adjacent term rental prices.** The synthetic futures
price for delivery month $M$ is:

$$F_t^{\mathrm{syn,m}}(M) = (M-t)\Pi_t(t \rightarrow M) - (M-t-1)\Pi_t(t \rightarrow M-1) \tag{3}$$

In the continuous-time limit, the synthetic forward price is the marginal extension of the term
rental price:

$$F_t^{\mathrm{syn}}(T) = \frac{\partial}{\partial T}\left[(T-t)\Pi_t(t \rightarrow T)\right] \tag{4}$$

**A strip of futures is equivalent to a financialized term rental rate.** As an alternative to a
term rental contract, a user could replicate the same compute access by buying a strip of futures
contracts across every date from $t$ to $T$. The market's implied term rental rate is the integral
of the futures price strip:

$$\Pi_t^{\mathrm{fin}}(t \rightarrow T) = \frac{1}{T-t}\int_t^T F_t(s)\, ds \tag{5}$$

(with the discrete monthly analogue in Eq. (6) of the source).

**No-arbitrage restrictions**, if physical term rentals and financial futures were perfect
substitutes:

$$F_t(T) = F_t^{\mathrm{syn}}(T) \quad\text{and}\quad \Pi_t^{\mathrm{fin}}(t\rightarrow T) = \Pi_t(t\rightarrow T) \tag{7}$$

**The arbitrage trades:**
- If $\Pi_t^{\mathrm{fin}} > \Pi_t$ (futures strip expensive vs. term rental): buy the underpriced
  physical term rental capacity, sublet it spot over the interval, and simultaneously sell the
  expensive futures strip. Assuming no basis risk, the spot legs cancel and the arbitrage gain is
  $\Pi_t^{\mathrm{fin}}(t\rightarrow T) - \Pi_t(t\rightarrow T)$.
- If $\Pi_t^{\mathrm{fin}} < \Pi_t$ (term rental expensive vs. futures strip): sell the overpriced
  term rental capacity, buy the needed capacity spot, and simultaneously buy the cheap futures
  strip — capturing the reverse gain.

The natural arbitrageurs are compute providers, who can easily intermediate across markets. A
similar arbitrage could in principle be run by anyone without touching the spot market if either
(1) the futures contract allowed physical delivery, or (2) the physical contract were cash-settled
— but neither condition currently holds.

##### 5.2.1 Limits to arbitrage

The frictionless no-arbitrage condition in Eq. (7) is not expected to hold exactly, because
physical term rentals and cash-settled futures are **not perfect substitutes** — introducing basis
risk. The viability of the two arbitrage directions is **asymmetric**:

- **Harder direction:** if physical term rentals are expensive relative to the futures strip, an
  arbitrageur ideally wants to buy the *specific* capacity underlying the sold rental agreement —
  but that capacity is tied to a particular provider, location, configuration, and reliability
  tier, while the futures contract references a generic index. The spot payment to source capacity
  and the spot payment in the futures payoff don't cancel cleanly. The arbitrageur is left exposed
  to basis risk.
- **Easier direction:** the reverse arbitrage (buy under-priced term rental, sell futures) is
  better executed by a provider, who controls capacity across many customers, regions, and
  configurations and can pool it — effectively self-indexing. Basis risk here is comparatively lower.

This asymmetry implies the no-arbitrage relationship holds with a **"physical access wedge"**:

$$\Delta_t^\Pi(t\rightarrow T) = \Pi_t(t\rightarrow T) - \Pi_t^{\mathrm{fin}}(t\rightarrow T) \tag{8}$$

and correspondingly:

$$F_t(T) = F_t^{\mathrm{syn}}(T) - \Delta_t^F(T) \tag{9}$$

The physical access wedge is expected to be:

- **Non-negative** ($\Delta_t^\Pi \geq 0$), due to the arbitrage asymmetry described above.
- **Increasing in time horizon** $T-t$. At short horizons the wedge reflects idiosyncratic
  provider/localized frictions; at long horizons it also reflects technological obsolescence — the
  physical hardware underlying a term rental ages, while a financial index can dynamically
  rebalance toward next-generation chips.

**Bottom line — synthetic futures prices are likely an upper bound on true financial futures
prices:**

$$F_t(T) < F_t^{\mathrm{syn}}(T) \tag{11}$$

Physical term rentals bundle price insurance *with* a real capacity-locking option, and that
option is valuable when users have limited alternatives for securing scarce compute — this option
value is what makes synthetic futures pricier than the (eventual) real futures contract. The
authors expect this tracking gap to shrink as intermediaries and standardized capacity packages
develop.

---

### 6. The risk pricing of compute futures

The paper draws an analogy to **electricity forward markets**: the electricity forward premium
$\lambda_t^{\mathrm{syn}}(T) = E_t[S_T - F_t^{\mathrm{syn}}(T)]$ can switch sign depending on which
side of the market — providers or users — exerts stronger hedging pressure. Providers exposed to
revenue risk tend to sell forward (pushing the premium up); users exposed to upward price spikes
tend to buy forward (pushing it down). Similar dynamics are expected for compute.

**The core risk-return relation:**

$$F_t(T) = E_t[S_T] - \lambda_t(T)$$

where $E_t[S_T]$ is the expected future spot rental price and $\lambda_t(T)$ is the risk premium
for holding a long compute futures position. Equivalently:

$$\lambda_t(T) = E_t[S_T - F_t(T)]$$

— the expected payoff of a zero-upfront-cost long futures position, i.e., the compensation
investors require for buying compute forward.

**Canonically**, the risk premium is the (standardized) covariance between the compute payoff and
the stochastic discount factor $M_{t,T}$:

$$\lambda_t(T) = -\frac{\mathrm{Cov}_t(M_{t,T}, S_T)}{M_{t,T}}$$

with $M_{t,T}$ increasing in the marginal utility of the "marginal" futures investor.

**Sign logic:** if compute providers are the primary hedgers/marginal investors, the covariance
between compute prices and their marginal utility is negative (providers benefit from *higher*
prices, so their marginal utility falls as prices rise). Providers are short hedgers; long
positions effectively insure them — and in equilibrium the compute risk premium is **positive** as
compensation for providing that insurance. (If AI developers were instead the dominant hedgers,
the logic reverses.)

At the market's initial launch, the authors expect natural hedgers (compute providers and AI
developers) to dominate, with providers' short-selling decisions particularly impactful early on.
As the market matures, hedge funds, CTAs, index funds, and asset managers would play a larger
role, and the risk-premium sign would then depend on whether compute prices are high in "good" or
"bad" states for the marginal investor generally. Under the assumption that a growing AI
infrastructure is broadly viewed as boosting profitability (i.e., compute prices are high in good
states), the risk premium would again be **positive**:

$$F_t(T) < E_t[S_T] \tag{12}$$

A long future position is then "risky" in the sense that it pays off precisely when payment is
least needed (in good states), so it must be priced at a discount to expected spot.

The authors caution this sign is not guaranteed — a supply-side story (e.g., a semiconductor
breakthrough sharply lowering compute costs and obsoleting existing benchmarks) could flip the
sign negative. They also note that, at this early stage — with hedging still primarily
*operational* rather than broadly financialized — it would be premature to apply the standard
asset-pricing factor toolkit (e.g., Fama-French-style covariance-with-priced-factors analysis) to
compute data. Instead, the paper's empirical work (Section 7) estimates the risk premium directly
via average hold-to-maturity and constant-maturity realized (excess) returns, without assuming a
specific asset-pricing model.

Finally, the authors stress that a **positive compute risk premium today is a statement about
provider hedging pressure**, not about compute's correlation with the broader economy — that
broader-market interpretation would only become relevant after mature financialization integrates
compute pricing with equity-market pricing factors.

---

### 8. Conclusions

- Compute is a key AI-economy input; its spot price reflects access to a scarce resource, its
  forward price reflects uncertainty about AI adoption, model progress, and infrastructure buildout.
- Because compute is not storable, spot prices **cannot** be translated into futures prices via
  standard cash-and-carry no-arbitrage logic — compute behaves more like electricity than oil,
  despite the popular "new oil" framing.
- Term rental contracts provide a no-arbitrage benchmark instead. Due to basis risk from physical
  access constraints, synthetic futures (forward prices implied by term rentals) are expected to
  be an **upper bound** on genuine compute futures prices — a gap expected to narrow as
  financialization and intermediation mature.
- In a formal futures market, prices will reflect investors' expectations of future spot compute
  prices, adjusted by a risk premium. If compute prices are negatively correlated with the
  marginal investor's marginal utility — plausible for compute providers as the dominant early
  hedgers — the risk premium should be **positive**.
- Empirically (not reproduced here — see Section 7 in the source), the authors build the first
  synthetic-futures return panel across GPU generations and maturities, and preliminary
  hold-to-maturity and constant-maturity return evidence is broadly consistent with a positive
  risk premium.
- Open questions flagged for future work: optimal compute index design pre-launch, how post-launch
  hedging pressure from providers vs. users will shape the risk premium, the eventual role of
  general financial participants, which risk factors will price compute risk, how the
  forward/futures wedge will evolve, and how the resulting forward curve will discipline
  investment in chips, data centers, and energy.
