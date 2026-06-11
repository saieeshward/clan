# Market Research: CRM Options for Brightline (40-seat Dublin Ad Agency)

**Author:** market-researcher agent | **Date:** 2026-06-10
**Caveat:** No web access. All pricing and feature claims are from model knowledge with a training cutoff of early 2026; vendors change list prices frequently. Figures are EUR list-price ballparks per user/month, billed annually, excluding VAT. Verify against current vendor pricing pages before contracting.

## 1. Salesforce (Sales Cloud)

**Capabilities.** The market's deepest platform: highly customisable objects, enterprise workflow automation (Flow), strong reporting/dashboards, Einstein/Agentforce AI add-ons, and the largest ISV ecosystem (AppExchange). Marketing-side depth lives in separate, separately-priced products (Marketing Cloud / Account Engagement), which matters for an ad agency.

**Pricing (ballpark).** Starter ~EUR 25; Pro Suite ~EUR 100; Enterprise ~EUR 165; Unlimited ~EUR 330 per user/month. A realistic 40-seat agency config (Enterprise) lands around **EUR 6,600/month (~EUR 79k/yr)** before add-ons; implementation partners typically add EUR 15-50k one-off. Starter/Pro tiers are cheaper but lose the customisation that justifies Salesforce in the first place.

**Strengths/weaknesses.** Strengths: scalability, granular permissions, audit trails, EU data residency options (Hyperforce EU), best-in-class ecosystem. Weaknesses: cost, admin burden (40-seat orgs usually need a part-time/full-time admin or partner), slower time-to-value, and adoption friction for non-sales-ops users.

**Ad-ops fit.** No native Meta/Google Ads connectors in Sales Cloud itself; relies on AppExchange apps, Marketing Cloud, or middleware (Zapier/Make/Workato, LeadsBridge for lead-form sync). GA4 integration is indirect. Strong but expensive path.

## 2. HubSpot

**Capabilities.** CRM suite organised in Hubs (Marketing, Sales, Service, Content, Operations). Excellent UX, fast onboarding, native email/forms/landing pages, solid automation in Pro tiers, good native reporting. Free CRM core lowers entry friction.

**Pricing (ballpark).** Sales Hub Pro ~EUR 90/seat; Enterprise ~EUR 150/seat. Marketing Hub is priced largely by marketing contacts, not seats (Pro from ~EUR 800/month for 2k contacts). A 40-seat mix (e.g., 15 Sales Pro seats, Marketing Pro, remainder on free/core seats) plausibly lands **EUR 2,500-4,000/month (~EUR 30-48k/yr)**. All 40 on paid Sales Pro seats would be ~EUR 3,600/month. One-off Pro onboarding fee (~EUR 1,500-3,000) applies.

**Strengths/weaknesses.** Strengths: adoption (consistently the easiest of the three for mixed sales/account-management teams), native ad tooling, strong inbound marketing DNA — a natural cultural fit for an agency. Weaknesses: costs scale sharply with marketing contacts and Enterprise features; custom-object flexibility weaker than Salesforce; reporting limits on lower tiers.

**Ad-ops fit. Best of the three natively.** Built-in Ads tool syncs Meta, Google, and LinkedIn ad accounts, pulls lead-gen forms into the CRM, and reports ad-attributed revenue. Native GA4 measurement is limited but well-served by standard connectors; large app marketplace covers programmatic gaps (DSPs generally need middleware for all three vendors).

## 3. Zoho CRM

**Capabilities.** Strong core SFA, workflow automation, Zia AI, and the wider Zoho One bundle (45+ apps incl. Campaigns, Desk, Books) at an aggressive price. Canvas allows deep UI customisation.

**Pricing (ballpark).** Standard ~EUR 20; Professional ~EUR 35; Enterprise ~EUR 50; Ultimate ~EUR 65 per user/month. Zoho One bundle ~EUR 37-45/user/month all-employee pricing. 40 seats on CRM Enterprise: **~EUR 2,000/month (~EUR 24k/yr)** — roughly a third of Salesforce.

**Strengths/weaknesses.** Strengths: unbeatable price/feature ratio, EU data centres, GDPR-aligned posture, low lock-in. Weaknesses: weaker ecosystem and partner network in Ireland, UX less polished, integrations often shallower (sync gaps, API quirks), smaller talent pool of admins.

**Ad-ops fit.** Native Google Ads integration in CRM; Meta lead ads via Zoho Social/Campaigns or middleware; GA4 via Zoho Marketing tools. Workable but more assembly required than HubSpot.

## Comparative read and pick

For a 40-person agency where most users are account managers and new-business folk (not sales-ops engineers), adoption risk and native ad-platform integration dominate. Salesforce is over-engineered and over-priced for this size; Zoho is the value play but adds integration assembly and ecosystem risk. **HubSpot** offers the best adoption profile, the only genuinely native Meta/Google Ads + attribution tooling, and a mid-pack TCO (~EUR 30-48k/yr). Recommended vendor: **HubSpot**, confidence 4/5 (pricing figures are the main uncertainty given no web access).
