# Brightline CRM Evaluation — Report Draft

## market-researcher

**Assumptions and caveats.** No web access was used; all figures are from model knowledge with a cutoff of January 2026. Vendor list prices change frequently and EUR figures below are list-price ballparks (annual billing, ex-VAT) — verify against current vendor quotes before any decision. Where vendors price in USD, I assume rough EUR parity. "40-seat" costing assumes all 40 staff get a paid seat; in practice an agency could licence ~25 sales/account-management seats and use free/read-only seats for the rest, materially cutting cost (especially on HubSpot and Salesforce).

### Capabilities overview

**Salesforce (Sales Cloud).** The deepest and most configurable platform: custom objects, advanced workflow (Flow), territory management, forecasting, CPQ add-ons, and the largest ISV ecosystem (AppExchange). Reporting and permissioning are best-in-class. The cost is complexity: it usually requires an admin (often 0.25-0.5 FTE at this size) or a partner for setup, and the UI is the least loved by non-sales staff.

**HubSpot (Sales Hub + Marketing Hub).** Strongest usability and fastest time-to-value. Native marketing tooling (email, forms, landing pages, ad management) lives in the same data model as the CRM, which suits an agency that runs campaigns. Automation is good but shallower than Salesforce; custom objects and granular permissions only arrive at Enterprise tier. Marketing Hub pricing scales with contact volume, which can surprise agencies holding large prospect lists.

**Zoho CRM.** Best value for money. Covers ~80% of Salesforce's functionality (workflows, blueprints, scoring, decent API) at a fraction of the price, plus the wider Zoho One suite (projects, finance, desk) if Brightline wants an all-in-one stack. Weaknesses: less polished UX, weaker ecosystem and partner network in Ireland/EU, and reporting/AI features (Zia) lag the other two.

### Ballpark pricing (EUR per user/month, annual billing)

| Platform | Realistic tier for Brightline | List EUR/user/mo | 40 seats, EUR/year (approx.) |
|---|---|---|---|
| Salesforce | Sales Cloud Enterprise | ~165 | ~79,000 |
| Salesforce (lighter) | Pro Suite | ~100 | ~48,000 |
| HubSpot | Sales Hub Professional (~90/seat) + Marketing Hub Pro (~800/mo flat) | blended ~110 | ~53,000 |
| Zoho CRM | Enterprise | ~40 | ~19,000 |

Implementation/onboarding is extra: expect EUR 10-30k partner cost for Salesforce, ~EUR 3-6k mandatory onboarding for HubSpot Pro, and near-zero to EUR 5k for Zoho.

### Integration fit with ad-ops tooling

- **Meta/Google Ads:** HubSpot is the standout — native ads integration syncs audiences, captures lead-ad submissions, and attributes ad spend to CRM deals out of the box. Salesforce needs Marketing Cloud Advertising (a paid add-on, formerly Advertising Studio) or third-party connectors. Zoho has a native Google Ads integration in CRM and Meta lead-ad sync via Zoho Social/Marketing Automation — functional but more assembly required.
- **GA4:** No CRM has a deep native GA4 integration; all three rely on UTM capture plus connectors (Zapier/Make, or GA4 BigQuery export into the CRM reporting layer). HubSpot attribution partially substitutes for GA4; Salesforce CRM Analytics can ingest GA4 data with effort.
- **Programmatic (DV360, The Trade Desk, etc.):** None offer native DSP integrations. The realistic pattern is CRM -> CDP/clean-room (LiveRamp, Hightouch-style reverse ETL) -> DSP audiences. Salesforce has the strongest enterprise path here (Data Cloud with audience activation), but at significant extra cost; HubSpot and Zoho depend on middleware. For a 40-person agency, middleware (Zapier/Make/Fivetran-lite) is the pragmatic answer regardless of CRM.

### Strengths/weaknesses summary

- Salesforce: + depth, ecosystem, scalability; - cost, admin burden, adoption risk for non-sales staff.
- HubSpot: + usability, native ad-ops fit, fast rollout; - contact-based marketing pricing, Enterprise-gated permissions.
- Zoho: + price, suite breadth; - polish, weaker EU partner ecosystem, lighter ad-ops integrations.

**Steer for next agents:** HubSpot looks like the best capability-per-euro fit for an ad agency workflow; Zoho is the budget hedge; Salesforce is justifiable only if Brightline expects enterprise-client procurement requirements demanding it.
