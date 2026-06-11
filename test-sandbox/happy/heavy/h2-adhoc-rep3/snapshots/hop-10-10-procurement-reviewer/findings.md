
## 01-market-researcher

market_tam_eur: 2400000
market_segment: EU mid-market logistics
market_growth_pct: 12
market_incumbent_share.HubSpot: 34
market_incumbent_share.Zoho: 21
market_incumbent_share.Salesforce: 29
market_buyer_priority: time-to-value, EU data residency, integration depth

## 02-pricing-analyst

pricing_hubspot_seat: 50
pricing_zoho_seat: 38
pricing_salesforce_seat: 75
pricing_seats: 40
pricing_hubspot_annual: 24000
pricing_zoho_annual: 18240
pricing_salesforce_annual: 36000
pricing_cheapest: Zoho

## 03-risk-analyst

risk_top: migration data quality, user adoption, integration latency
risk_migration_sev: High
risk_adoption_sev: Medium
risk_zoho_maturity_sev: Medium
risk_hubspot_maturity_sev: Low

## 04-gdpr-reviewer

gdpr_residency_required: EU (Frankfurt)
gdpr_hubspot_eu_pinning: add-on
gdpr_zoho_eu_pinning: included
gdpr_dpa_both: true
gdpr_subprocessor_notice_days: 30

## 05-integrations-assessor

integ_required: NetSuite, Slack, Twilio, DocuSign, Outlook, Shopify
integ_hubspot_native: 5
integ_zoho_native: 4
integ_zoho_gap: NetSuite (via connector)
integ_effort_weeks: 9

## 06-customer-discovery

cust_interviews: 6
cust_top_need: lead response < 2h
cust_adoption_target_pct: 90
cust_champion_count: 4
cust_training_hours: 24
cust_pref_lean: Zoho on cost, HubSpot on UI

## 07-competitive-intel

comp_hubspot_strength: marketing suite
comp_zoho_strength: price + EU residency
comp_salesforce_strength: enterprise depth
comp_switching_cost_zoho: low
comp_zoho_export_clause: true

## 08-finance-modeler

fin_zoho_y1_total: 18240
fin_hubspot_y1_total: 24000
fin_zoho_3yr: 54720
fin_hubspot_3yr: 72000
fin_savings_zoho_3yr: 17280
fin_payback_months: 7
fin_recommend_lean: Zoho

## 09-rollout-planner

rollout_phase1_date: 2026-07-01
rollout_phase2_date: 2026-09-15
rollout_phase3_date: 2026-11-01
rollout_go_live_before_peak: true
rollout_weeks: 9
rollout_champion_program: true

## 10-procurement-reviewer

procurement_lead_time_weeks: 3
procurement_contract_term_years: 1
procurement_key_commitments: ["EU data residency SLA (99.9% uptime, Frankfurt region)", "DPA signed before go-live", "Quarterly data export in CSV/JSON per Zoho export clause", "Named customer success manager for onboarding period", "Renewal price cap: max 5% increase at end of year 1"]
procurement_negotiation_leverage: ["40-seat deal is mid-market floor; bundle multi-year discount request", "Competitor quote from HubSpot as walk-away leverage", "October peak deadline creates urgency — vendor is incentivised to close quickly", "Zoho switching cost is low (export clause confirmed), so Zoho will compete on price to lock in"]
procurement_risks: [{"risk": "Auto-renewal lock-in if notice period missed", "severity": "Medium", "mitigation": "Set calendar reminder 90 days before renewal; negotiate 60-day notice window in contract"}, {"risk": "Price escalation on renewal beyond CPI", "severity": "Low", "mitigation": "Negotiate explicit annual price cap clause (max 5%) at signing"}, {"risk": "Vendor invoices in USD creating FX exposure", "severity": "Low", "mitigation": "Request EUR-denominated invoicing or agree fixed EUR rate at contract signature"}]
procurement_export_clause_feasible: true
