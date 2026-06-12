## 01-market-researcher

market_tam_eur: 2400000
market_segment: EU mid-market logistics
market_growth_pct: 12
market_incumbent_share_HubSpot: 34
market_incumbent_share_Zoho: 21
market_incumbent_share_Salesforce: 29
market_buyer_priority_1: time-to-value
market_buyer_priority_2: EU data residency
market_buyer_priority_3: integration depth

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

## 10-lead-partner

rec_vendor: Zoho CRM
rec_year1_cost_eur: 18240
rec_3yr_cost_eur: 54720
rec_savings_vs_hubspot_3yr: 17280
rec_go_live_target: 2026-09-15
rec_peak_season_safe: true
rec_top_risk_1: migration data quality (High — mitigate with dedicated data owner pre-migration)
rec_top_risk_2: Zoho workflow maturity (Medium — validate advanced workflow requirements in pilot)
rec_top_risk_3: user adoption (Medium — champion programme + 24 h training)
rec_gdpr_note: EU region pinning included at no extra cost; DPA in place
rec_integration_note: NetSuite via supported connector; all other 5 integrations native; 9 wk effort
rec_rationale: Zoho wins on all three buyer priorities — lowest year-1 cost, EU residency included, and sufficient integration depth. HubSpot's superior UI does not justify EUR5,760 annual premium for a 40-seat logistics team at this stage. Salesforce eliminated on cost.
rec_confidence: High
