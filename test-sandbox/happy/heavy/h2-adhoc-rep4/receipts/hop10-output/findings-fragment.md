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
