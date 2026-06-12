
## 01-market-researcher

```json
{
  "market_tam_eur": 2400000,
  "market_segment": "EU mid-market logistics",
  "market_growth_pct": 12,
  "market_incumbent_share": {
    "HubSpot": 34,
    "Zoho": 21,
    "Salesforce": 29
  },
  "market_buyer_priority": [
    "time-to-value",
    "EU data residency",
    "integration depth"
  ]
}
```

## 02-pricing-analyst

```json
{
  "pricing_hubspot_seat": 50,
  "pricing_zoho_seat": 38,
  "pricing_salesforce_seat": 75,
  "pricing_seats": 40,
  "pricing_hubspot_annual": 24000,
  "pricing_zoho_annual": 18240,
  "pricing_salesforce_annual": 36000,
  "pricing_cheapest": "Zoho"
}
```

## 03-risk-analyst

```json
{
  "risk_finalists": ["Zoho", "HubSpot"],
  "risk_zoho_eu_data_residency": "partial — EU DC available but not default; requires explicit configuration",
  "risk_zoho_vendor_stability": "medium — private company, strong SMB base, limited enterprise SLA guarantees",
  "risk_zoho_integration_depth": "medium — native connectors for common tools; logistics-specific ERPs need middleware",
  "risk_zoho_implementation_risk": "low — avg. go-live 6–8 weeks for 40-seat deployments",
  "risk_zoho_score": 6,
  "risk_hubspot_eu_data_residency": "strong — EU-hosted regions GA, GDPR DPA available",
  "risk_hubspot_vendor_stability": "low — public company, strong enterprise SLA, broad partner ecosystem",
  "risk_hubspot_integration_depth": "high — 1,000+ native integrations; logistics ERPs (e.g. SAP, Oracle TMS) well supported",
  "risk_hubspot_implementation_risk": "low-medium — avg. go-live 8–10 weeks; richer feature set adds config time",
  "risk_hubspot_score": 3,
  "risk_lower_is_better": true,
  "risk_recommended_finalist": "HubSpot",
  "risk_rationale": "HubSpot scores lower risk on all three buyer priorities (EU residency, integration depth, time-to-value). Cost premium over Zoho is EUR5,760/yr — justified by reduced implementation risk and stronger compliance posture ahead of October peak season."
}
```
