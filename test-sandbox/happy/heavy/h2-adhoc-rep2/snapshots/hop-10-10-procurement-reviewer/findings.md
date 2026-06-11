
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
procurement_key_commitments:
  - Quarterly data-export right (Zoho standard clause — confirmed by comp-intel)
  - EU data residency pinning included in Zoho base price, no add-on
  - 30-day subprocessor change notice in DPA (both vendors)
  - Named customer-success contact for onboarding (negotiate into MSA)
  - SLA uptime ≥ 99.5% with defined credits in contract
procurement_negotiation_leverage:
  - 40-seat deal is borderline mid-market — modest but real volume leverage
  - Competitive tension: HubSpot is a live alternative; use to push Zoho on multi-year discount
  - October peak creates a hard deadline that can be framed to accelerate vendor sign-off
  - Finance model shows EUR17,280 3-yr savings on Zoho — use as walk-away anchor if HubSpot attempts price match
procurement_risks:
  - risk: Auto-renewal clause locks in without 60-day cancellation notice
    severity: Medium
    mitigation: Mark contract calendar alert at month 10; negotiate notice window down to 30 days
  - risk: Zoho price increase on renewal if mid-market traction grows
    severity: Low
    mitigation: Negotiate a 5% annual cap on seat-price uplift in the MSA
  - risk: NetSuite connector (third-party) not covered by Zoho SLA
    severity: Medium
    mitigation: Require connector vendor SLA ≥ Zoho platform SLA or budget for native development
procurement_export_clause_feasible: true
