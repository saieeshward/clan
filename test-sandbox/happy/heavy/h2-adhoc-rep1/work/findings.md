
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
procurement_key_commitments: ["Annual subscription renewable monthly after year 1", "Quarterly data export right (Zoho export clause verified)", "DPA execution before go-live", "SLA: 99.9% uptime with credits", "30-day subprocessor notice obligation"]
procurement_negotiation_leverage: ["Competitor bids in hand (HubSpot at EUR24,000 vs Zoho EUR18,240)", "40-seat volume qualifies for mid-market discount tier", "Annual prepay in exchange for 5-8% further discount", "Peak-season deadline creates urgency for vendor to close before October", "Low switching cost (export clause) reduces lock-in fear — vendor knows buyer can walk"]
procurement_risks: [{"risk": "Vendor price increase at renewal after year 1", "severity": "Medium", "mitigation": "Negotiate renewal cap of CPI+2% in initial contract"}, {"risk": "NetSuite connector billed separately at renewal", "severity": "Low", "mitigation": "Lock connector pricing in contract schedule at signing"}, {"risk": "EU data residency add-on pricing opaque (HubSpot fallback risk)", "severity": "Low", "mitigation": "Require written quote inclusive of EU pinning before any HubSpot fallback decision"}, {"risk": "Delayed contract execution missing 2026-07-01 phase-1 start", "severity": "High", "mitigation": "Target contract signature by 2026-06-25; assign procurement champion now"}]
procurement_export_clause_feasible: true

## 11-legal-reviewer

legal_dpa_required: true
legal_gdpr_transfer_mechanism: Standard Contractual Clauses (SCCs) — both Zoho and HubSpot rely on EU SCCs for any data transfer outside EEA; Zoho EU region pinning eliminates most cross-border flows by default
legal_liability_cap_eur: 18240
legal_ip_ownership_clear: true
legal_indemnification_adequate: true
legal_risks: [{"risk": "DPA not executed before go-live", "severity": "High", "mitigation": "Procurement commitment already conditions go-live on DPA execution; legal to verify signed DPA is in place by 2026-06-25 alongside contract"}, {"risk": "SCC adequacy challenged post-go-live (regulatory shift)", "severity": "Medium", "mitigation": "Zoho EU pinning minimises EEA-to-third-country flows; monitor EDPB guidance; include re-negotiation trigger clause if SCC framework is invalidated"}, {"risk": "Liability cap set to 12-month fees — may be insufficient for a data breach affecting 40 seats of customer contact data", "severity": "Medium", "mitigation": "Negotiate uplift to 2x annual fees or add cyber liability insurance to cover gap between vendor cap and actual breach exposure"}, {"risk": "Subprocessor change notice window (30 days) may be too short to object before processing resumes", "severity": "Low", "mitigation": "Contract should specify that new subprocessor processing is suspended until objection period lapses or buyer consents"}, {"risk": "Zoho data deletion / return on termination timeline undefined", "severity": "Low", "mitigation": "Insert clause requiring certified deletion or export within 30 days of contract termination; align with quarterly export right already negotiated"}]
