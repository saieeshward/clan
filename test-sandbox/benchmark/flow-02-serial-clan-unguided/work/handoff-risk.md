

## Handoff: risk-analyst → lead-partner (2026-06-10)

Stage 2 complete. `shared/data.yaml` now holds `risk_analysis` (gdpr_posture, lock_in_and_exit, migration_risk, hidden_cost_risk, support_risk, a 14-entry risk_register with severity 1-5 + mitigations, and ranking_impact). The human view gained a styled "Stage 2 · Risk Analysis" section appended inside `div.wrap` (after the stage-1 verdict).

Key findings for the final recommendation:
1. Ranking HubSpot > Zoho > Salesforce SURVIVES the risk stress-test. HubSpot's worst risks (marketing-contact tier creep sev-4, renewal auto-uplift) are contract-mitigable; GDPR (Frankfurt) and Dublin-based support are the best fit.
2. Non-negotiable contract terms to bake into your recommendation: (a) HubSpot EU residency selected AT portal creation — cannot be moved later; (b) written contact-tier pricing schedule; (c) renewal uplift cap; (d) pre-contract verification of all Jan-2026 list prices (sev-3, applies to every vendor).
3. Salesforce composite risk is highest (adoption 4, cost-creep 4, lock-in 3) — only defensible if Brightline needs Data Cloud audience activation now. Zoho viable as budget fallback but carries EUR 5-10k/yr middleware glue, MODERATE GDPR diligence (India parent, SCC-reliant), and thin Irish support.
4. Open for you (hop 3): single costed recommendation (suggest 3-yr TCO view, incl. marketing-contact tier growth scenario), phased rollout plan, and the pre-contract verification gate as phase 0.

Assumptions: knowledge cutoff Jan 2026; DPF assumed valid but litigation-exposed; Brightline data = B2B contacts only (no special-category data); 40 seats as modelled in stage 1.
