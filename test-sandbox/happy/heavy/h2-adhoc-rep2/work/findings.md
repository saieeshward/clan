
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

## 11-legal-reviewer

legal_dpa_required: true
legal_gdpr_transfer_mechanism: Standard Contractual Clauses (SCCs) — both Zoho and HubSpot rely on EU SCCs for any data leaving EEA; Zoho EU region pinning minimises transfers but SCCs remain the contractual backstop
legal_liability_cap_eur: 18240
legal_ip_ownership_clear: true
legal_indemnification_adequate: false
legal_risks:
  - risk: DPA auto-execution gap — procurement noted 30-day subprocessor notice period but contract must explicitly require written countersignature on DPA amendments, not merely notification
    severity: Medium
    mitigation: Require mutual written consent (not just notice) for material subprocessor additions before signing MSA
  - risk: Liability cap inadequate — Zoho standard cap is 12 months of fees (~EUR18,240); a data breach involving 40-seat logistics data could trigger GDPR fines of up to EUR20M/4% global turnover, leaving Brightline exposed for the gap
    severity: High
    mitigation: Negotiate liability cap to minimum EUR250,000 for data-breach/IP claims; alternatively, obtain dedicated cyber-liability insurance bridging the gap
  - risk: IP ownership of custom workflow configurations — Zoho MSA standard terms assert vendor right to anonymise and use aggregate usage data; custom workflow logic could be inadvertently included
    severity: Low
    mitigation: Add explicit clause: all custom field schemas, workflow definitions, and report templates are Brightline IP; vendor usage limited to aggregate anonymised telemetry only
  - risk: Auto-renewal legal exposure — procurement flagged 60-day notice window; under EU contract law, failure to cancel may constitute tacit consent to revised pricing terms published by vendor
    severity: Medium
    mitigation: Negotiate 30-day notice window and stipulate that any renewal pricing changes require written acceptance, not silence-as-consent
  - risk: NetSuite third-party connector has separate EULA not reviewed — could introduce conflicting data-processing terms or IP assignment clauses
    severity: Medium
    mitigation: Legal review of connector vendor EULA before go-live; ensure connector DPA aligns with Zoho DPA and Brightline's GDPR obligations
