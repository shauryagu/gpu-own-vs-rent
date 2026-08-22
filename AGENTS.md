# AGENTS.md

## Project

A reservation-pricing simulator for perishable compute capacity, delivered as a
working full-stack system over a 7-day build. Compute (GPU rental) is the live
case study; the subject is capacity control under uncertainty.

Full spec: `docs/project-plan.md`
Read-order and rationale for source material: `docs/reading-list.md`

**Standing constraint: free, publicly available data only.** No paid feeds, no
purchased history, no private venue data. This shapes the method (sweep
parameters rather than estimate them) — do not propose paid data sources as a
fix for a modeling gap.

## Setup / test / build commands

<!-- Fill in once scaffolded on Day 1. Placeholder shape below. -->
- Install: `TODO`
- Dev server: `TODO`
- Test: `TODO`
- Lint: `TODO`
- DB migrate: `TODO`

## Architecture (high level)

```
React + TS client  <-- SSE (clock, bids, marks) / REST -->  API layer
                                                                  |
                                              Simulation core (clock, ledger,
                                              pricing engine) + Job runner (sweeps)
                                                                  |
                                                        Postgres (append-only
                                                        event log + projections)
                                                                  ^
                                              Ingest (free sources, cached to disk)
```

- **Simulation clock is server-authoritative.** The client never advances time;
  it renders state and issues commands. This is a hard invariant, not a
  convenience — do not add client-side time advancement for any reason,
  including tests or demos.
- **Event log is the source of truth.** A run is fully determined by its seed
  plus its decision events. Any new state must be derivable by replay, not
  stored as an independent mutation path.
- **Ledger invariants are enforced in the database**, not just application
  code (exclusion constraints or serializable transactions + optimistic
  versioning). See `docs/project-plan.md` §3 for the five invariants.

## Domain references — consult before implementing

This project translates results from three papers directly into code. Before
touching the pricing engine, the ledger's whole-node/group-bid logic, or the
hedge layer, load the `research-references` skill — it maps specific paper
sections to specific parts of this codebase, so you're implementing a cited
result rather than re-deriving one.

- Pricing regimes R1–R4, opportunity cost, bid-price logic → Talluri & van Ryzin
- Whole-node / group-bid formulation, DLP engine, stranding cost →
  network revenue management paper, §1 and §3.1 particularly
- Synthetic futures curve, why cash-and-carry fails for compute, risk premium
  sign → Bandi & Su

Do not re-derive or "improve on" these results from first principles without
flagging it — the project's stated positioning is that it applies an
established result to an open operator-side question, not that it discovers
new theory.

## Non-negotiables

- Free data sources only — see `docs/llms.txt` for the approved list.
- Whole-node allocations are indivisible when `divisible = false` on a
  deployment. Do not silently allow fractional-node fills.
- Accepted bids are immutable (ledger invariant 5).
- The half-life parameter (price-process mean reversion) is swept, never
  point-estimated from the ~90-observation OCPI window. See
  `docs/project-plan.md` §5.3 before writing any calibration code.
- Bid-price schemes are known to be non-optimal for network/bundle requests
  (see network RM paper §3). Do not present R3/R4 as globally optimal in
  comments, docs, or UI copy — "near-optimal heuristic" is the accurate frame.

## Do not

- Do not add paid API keys or purchased datasets to the ingest layer.
- Do not implement continuous client-side clock advancement.
- Do not collapse the four pricing regimes into one "best" regime — the
  comparison across all four, on identical seeded bid streams, is the point
  of the project (see D3, D9 in `docs/project-plan.md` §1).
- Do not remove the parameter-sweep job (see project-plan.md §8, "never cut"
  list) even under time pressure — fall back to precomputed sweep results
  before dropping the sweep itself.

## When context is missing

If a task requires a data source, threshold, or modeling choice not covered
in `docs/project-plan.md`, `docs/reading-list.md`, or the `research-references`
skill, stop and ask rather than inventing a plausible-sounding default —
several parameters here (cost basis, basis error range, half-life) are
deliberately swept because they're not known, and a silently invented point
value will misrepresent that.
