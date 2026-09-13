# Open questions (through Gate 3)

Binding resolutions live in
[`designs/2026-08-22-gate-0-1-mvp.md`](designs/2026-08-22-gate-0-1-mvp.md).
What the code does: [`status.md`](status.md). Names: [`vocab.md`](vocab.md).

Do not invent \(P\), \(r\), primary \(T\), or primary \(u\). Do not collapse
leftover \(L\) and salvage \(R^{\star}\). Do not add a paid Ornn key.

**Code after Gate 3.** `collect --series current|daily|epoch` writes raw bodies.
`chi invert` on frozen daily-index fixtures prints \(L\) and \(R^{\star}\);
\(F(\theta)\) only with `--residual-cents`. Omit \(\pi\) on invert ⇒ leftover
includes power. `chi stack` requires named \(\pi\) (LMP fixture or manual
source) and prints \(S = F_{\mathrm{capital}} + e + L\) on the default PUE
grid 1.0 / 1.2 / 1.5 (or declared `--pue`). `chi replay` folds a fixture log
to an ingest catalog (`ocpi.current`). Collect stays file-based. Invert does
not read the log. Replay is hourly current, not invert \(S\).

| Status | Meaning |
|---|---|
| **open** | Operational or implementation gap. |
| **closed-for-now** | Binding. Do not reopen without a design change. |
| **deferred** | Later gate; no silent default. |

---

## Closed in Gates 0–3

| Item | Status |
|---|---|
| Two inverses, distinct JSON keys `leftover_usd_per_gpu_hour` / `implied_salvage_usd` | closed — PR 4 algebra, PR 6 CLI |
| Fair = NPV \(=0\); \(F(\theta)\) only with `--residual-cents` (0 is present) | closed — PR 6 |
| Accounting overlay labeled \(T=6,R=0\), not primary \(\theta\) | closed — PR 6 |
| Cite Bandi & Su §5.1; \(F(\theta)\) is not \(F_t(T)\) | closed — comments + invert copy |
| Calculator, not an exchange or competing index | closed |
| Invert \(S\) = `ocpi/daily-index/{slug}.json` only; teaching token kept | closed — PR 6 |
| A100 SXM4 / RTX 5090 fail closed at Epoch, not only missing file | closed — PR 6 |
| Invert never reads `data/` or `--data-dir`; goldens from the binary | closed — PR 6 |
| Required `--purchase-cents --life-years --utilization --discount-rate` | closed — PR 6 |
| \(S\) JSON is source token; computed money `round_dp(12)` | closed — PR 6 (energy JSON still Display `"0"` — nit, not a new question) |
| Invert prints `valid_on` and wrapper `fetched_at`; no `--as-of` | closed — print; query is Gate 6 |
| Half-life never estimated; \(H=8760\); invert omit \(\pi\) ⇒ leftover includes power | closed |
| R1–R4 / `docs/project-plan.md` stay out | closed |
| No paid Ornn key on collect | closed |
| Event log + `chi replay` twice → byte-identical ingest catalog (`ocpi.current`). Collect file-based. Invert does not read the log. | closed — Gate 2 |
| Cost-stack *panel* \(S = F_{\mathrm{capital}} + e + L\); `project::CostStack`; `chi stack` | closed — Gate 3 |
| Named \(\pi\): PJM RTO LMP fixture **or** manual USD/kWh + source; stack fails closed without it | closed — Gate 3 |
| Default PUE grid 1.0, 1.2, 1.5; repeatable `--pue` replaces the grid | closed — Gate 3 |
| Accounting overlay useful life **6 years** cites CoreWeave FY 2025 10-K (CIK 0001769628, filed 2026-03-02) — docs only; code overlay stays labeled \(T=6,R=0\) | closed — Gate 3 docs |

---

## Still open (ops)

### Missed hours

Hourly current cannot be reconstructed. launchd does not retry a failed :05
run. Do not interpolate daily-index into `ocpi.current`.

### First live collect / launchd trust

Plist and install script ship. Point the agent at main’s
`target/release/chi`, not a worktree. Rollback is `launchctl unload`; do not
truncate JSONL.

### Free-list vs fixtures

A 401 on a “free” path is an allow-list bug. Confirm live `/api/gpu-types-free`
still matches the fixture names when this machine is the collector.

---

## Deferred

| Gate | What |
|---|---|
| **4** | \(\Theta_L(S)\) and \(\Theta_{R^{\star}}(S)\) as two surfaces in `project`. **Next** after Learn before. |
| **5** | H100e maps beyond identity. No OLS on five GPUs. |
| **6** | `chi invert --as-of`. `AsOf` exists and is unused. |
| **7+** | Live marks, axum/SSE, venue panel, client. SQLite when as-of queries hurt. |

Purchase *band*, WACC, and used-market residual curves stay undeclared. Flags
exist; we will not invent the dollars.

---

## Usability follow-ups

Parked from the Gate 3 CLI review. **Not a gate.** Do not restyle invert
teaching goldens unless we accept golden churn. Buyer path is `chi stack`.

**Shipped in Gate 3 — do not redo.** `chi stack` PUE table with \(e/S\);
named \(\pi\) fail-closed; `chi --help` about-lines; invert omit \(\pi\) still
means leftover includes power; `--purchase-cents` stays cents (no dollar
default).

| Item | Constraint | Possible later |
|---|---|---|
| Cents typo (`25000` vs `2500000`) | Keep `--purchase-cents`. No silent \(P\). | Echo declared \(P\) as USD on stdout; help examples. Not `--purchase-usd`. |
| Invert silent-zero \(\pi\), one PUE | Teaching default. Stack is the panel. | Louder invert note that omitted \(\pi\) folds power into leftover \(L\). Churns goldens. |
| Invert punchline is late (metadata first) | Frozen transcript. | Reorder only if we accept golden churn. |
| `--residual-cents` tautology note | \(F(\theta)\) only with the flag. | Shorter copy. Churns goldens. |
| Replay stdout | Compact catalog JSON; series is hourly current (`"2.63"`), not invert \(S\). | `--format text` for “what did I collect?” |
| `justfile` wrapping cargo vs Compose | Considered; Compose one-shots exist. | Optional host wrapper. Not a second product. |

**Parked nits (code, not UX).** Mix `--lmp-fixture` with manual energy flags
already bails, untested. `default_pue_grid` test name omits 1.0. Mixed
`designs/` vs `docs/designs/` hrefs in older docs. `parse_index_value`
bad-token still says `index_value`. Empty `pues` at the library yields Ok
zero rows (CLI never passes empty).

---

## Answer-by

| When | Blocks |
|---|---|
| First live collect + launchd | Missed hours, free-list, no key in plist |
| Learn before Gate 4, then TDD | Deferred Gate 4 row |
| Usability follow-ups | Not on the Gate 4 critical path |
