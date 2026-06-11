## 11-legal-reviewer

legal_dpa_required: true
legal_gdpr_transfer_mechanism: EU Standard Contractual Clauses (SCCs) + EU data residency addendum
legal_liability_cap_eur: 36000
legal_ip_ownership_clear: true
legal_indemnification_adequate: false
legal_risks:
- risk: Auto-renewal clause without opt-out diarisation exposes Brightline to unintended multi-year commitment
  severity: Medium
  mitigation: Require written notice of non-renewal 30 days prior; add calendar trigger at month 10; legal review of auto-renewal language before signing
- risk: HubSpot EU data residency is an add-on and its absence at contract execution would breach GDPR Article 46 transfer requirements
  severity: High
  mitigation: Make EU-residency addendum a condition precedent to contract execution; obtain firm written quote before signing
- risk: Liability cap (typically 12 months fees) may be insufficient for a data breach given GDPR fines up to 4% global turnover
  severity: High
  mitigation: Negotiate enhanced liability cap for data-protection breaches; require vendor cyber-liability insurance evidence
- risk: Indemnification clauses in standard Zoho/HubSpot contracts exclude consequential losses, leaving Brightline exposed for downstream logistics disruptions
  severity: Medium
  mitigation: Negotiate mutual indemnification for data-breach and IP infringement; review exclusion-of-consequential-loss carve-outs
- risk: Subprocessor change notice (30 days) may be insufficient for Brightline to evaluate privacy impact before data is transferred to new subprocessors
  severity: Low
  mitigation: Contractually require 30-day advance notice with right to object; include DPA clause permitting termination if objection not resolved
