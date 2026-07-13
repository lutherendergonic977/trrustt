# Monetization Model — Detailed

## 1. Revenue Streams Summary

```
                          TOTAL ADDRESSABLE REVENUE
                                   │
        ┌──────────────┬───────────┼───────────┬──────────────┐
        │              │           │           │              │
   DIRECT SALES   ENTERPRISE   WHITE-LABEL   API/PAYG    MARKETPLACE
   (B2C/B2B)      (B2B)        (B2B)         (B2B)       (B2C/B2B)
      30%            35%          15%          10%          10%
```

## 2. Detailed Pricing Tiers

### 2.1 Direct Sales Tiers

| Plan | Price | Target | Key Features |
|---|---|---|---|
| **Free** | $0 | Individual learners, evaluation | 3 dashboards/month, beginner DAX, watermark, community support |
| **Pro** | $149/year or $299 perpetual | Individual analysts, freelancers | Unlimited dashboards, all DAX levels, theme engine, CLI, 5 MCP servers |
| **Pro+** | $249/year | Power users | All Pro + image-to-dashboard, advanced AI models, priority support |

### 2.2 Enterprise Tiers

| Plan | Price | Min Seats | Key Features |
|---|---|---|---|
| **Team** | $49/user/month | 5 | All Pro+ features, SSO, basic RBAC, shared configs, email support (24h) |
| **Business** | $39/user/month | 25 | Team + advanced RBAC, audit logging, admin console, SLA 99.5% |
| **Enterprise** | Custom | 100+ | Business + dedicated support, SLA 99.9%, on-premise option, custom integrations, white-label option |

### 2.3 White-Label / OEM

| Tier | Annual Fee | Revenue Share | Includes |
|---|---|---|---|
| **OEM Starter** | $25,000 | 15% | Branding kit, custom build, basic support |
| **OEM Growth** | $50,000 | 10% | Starter + priority updates, co-marketing |
| **OEM Enterprise** | $100,000+ | 5% | Growth + dedicated dev hours, custom features |

### 2.4 API / PAYG

| Metric | Price | Volume Discount |
|---|---|---|
| Dashboard generation | $0.50/request | 10K+ = $0.40, 100K+ = $0.30 |
| DAX measure generation | $0.10/measure | 1K+ = $0.08, 10K+ = $0.05 |
| Image analysis | $0.25/image | Custom |
| Insights generation | $0.15/request | Custom |

### 2.5 Marketplace Add-ons

| Product | Price Point | Examples |
|---|---|---|
| Theme packs | $9.99 - $29.99 | "Executive Dark", "Healthcare Pro", "Finance Elite" |
| DAX recipe packs | $19.99 - $49.99 | "Time Intelligence Pack", "Financial Ratios Pack" |
| Industry templates | $49.99 - $199.99 | "Retail Analytics Dashboard", "SaaS Metrics Dashboard" |
| Premium prompts | $9.99/month | Enhanced prompt chains for specific domains |

## 3. Payment Infrastructure

### 3.1 Payment Processor: Paddle

**Why Paddle:**
- Handles global sales tax / VAT / GST (MoR — Merchant of Record)
- Supports 200+ markets
- Recurring billing (subscriptions)
- One-time purchases
- In-app checkout
- Fraud protection
- Localized pricing

### 3.2 Enterprise Invoicing

For deals > $10,000/year:
- NET-30 invoice via Stripe Invoicing or manual
- PO (Purchase Order) support
- Wire transfer / ACH
- Custom contract with SLA

### 3.3 License Key Generation

```
Purchase ──▶ Paddle webhook ──▶ License Server generates key ──▶ Email to user
                                   │
                                   ├── Key format: XXXX-XXXX-XXXX-XXXX
                                   ├── Signed with HMAC-SHA256
                                   ├── Contains: tier, features, expiry, seats
                                   └── Stored in license DB
```

## 4. Marketplace Distribution

### 4.1 Microsoft AppSource

- **Listing type:** Power BI Visual / App
- **Revenue share:** 15% to Microsoft
- **Requirements:** 
  - Microsoft Partner Network membership ($0 for developer)
  - App certification process
  - Security & compliance review
- **Timeline:** 4-8 weeks for approval

### 4.2 Azure Marketplace

- **Listing type:** SaaS / Managed Application
- **Revenue share:** 15% to Microsoft (reduced with IP co-sell)
- **Benefits:**
  - Enterprise procurement via Azure billing
  - MACC (Microsoft Azure Consumption Commitment) eligible
  - Co-sell ready status

### 4.3 GitHub Marketplace

- **Listing type:** GitHub App / Action
- **Revenue share:** 0% (free listing)
- **Benefits:** Developer audience, OSS community

### 4.4 AWS Marketplace

- **Listing type:** SaaS
- **Revenue share:** Varies (typically 5-20%)
- **Benefits:** Enterprise reach beyond Azure

### 4.5 Own Website (Direct)

- **Processor:** Paddle
- **Revenue share:** ~5% (Paddle fees)
- **Benefits:** Full control, highest margin, brand building

## 5. Customer Acquisition

### 5.1 Marketing Channels

| Channel | Budget Allocation | Target CPA |
|---|---|---|
| Content marketing (blog, tutorials) | 25% | Organic |
| LinkedIn Ads | 20% | $50 |
| Google Ads (Power BI keywords) | 20% | $40 |
| Power BI community (forums, events) | 15% | Organic |
| YouTube (tutorials, showcases) | 10% | Organic |
| Partner referrals (consultants) | 10% | Variable |

### 5.2 Conversion Funnel Targets

```
Website Visitors          10,000/mo  (100%)
Free Tier Signups          2,000/mo  (20%)
Active Free Users          1,000/mo  (10%)
Pro Trial Starts             300/mo  (3%)
Pro Conversions              100/mo  (1%)
Enterprise Leads              20/mo  (0.2%)
Enterprise Closed              5/mo  (0.05%)
```

## 6. Financial Projections

### 6.1 Year 1 (Conservative)

| Revenue Source | Monthly (Month 12) | Annual |
|---|---|---|
| Pro subscriptions (500 users) | $6,208 | $62,500 |
| Team subscriptions (10 teams × 10 users) | $4,900 | $39,200 |
| Enterprise (3 deals) | $6,250 | $50,000 |
| Marketplace add-ons | $1,500 | $15,000 |
| **Total** | **$18,858** | **$166,700** |

### 6.2 Year 3 (Target)

| Revenue Source | Monthly | Annual |
|---|---|---|
| Pro subscriptions (3,000 users) | $37,250 | $447,000 |
| Team subscriptions (100 teams) | $49,000 | $588,000 |
| Enterprise (20 deals) | $41,667 | $500,000 |
| White-Label (5 partners) | $12,500 | $150,000 |
| API/PAYG | $8,333 | $100,000 |
| Marketplace | $12,500 | $150,000 |
| **Total** | **$161,250** | **$1,935,000** |

---

> **Document Version:** 1.0  
> **Part of:** IntelliDashboard Builder Business Docs
