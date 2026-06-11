# Risk Analysis: CRM Selection for Brightline (Dublin, 40-person ad agency)

Role: risk-analyst | Flow: flow-07-parallel-adhoc-guided | Date: 2026-06-10
Assumptions: knowledge-only (no web access); vendor capabilities as of my training data (early 2026). Brightline is an Irish data controller subject to GDPR; client and prospect personal data will live in the CRM; ~40 seats; no in-house DPO or large IT team.

## GDPR posture and data residency

- **Salesforce**: Strongest formal posture. EU data centres (incl. Frankfurt, Paris; Hyperforce EU Operating Zone offers EU-only data processing and EU-based support access). Mature DPA, SCCs, processor binding corporate rules, granular audit trails. Caveat: Hyperforce EU residency guarantees can require specific editions/add-ons; default provisioning may still route some support/telemetry outside the EU. Severity of residual risk: low.
- **HubSpot**: Offers an EU data hosting option (Frankfurt, AWS eu-central-1) selectable at portal creation - it cannot easily be changed later, which is itself a migration risk if the portal is created in the US region by mistake. US parent company means Schrems II / transfer-impact-assessment exposure for any US-touching subprocessors (HubSpot relies on the EU-US Data Privacy Framework). Good DPA and cookie/consent tooling (useful for an ad agency). Residual risk: low-medium.
- **Zoho**: Operates EU data centres (Amsterdam, Dublin) and lets you pick the EU DC at signup. Privacy-forward positioning (no ad-funded model). However, its compliance documentation, subprocessor transparency, and audit-report availability (SOC 2/ISO scope per service) are less mature than Salesforce/HubSpot, and group entities are headquartered in India/US - TIA work falls on Brightline. Residual risk: medium.

## Vendor lock-in and exit costs

- **Salesforce**: Highest lock-in. Proprietary metadata model, Apex code, and ecosystem (AppExchange) make exit expensive; weekly data export is free but attachments/metadata reconstruction is laborious. Multi-year contracts with auto-renewal uplift clauses (~7% typical) and aggressive non-cancellation terms. Exit cost estimate: high (consultant-assisted, 2-4 months).
- **HubSpot**: Medium lock-in. Good native exports (CSV, API), but workflows, reporting, and marketing assets do not port. Annual prepay is standard; downgrades only at renewal. Exit cost: moderate (4-8 weeks).
- **Zoho**: Lowest lock-in. Monthly billing available, full data export, modest customisation surface. Exit cost: low-moderate.

## Migration and adoption risk

All three import CSV/contacts easily; the real risk is process migration (pipelines, ad-ops integrations, email sync). Salesforce carries the highest implementation risk for a 40-person agency with no admin: industry experience says SMB Salesforce projects routinely need a paid partner (EUR 10-30k) and a part-time admin thereafter, or adoption stalls. HubSpot has the lowest adoption risk (consumer-grade UX, strong onboarding); Zoho sits between - cheap but UX inconsistency across its module sprawl historically drags adoption and increases shadow-spreadsheet risk.

## Hidden-cost risk

- Salesforce: add-ons (CPQ, Marketing Cloud Account Engagement, API limits, sandboxes, premier support 30% uplift) routinely double the headline per-seat price; renewal uplifts.
- HubSpot: marketing contacts tiering and Hub bundling - costs jump sharply when you cross contact tiers or need Marketing Hub Professional alongside Sales Hub; onboarding fees are mandatory on Professional+.
- Zoho: cheapest headline, but Zoho One bundle upsell, per-user add-ons, and paid support tiers; main hidden cost is internal time spent configuring and working around rough edges.

## Support risk

Salesforce standard support is slow without paid Premier; partner-dependent. HubSpot includes phone/chat support on Professional with generally good SLAs and EU-hours coverage. Zoho support is the weakest of the three (ticket latency, escalation friction), partly offset by low cost and a Dublin presence.

## Risk register (summary)

See risk-analyst-data.json for the structured register. Top three risks overall: (1) Salesforce cost/complexity overrun for a 40-seat agency, (2) HubSpot contact-tier and bundling cost creep, (3) Zoho support and compliance-documentation gaps.

## Recommendation (risk lens only)

**HubSpot**, EU (Frankfurt) data residency selected at portal creation, annual Sales Hub Professional, with a written transfer impact assessment covering US subprocessors and a quarterly export of all CRM data to Brightline-controlled storage as an exit hedge. It carries the best balance: acceptable GDPR posture, low adoption risk for a non-technical agency, and moderate, manageable lock-in. Salesforce is over-scaled and over-priced in risk-adjusted terms for 40 seats; Zoho is the fallback if budget dominates and the team accepts higher support/compliance friction.

Confidence: 4/5 (vendor terms and DC footprints can shift; verify Hyperforce/EU-DC specifics and current DPF status before contract signature).
