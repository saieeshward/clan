
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

## 10-procurement-reviewer

procurement_lead_time_weeks: 3
procurement_contract_term_years: 1
procurement_key_commitments:
- Annual subscription invoiced upfront; 30-day cancellation notice required post-term
- EU data residency addendum (Zoho included, HubSpot add-on) must be signed before data migration
- SLA: 99.9% uptime guarantee with service credit clause
- Quarterly data export right (Zoho) — contractually enforceable per competitive-intel finding
- User-count true-up clause: additional seats billed pro-rata within term
procurement_negotiation_leverage:
- 3-year commitment offer unlocks ~15% volume discount from both Zoho and HubSpot
- October peak deadline creates urgency leverage to demand faster onboarding credits
- Competing vendor quote (Salesforce eliminated) can be presented to drive Zoho/HubSpot concessions
- 40-seat deal is above Zoho's mid-market threshold; dedicated CSM can be negotiated at no cost
procurement_risks:
- risk: Auto-renewal clause locks Brightline into a second year if not cancelled 30 days prior
  severity: Medium
  mitigation: Diary reminder at month 10; legal review of auto-renewal language before signing
- risk: HubSpot EU-pinning add-on cost not yet quoted; may erode pricing advantage
  severity: Medium
  mitigation: Require firm written quote for EU add-on before contract execution
- risk: True-up clause could raise costs if headcount grows beyond 40 seats
  severity: Low
  mitigation: Negotiate a 10% headcount buffer at current per-seat rate
procurement_export_clause_feasible: true
