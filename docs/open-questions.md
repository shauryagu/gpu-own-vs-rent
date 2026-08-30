# Open questions (through PR 6)

Binding resolutions live in
[`designs/2026-08-22-gate-0-1-mvp.md`](designs/2026-08-22-gate-0-1-mvp.md).
What the code does: [`status.md`](status.md). Names: [`vocab.md`](vocab.md).

Do not invent \(P\), \(r\), primary \(T\), or primary \(u\). Do not collapse
leftover \(L\) and salvage \(R^{\star}\). Do not add a paid Ornn key.

**Code after PR 6.** `collect --series current|daily|epoch` writes raw bodies.
`chi invert` on frozen daily-index fixtures prints \(L\) and \(R^{\star}\);
\(F(\theta)\) only with `--residual-cents`. `chi_log` is still an unlinked stub.

| Status | Meaning |
|---|---|
| **open** | Operational or implementation gap. |
| **closed-for-now** | Binding. Do not reopen without a design change. |
| **deferred** | Later gate; no silent default. |

---

## Closed in Gates 0–1

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
| Half-life never estimated; \(H=8760\); \(\pi=0\); PUE \(=1.0\) | closed |
| R1–R4 / `docs/project-plan.md` stay out | closed |
| No paid Ornn key on collect | closed |

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
| **2** | `chi_log`: `SourceFetched` / `SeriesParsed`, CAS payloads, `chi replay` twice → identical catalog. Collect stays file-based until a later increment. Invert does not read the log. |
| **3** | Cost-stack *panel*; PUE sweep; named LMP. Leftover \(L\) already exists. No PJM in this repo yet. |
| **4** | \(\Theta_L(S)\) and \(\Theta_{R^{\star}}(S)\) as two surfaces in `project`. |
| **5** | H100e maps beyond identity. No OLS on five GPUs. |
| **6** | `chi invert --as-of`. `AsOf` exists and is unused. |
| **7+** | Live marks, axum/SSE, venue panel, client. SQLite when as-of queries hurt. |

Purchase *band*, WACC, and used-market residual curves stay undeclared. Flags
exist; we will not invent the dollars.

---

## Answer-by

| When | Blocks |
|---|---|
| First live collect + launchd | Missed hours, free-list, no key in plist |
| Gate 2 | Event log before a published number is a research artifact |
| Gate 3 / 4 / 5 / 6+ | Deferred table |
