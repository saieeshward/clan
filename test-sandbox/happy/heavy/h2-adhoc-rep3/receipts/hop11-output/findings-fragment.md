## 11-legal-reviewer

legal_dpa_required: true
legal_gdpr_transfer_mechanism: EU Standard Contractual Clauses (SCCs) + EU data residency (Frankfurt)
legal_liability_cap_eur: 18240
legal_ip_ownership_clear: true
legal_indemnification_adequate: false
legal_risks: [{"risk": "DPA not executed before go-live", "severity": "High", "mitigation": "Make DPA signature a contract condition precedent; confirm before data migration begins"}, {"risk": "Subprocessor change notice (30 days) may be insufficient for internal approval cycles", "severity": "Medium", "mitigation": "Negotiate 60-day notice window; include right to object and exit without penalty if new subprocessor is unacceptable"}, {"risk": "Liability cap equals one year's fees (EUR 18,240) — insufficient for a data breach affecting EU customers", "severity": "High", "mitigation": "Negotiate cap to 2x annual contract value or add data-breach-specific liability carve-out; require Zoho to maintain cyber-liability insurance"}, {"risk": "Indemnification clause covers IP infringement only; no indemnity for Zoho-caused data-breach regulatory fines", "severity": "Medium", "mitigation": "Insert mutual indemnification clause covering third-party regulatory claims arising from vendor-side security failures"}, {"risk": "Governing law and jurisdiction may default to Zoho home jurisdiction (India/Delaware)", "severity": "Low", "mitigation": "Negotiate Irish or EU governing law and exclusive EU jurisdiction clause given Brightline EU domicile"}]
