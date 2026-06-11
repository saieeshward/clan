
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
  - Annual subscription billed upfront; cancellation forfeits remaining term
  - EU data-residency SLA included in Zoho Enterprise at no surcharge
  - HubSpot EU data-pinning add-on must be written into order form explicitly
  - Both vendors require 30-day written notice before renewal to renegotiate terms
  - NetSuite connector for Zoho requires separate third-party licence (est. EUR 1,200/yr)
procurement_negotiation_leverage:
  - 40-seat deal is borderline SMB/mid-market; vendors discount 10-15% for multi-year commit
  - Zoho's lower ASP means HubSpot AE has greater incentive to match on price
  - October go-live deadline creates urgency; negotiate implementation credits to compensate
  - Quarterly data-export clause (Zoho) can be leveraged as walk-away credibility
  - Request waiver of first-year onboarding fee (~EUR 2,000) as part of deal close
procurement_risks:
  - risk: Zoho contract auto-renews at list price if notice window missed
    severity: Medium
    mitigation: Set calendar reminder 45 days before renewal; assign procurement owner
  - risk: HubSpot EU add-on not on order form leaves data-residency unenforceable
    severity: High
    mitigation: Legal to review final order form before signature; require DPA addendum
  - risk: Third-party NetSuite connector vendor may change pricing independently
    severity: Low
    mitigation: Lock connector pricing for 24 months in side letter at signing
  - risk: FX exposure if invoiced in USD not EUR
    severity: Low
    mitigation: Negotiate EUR invoicing; if not available, hedge via forward contract
procurement_export_clause_feasible: true

## 11-legal-reviewer

legal_dpa_required: true
legal_gdpr_transfer_mechanism: Standard Contractual Clauses (SCC) — both Zoho and HubSpot rely on EU SCCs under GDPR Art. 46(2)(c) for any cross-border sub-processor transfers; Zoho EU region pinning reduces transfer surface
legal_liability_cap_eur: 18240
legal_ip_ownership_clear: true
legal_indemnification_adequate: false
legal_risks:
  - risk: HubSpot EU data-pinning add-on omitted from order form renders data-residency commitment legally unenforceable, exposing Brightline to GDPR Art. 44 violation
    severity: High
    mitigation: Legal must review final order form pre-signature; DPA addendum must explicitly cite EU processing region and list sub-processors
  - risk: Liability cap (typically 12 months subscription fees) is low relative to potential GDPR supervisory-authority fines (up to 4% global turnover); indemnification clauses in standard SaaS contracts rarely cover regulatory penalties
    severity: High
    mitigation: Negotiate enhanced liability cap to minimum 2× annual contract value and seek explicit GDPR-fine indemnification carve-in; obtain cyber-liability insurance as backstop
  - risk: Third-party NetSuite connector vendor constitutes an additional data sub-processor not covered by primary DPA; no SCCs in place for that vendor
    severity: Medium
    mitigation: Require NetSuite connector vendor to execute a sub-processor DPA with SCCs before go-live; include in Zoho's sub-processor registry obligation
  - risk: Auto-renewal clause (Zoho) may bind Brightline to additional annual term without affirmative consent, raising consumer contract concerns under EU Directive 2011/83/EU and national implementation
    severity: Medium
    mitigation: Negotiate mutual written-notice renewal requirement; assign contract-renewal owner as per procurement recommendation
  - risk: IP ownership of any custom configuration, workflows, or integrations built during onboarding is typically assigned to the vendor in standard SaaS T&Cs; exit portability only covers data
    severity: Low
    mitigation: Negotiate explicit IP assignment clause for all custom work product delivered by vendor professional services; ensure quarterly data-export right covers all custom fields
