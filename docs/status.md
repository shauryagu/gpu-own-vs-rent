# Status (through PR 6)

What the code does after workspace scaffold (PR 1), hourly collector (PR 2),
domain newtypes (PR 3), NPV identity (PR 4), daily-index / Epoch ingest (PR 5),
and invert CLI (PR 6). On `main` at
[github.com/shauryagu/gpu-own-vs-rent](https://github.com/shauryagu/gpu-own-vs-rent).
Binary: `chi`.

Open / deferred questions: [`open-questions.md`](open-questions.md).
Binding spec: [`designs/2026-08-22-gate-0-1-mvp.md`](designs/2026-08-22-gate-0-1-mvp.md).
Names: [`vocab.md`](vocab.md).

---

## 1. What this is

Given public GPU rental prices and declared capital inputs, which assumptions
about life, use, and resale make **owning** a fair deal versus **renting** —
and how far that is from accounting (\(T=6\,\mathrm{y}\), \(R=0\))?

Fair means discrete NPV of buy-and-earn-rent is zero. Not an optimum. Not
“minimize basis risk.” One \(S\) fits many \(\theta\); that set is the result.

We **consume** Ornn OCPI. Bandi & Su: a GPU-hour is not storable, so
cash-and-carry fails ([`source/AiComputeAssetPricing.md`](source/AiComputeAssetPricing.md)
§5.1). Cite it. Do not re-derive it. They priced the service flow; we ask
about the hardware stock.

---

## 2. Locked decisions

| Decision | Why it stays |
|---|---|
| Two inverses: leftover \(L\) and salvage \(R^{\star}\) | Opposite signs in \(S\), different units. Never “implied residual.” |
| Invert \(S\) = latest **daily-index** | Teaching token `2.879583333333333`. Not current. Not history `2.88`. |
| Hourly current is collected anyway | Lost hours cannot be rebuilt from the free daily window. |
| No silent \(P\), \(r\), primary \(T\), primary \(u\) | Accounting \(T=6,R=0\) is a labeled overlay. |
| `--residual-cents` required for \(F(\theta)\) | Zero is declared, not omitted. |
| A100 SXM4 / RTX 5090 fail closed | Do not guess 40 vs 80 GB. |
| H100 SXM → H100e is identity only | Other SKUs are Gate 5. |
| Free public data only | A 401 on a “free” path is a bug, not a key. |
| Half-life never estimated | Not an MVP parameter. |
| Discrete annual NPV, \(H=8760\) | No root finder. |
| Energy \(\pi=0\), PUE \(=1.0\) | Product \(e\) is still evaluated. |
| Do not implement `docs/project-plan.md` | Seed reservation simulator. Out of scope. |

Architecture: `chi` → `ingest` + `project` → `domain`. `chi_log` unlinked
(Gate 2). `domain` has no I/O. One MVP trait: `ingest::HttpGet`. Blocking
`reqwest`, 15 s. Money is `Decimal`; `f64` is physics. Three serde scales:
ingest **source token**, domain **display** (USD 2, USD/GPU-hour 4), invert
JSON **`round_dp(12)`** on computed money. \(S\) in invert JSON is the source
token.

---

## 3. What runs today

| Command | Behavior |
|---|---|
| `chi --help` | `collect`, `invert`. |
| `chi collect` / `--series current` | Free-list + per-GPU current. Raw envelopes + `ocpi.hourly.v1` JSONL. |
| `chi collect --series daily` | Raw daily-index / all / history. No hourly JSONL. |
| `chi collect --series epoch` | Epoch `ml_hardware.csv` under `data/raw/epoch/`. |
| `chi invert` | Frozen `{fixture_dir}/ocpi/daily-index/{slug}.json`. Required `--purchase-cents --life-years --utilization --discount-rate`. Always prints \(L\) and \(R^{\star}\). \(F(\theta)\) only with `--residual-cents`. Accounting overlay \(T=6,R=0\). No HTTP, no `--data-dir`. |

Teaching H100 (`--gpu "H100 SXM" --fixture-dir fixtures --purchase-cents 2500000 --life-years 5 --utilization 0.60 --discount-rate 0.10`): \(S=2.879583333333333\), \(L\approx 1.6248\), \(R^{\star}\approx -52138\). Goldens are binary stdout, not the design sample.

**Not here:** event log / `chi replay`, SQLite, server, UI, \(\Theta\) sweep.

launchd: `scripts/ocpi-hourly.plist` + `scripts/install-ocpi-hourly.sh`.
Install on the **main** checkout’s `target/release/chi`, not a worktree.

---

## 4. Files

```
chi (binary)  -->  ingest  -->  domain
              -->  project -->  domain
chi_log (stub, unlinked)
```

| Path | Role |
|---|---|
| `crates/chi/src/main.rs` | `Cmd { Collect, Invert }`. |
| `crates/chi/src/collect.rs` | `--data-dir`, `--series`. Injects `now_utc()`, `LiveHttp`. |
| `crates/chi/src/invert.rs` | Daily-index wrapper → leftover + implied salvage; optional fair rent. |
| `crates/chi/tests/invert_fixture.rs` | Seven CLI tests + binary goldens. |
| `crates/project/src/lib.rs` | Thin `NamedInverses { leftover, implied_salvage }`. Algebra stays in `domain`. |
| `crates/domain/src/identity.rs` | `capital_rent`, `fair_rent`, `leftover`, `implied_salvage`. |
| `crates/ingest/src/ocpi_daily.rs` | Wrapper → `ObservedSpot` (`OcpiDailyIndex` only). |
| `crates/ingest/src/epoch.rs` | H100 → `NVIDIA H100 SXM5 80GB` TDP 700 W. Release price is annotation. |
| `fixtures/ocpi/daily-index/H100_SXM.json` | Invert \(S\). |
| `fixtures/ocpi/daily-index/A100_SXM4.json`, `RTX_5090.json` | Synthetic wrappers so Epoch fail-closed is reachable. Not teaching \(S\). |

Invert does not open `daily-index-all`, `daily-history`, `current`, or `data/`.

---

## 5. Next

**Gate 2** — `chi_log`: `SourceFetched` / `SeriesParsed`, content-addressed
payloads, `chi replay` twice → byte-identical ingest catalog. Fixture log
only at first; collect unchanged. Invert still does not read the log.

Do not start Gate 3–10 or the seed simulator from this snapshot.

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --workspace
cargo run -p chi -- invert --gpu "H100 SXM" --fixture-dir fixtures \
  --purchase-cents 2500000 --life-years 5 --utilization 0.60 \
  --discount-rate 0.10 --format text
```
