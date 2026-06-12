
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

risk_top_1: migration data quality
risk_top_2: user adoption
risk_top_3: integration latency
risk_migration_sev: High
risk_adoption_sev: Medium
risk_zoho_maturity_sev: Medium
risk_hubspot_maturity_sev: Low
risk_finalists: HubSpot, Zoho
risk_recommendation: HubSpot preferred on maturity; Zoho viable if budget is primary constraint

## 04-gdpr-reviewer

gdpr_residency_required: EU (Frankfurt)
gdpr_hubspot_eu_pinning: add-on (additional cost)
gdpr_zoho_eu_pinning: included (no extra cost)
gdpr_dpa_both: true
gdpr_subprocessor_notice_days: 30
gdpr_winner: Zoho
gdpr_note: Zoho's EU data residency is bundled; HubSpot's is a chargeable add-on that widens its cost gap further

## 05-integrations-assessor

integ_required: NetSuite, Slack, Twilio, DocuSign, Outlook, Shopify
integ_hubspot_native: 5
integ_zoho_native: 4
integ_zoho_gap: NetSuite (via third-party connector, supported but not native)
integ_hubspot_gap: none in required list
integ_effort_weeks_hubspot: 8
integ_effort_weeks_zoho: 9
integ_connector_cost_zoho_netsuite_est_eur: 1200
integ_winner: HubSpot
integ_note: HubSpot covers all 6 required connectors natively; Zoho needs an additional NetSuite connector adding ~1 week effort and ~EUR 1,200/yr cost, partially eroding its price advantage

## 06-customer-discovery

cust_interviews: 6
cust_top_need: lead response < 2h
cust_adoption_target_pct: 90
cust_champion_count: 4
cust_training_hours: 24
cust_pref_lean: Zoho on cost, HubSpot on UI
cust_note: Internal champions (4 of 6) prefer HubSpot UI for speed; ops-side users lean Zoho on cost. 24h training budget is achievable with either vendor. 90% adoption target is tighter than industry median — maturity and UX matter. HubSpot's cleaner UI reduces adoption risk given the 90% target.

## 07-competitive-intel

comp_hubspot_strength: marketing suite
comp_zoho_strength: price + EU residency
comp_salesforce_strength: enterprise depth
comp_switching_cost_zoho: low
comp_zoho_export_clause: true
comp_note: Zoho's quarterly export clause keeps switching cost low — a hedge against its maturity risk. Salesforce remains disqualified on cost. HubSpot and Zoho are the only viable finalists; competitive positioning aligns with prior findings.

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
rollout_phase1_scope: configuration, data migration, admin training
rollout_phase2_date: 2026-09-15
rollout_phase2_scope: marketing suite activation, full-team onboarding
rollout_phase3_date: 2026-11-01
rollout_phase3_scope: post-peak review, optimisation, advanced workflow enablement
rollout_go_live_before_peak: true
rollout_total_weeks: 9
rollout_champion_program: true
rollout_champion_count: 4
rollout_training_hours_planned: 24
rollout_peak_season_start: October
rollout_vendor_selected_for_plan: Zoho
rollout_note: 9-week plan delivers core CRM before the October peak; champion program (4 leads) drives 90% adoption target; phase-3 post-peak review addresses any advanced workflow gaps
