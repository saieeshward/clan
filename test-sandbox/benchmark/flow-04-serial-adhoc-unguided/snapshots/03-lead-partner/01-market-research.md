# Comparative Market Research: Salesforce vs HubSpot vs Zoho CRM
**For:** Brightline — 40-person advertising agency, Dublin, Ireland
**Stage:** 1 of 3 (market-researcher). Next stage: risk-analyst.
**Knowledge caveat:** No web access was available. All pricing and feature claims are from model training knowledge (cutoff ~Jan 2026). Vendor pricing changes frequently and is often quoted in USD; EUR figures below assume near-parity EUR list prices that vendors publish for the EU, annual billing, and exclude VAT. Treat all numbers as ±20% ballpark to be validated before contract.

## 1. Capabilities overview

**Salesforce (Sales Cloud).** The market leader for configurable, process-heavy sales orgs. Strengths: deepest customisation (objects, flows, approval chains), strong reporting/dashboards, mature AppExchange ecosystem, enterprise-grade permissions and audit trails. For an agency, it can model retainers, pitches, and media-plan pipelines, but only after meaningful configuration. Marketing capability requires separate paid products (Account Engagement/Pardot or Marketing Cloud), which roughly doubles cost.

**HubSpot (Sales Hub + Marketing Hub).** CRM plus native marketing automation in one platform. Strengths: best-in-class usability and adoption, built-in email marketing, forms, landing pages, ad management, and attribution reporting. The free CRM core lowers entry friction. Weaknesses: customisation ceilings (custom objects only on Enterprise), reporting less flexible than Salesforce, costs escalate sharply with marketing-contact tiers and Enterprise features.

**Zoho CRM.** The value option. Strengths: very low per-seat cost, surprisingly broad feature set (workflows, scoring, Zia AI, Canvas UI customisation), and the wider Zoho One suite (projects, books, campaigns) at a bundle price. Weaknesses: less polished UX, weaker ecosystem and partner network in Ireland, integrations often shallower (sync exists but field coverage and reliability lag), and admin documentation quality is inconsistent.

## 2. Ballpark pricing (EUR per user/month, annual billing, ex-VAT, 40 seats)

| Tier | Salesforce Sales Cloud | HubSpot Sales Hub | Zoho CRM |
|---|---|---|---|
| Entry/Pro | Starter ~€25; Professional ~€80 | Starter ~€20; Professional ~€90–100/seat | Standard ~€14; Professional ~€23 |
| Mid/Enterprise | Enterprise ~€165 | Enterprise ~€140–150/seat | Enterprise ~€40; Ultimate ~€52 |
| Realistic 40-seat config | Professional: ~€3,200/mo (~€38k/yr); Enterprise: ~€6,600/mo (~€79k/yr) | Sales Pro for 15 sellers + free seats for rest + Marketing Hub Pro (~€800/mo flat incl. contacts): ~€2,100–2,400/mo (~€26–29k/yr) | Professional all 40 seats: ~€920/mo (~€11k/yr); Enterprise: ~€1,600/mo (~€19k/yr) |

Notes: HubSpot prices marketing by contact tier, not only seats — agencies with large prospect databases should budget for contact-tier creep. Salesforce typically adds 5–15% at renewal and charges extra for premier support, sandboxes (below Enterprise), and Pardot (~€1,100/mo flat). Zoho One bundle is ~€37–45/user/mo for everything, sometimes cheaper than CRM Enterprise plus point tools.

## 3. Integration fit with ad-ops tooling

- **Meta/Google Ads:** HubSpot has *native* ads integration (audience sync, lead-ad capture, ROI reporting per campaign) — strongest of the three out of the box. Salesforce needs Pardot/Marketing Cloud or third-party connectors (Zapier, LeadsBridge) for audience sync; lead capture via web-to-lead is basic. Zoho has native Google Ads integration (Zoho CRM + Zoho Marketing) and Meta lead-ads sync via Zoho Social/Flow; functional but shallower field mapping.
- **GA4:** None of the three ingests GA4 natively into CRM records. HubSpot tracks its own web analytics (partial GA4 substitute); GA4-to-CRM joins usually go through BigQuery export + reverse-ETL (Hightouch/Census) or Zapier. Salesforce: same pattern, often via CRM Analytics or Datorama (extra cost). Zoho: Zoho Analytics can connect GA4 for dashboards cheaply.
- **Programmatic (DV360, The Trade Desk):** No native CRM connectors anywhere. Standard pattern is CRM → CDP/clean-room or CSV/API audience export. Salesforce has the strongest enterprise path (Data Cloud, Datorama heritage) but at significant cost; HubSpot and Zoho rely on middleware.
- **Middleware ecosystem:** Salesforce > HubSpot > Zoho in connector breadth; Zoho Flow partially offsets with cheap built-in automation.

## 4. Strengths/weaknesses summary for a 40-seat agency

- **Salesforce:** most powerful, highest cost and admin burden (likely needs a paid admin/partner ~€10–20k/yr extra); adoption risk highest.
- **HubSpot:** best adoption and native ad-ops fit; mid-range cost; watch contact-tier and Enterprise-upgrade creep.
- **Zoho:** lowest cost by 2–3x; acceptable capability; integration and ecosystem depth are the main compromise.

## 5. Open items deliberately left for the risk-analyst (stage 2)

1. GDPR posture (all three offer EU data residency and DPAs — HubSpot EU data centre in Frankfurt, Salesforce Hyperforce EU, Zoho EU DC in Amsterdam/Dublin — but depth of assessment is your remit).
2. Team adoption risk per platform (training load, change management, agency-typical turnover).
3. Vendor lock-in, renewal-uplift, and migration-out risk.
4. Validation of pricing assumptions above (flagged ±20%).
Stage 3 should then produce the costed recommendation + phased rollout per brief.md.
