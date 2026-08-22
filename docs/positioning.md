# Positioning

**Question.** Given public rental indices (OCPI) and public capital, spec, and energy inputs, what *feasible set* of (economic life, utilization, residual) makes owning a GPU a fair deal versus renting it — and how far is that set from accounting practice?

**Feasible set (not geography).** Not a data-center region. The free OCPI series are global. The object is \(\Theta(S)\): the set of parameter vectors \(\theta = (T, u, R)\) (life, utilization, residual) such that a forward GPU-hour identity \(F(\theta)\) matches observed OCPI \(S\) within a declared tolerance. Many \(\theta\) fit one \(S\). That non-uniqueness is the result, not a failure to point-estimate.

**Computer-science object.** Compute and keep that inverse correct over a streaming, heterogeneous, unit-incompatible ingest log. Every published number is `event log ⊕ parameter vector ⊕ code version`.

## Next to Bandi & Su

Bandi & Su (arXiv:2607.12156) already showed that a GPU-*hour* is not storable, so cash-and-carry from spot to futures fails, and that synthetic futures from term rentals upper-bound true futures. This project does not re-derive that.

They priced the *service flow*. We ask about the *hardware stock* that produces it: purchase price, economic life, utilization, residual, energy. The service is perishable; the chip is not. That is the operator/capital question their paper leaves open, stated so it can be computed from public data.

## Next to Ornn

Ornn’s path is a transaction-based index (OCPI) → cash-settled futures on that index → residual-value products → a venue that transfers compute risk.

We **consume OCPI**. We do not publish a competing benchmark. List prices (Lambda, RunPod, AWS) stay in a separate panel so offer vs cleared trade is visible.

Their *Basis Gap* article is the vocabulary: basis = what a contract guarantees − what the position actually is. Cash-settled OCPI futures hedge a generic index; a provider still holds specific chips. \(\Theta(S)\) is the set of life/utilization/residual assumptions under which that physical book is fairly priced versus the index. Outside the set, the cash hedge leaves residual risk — the quantity a later clearinghouse would haircut, and the quantity their residual-value product would cover.

**Exchange-like future, without building an exchange.** Listed commodities need a settlement index, a conversion factor (heterogeneous GPUs → a common unit, analogous to Treasury conversion factors), and a residual/basis model. Those three are the kernel of matching and variation margin. We implement the kernel as a typed, replayable system. We do not implement a CLOB, ICE connectivity, or KYC. Matching is known CS; the instrument is not.

## Constraints

- Free public data only. No paid Ornn history, no Silicon Data, no private venue tape.
- Do not estimate OCPI mean-reversion / half-life from ~90 daily points. Sweep unknowns.
- Purchase prices are bands, not MSRPs. Residuals are swept; sporadic used listings are anchors, not a curve. Utilization is unobserved.
- Bid arrivals are not public. A reservation-price simulator is out of scope until this cost stack exists and is worth deciding on.

## Collector

Hourly OCPI current prices cannot be reconstructed later. From the repo root:

```bash
cargo run -p chi -- collect
```

Appends one JSONL record per free GPU to `data/ocpi-hourly.jsonl`. Schedule that command hourly (cron or `scripts/ocpi-hourly.plist`). Raw HTTP bodies are stored under `data/raw/`.
