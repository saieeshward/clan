# Handoff: risk-analyst -> next agent (hop 2 of 3)

Risk findings are in document data (flat keys: `risk_summary`, `gdpr_data_residency_risk`, `vendor_lockin_exit_costs`, `migration_risk`, `hidden_cost_risk`, `support_risk`, `risk_register` (13 entries: risk/vendor/severity 1-5/mitigation), `risk_adjusted_recommendation_input`, `risk_assumptions`).

Key risk numbers to fold into the costed recommendation:
- Salesforce realistic year-2 TCO: EUR 95-110k/yr (vs 79.2k list) once admin/partner (10-25k), connectors (1-5k), and 7-10% renewal uplifts land. Severity-5 hidden-cost risk + sev-4 adoption and lock-in (exit EUR 15-30k). Treat as eliminated at 40 seats.
- HubSpot: mixed-seat 19.44k is sound IF governance holds - seat creep to 43.2k (sev 3) and Marketing Hub contact-tier creep (sev 4) are the watch items. CRITICAL rollout step: select EU (Frankfurt) DC at portal creation - irreversible later. Dublin EMEA HQ = support advantage. Exit cost EUR 8-15k.
- Zoho: 19.2k honest pricing; add EUR 2-6k one-off integration glue + year-1 premium support (~20-25% of licence). Sev-3 support risk needs a named internal champion. Dublin/Amsterdam DCs = best residency story. Exit EUR 5-10k (CRM only; avoid Zoho One bundle entanglement in year 1).
- Universal additions to TCO: ETL layer EUR 2.4-9.6k/yr, 15% price contingency (knowledge-cutoff prices), dedupe-before-import, joint-controller paperwork for ad-audience syncing.

GDPR: no blocker anywhere; all three offer EU hosting + Art. 28 DPAs + SCCs. Residual US transfer risk is low (SCCs + DPF); Brightline must update RoPA and document consent basis for ad-audience syncing.

Risk-adjusted ranking: 1) HubSpot mixed-seat, 2) Zoho CRM Enterprise, 3) Salesforce (drop).

For the final agent: build the single costed recommendation + phased rollout plan (pilot pod of 5-8 users first - it mitigates migration risk on any vendor) and produce the full-html deliverable per the brief's design requirements. Bake the mitigations above into the rollout phases (EU DC selection day 1, seat-governance policy, contact pruning cadence, champion nomination).
