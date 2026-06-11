# Risk Analysis — Brightline CRM Evaluation (risk-analyst)

Scope: GDPR/data residency, lock-in and exit cost, migration risk, hidden-cost risk, support risk for Salesforce, HubSpot, Zoho CRM. Knowledge-based (no web access); facts reflect vendor postures as of early 2026 and should be re-verified at contract time. Brightline = 40-person ad agency, Dublin, Ireland (EU data subject base, likely client PII in pipelines).

## 1. GDPR posture and data residency (EU hosting)

- **Salesforce**: Mature GDPR program, standard DPA with SCCs, Binding Corporate Rules. EU hosting available (EU instances incl. Frankfurt/Paris; "Hyperforce EU Operating Zone" offers EU residency with EU-based support access controls — but it is a paid premium tier). Risk: US CLOUD Act exposure as a US parent company remains, mitigated contractually, not eliminated.
- **HubSpot**: Offers an EU data centre (Frankfurt, AWS eu-central-1) selectable **only at portal creation** — you cannot migrate an existing US-hosted portal in place. GDPR-aligned DPA, SCCs, cookie/consent tooling built in (useful for an agency). Same US-parent caveat.
- **Zoho**: EU data centres (Amsterdam/Dublin) selectable at signup; Zoho operates its own DCs and has a comparatively strong privacy stance (no ad-based revenue). DPA and SCCs available. Zoho Corp is India-headquartered with US presence; GDPR adequacy for India does not exist, so the EU-DC choice matters.

Assumption: Brightline processes client contact data and campaign performance data, not special-category data; all three vendors are adequate **if** the EU region is selected at provisioning. Action item for rollout plan: provision in EU region on day one — this is irreversible-ish for HubSpot.

## 2. Vendor lock-in and exit costs

- **Salesforce**: Highest lock-in. Proprietary metadata model, Apex code, AppExchange dependencies; annual contracts with auto-renew and historically rigid non-cancellation terms; data export is free (weekly export) but rebuilding automations elsewhere is expensive. Exit cost: high (consulting-heavy).
- **HubSpot**: Moderate. Good APIs and CSV/API export of CRM objects; workflows and reporting are proprietary but simpler to re-implement. Annual prepay common; seat-tier changes mid-term are upward-only. Exit cost: moderate.
- **Zoho**: Lowest contractual lock-in (monthly billing possible, low per-seat cost), full data export via Zoho Backup/API. Functional lock-in grows if Brightline adopts the wider Zoho One suite. Exit cost: low–moderate.

## 3. Migration risk (into the chosen CRM)

Assumption: Brightline migrates from spreadsheets/a lightweight tool, not a legacy CRM. Inbound migration risk is therefore mostly data hygiene and adoption, not ETL complexity. Salesforce has the longest time-to-value (typically needs a partner; 2–4 months realistic). HubSpot is the fastest to stand up (days–weeks). Zoho is fast but configuration quality depends heavily on who sets it up; its flexibility invites messy implementations.

## 4. Hidden-cost risk

- **Salesforce**: Severe. Per-seat list price is only the start: API call limits, storage overages, Marketing Cloud/CPQ add-ons, mandatory partner/admin cost (~0.5 FTE or retainer), and premier support (~30% uplift) commonly double the headline price.
- **HubSpot**: Significant but legible: marketing-contact tiering can balloon costs for an agency holding large prospect lists; onboarding fees are mandatory on Pro tiers; key features gate at Professional/Enterprise.
- **Zoho**: Mild: low headline price, but Zoho One upsell, and productivity loss from rough UX edges is the real hidden cost.

## 5. Support risk

Salesforce standard support is slow without paid Premier; strong partner ecosystem in Dublin. HubSpot support is included (phone/chat on paid tiers) and generally rated well for SMBs; decent EMEA coverage. Zoho support is the weakest link — basic support is email-first and slow; paid premium support recommended; thinner EU partner network.

## Risk register

| # | Risk | Vendor | Severity (1-5) | Mitigation |
|---|------|--------|----------------|------------|
| 1 | US CLOUD Act / third-country transfer exposure despite EU hosting | All (SF, HubSpot US parents; Zoho IN parent) | 3 | Sign DPA+SCCs, select EU region, document transfer impact assessment; avoid storing sensitive client data in CRM free-text |
| 2 | Portal created in US region; EU residency unrecoverable without rebuild | HubSpot | 4 | Provision EU (Frankfurt) portal explicitly at signup; verify region before any data import |
| 3 | Cost overrun from add-ons, support uplift, admin/partner retainer | Salesforce | 4 | Fixed-scope implementation contract; cap add-ons in year 1; budget 1.5–2x list price |
| 4 | Marketing-contact tier costs scale with agency prospect lists | HubSpot | 3 | Mark non-active contacts as non-marketing; quarterly list hygiene; negotiate contact tier at signature |
| 5 | Multi-year auto-renew / rigid cancellation terms | Salesforce | 4 | 1-year initial term, negotiate renewal caps and termination-for-convenience clause |
| 6 | Slow/low-quality support stalls adoption at a 40-person firm with no CRM admin | Zoho | 3 | Buy premium support tier; engage local Zoho partner for first 6 months |
| 7 | Implementation over-runs and time-to-value slip (partner-dependent build) | Salesforce | 4 | If chosen, fixed-price QuickStart package; phase scope ruthlessly |
| 8 | Functional lock-in via proprietary workflows/automations | All (worst: Salesforce) | 3 | Document processes outside the tool; prefer native features over custom code; keep integrations API-standard |
| 9 | Messy self-service configuration creates data-quality debt | Zoho, HubSpot | 2 | Assign internal CRM owner; enforce required fields and dedupe rules from day one |
| 10 | Suite creep (Zoho One / HubSpot hubs) deepens exit cost over time | Zoho, HubSpot | 2 | Annual vendor review; adopt extra modules only with documented exit path |

## Risk-adjusted verdict

Salesforce carries the highest aggregate risk for a 40-person agency (cost, lock-in, implementation). Zoho is cheapest but has the weakest support and execution risk. HubSpot is the best risk-adjusted fit: fast time-to-value, EU data centre, legible pricing — provided the EU portal-region and marketing-contact risks (register #2, #4) are mitigated at signup.

**recommended_vendor: HubSpot** — **confidence: 4/5**
