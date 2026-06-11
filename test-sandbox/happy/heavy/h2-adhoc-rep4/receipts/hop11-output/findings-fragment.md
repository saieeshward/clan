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
