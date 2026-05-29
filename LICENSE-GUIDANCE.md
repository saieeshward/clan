# CLAN Licensing Strategy

---

## Recommendation

Use a **three-tier licensing model** — different licences for different layers of the CLAN ecosystem. This is the model used by successful open formats that maintain commercial products alongside them (PDF/Adobe, Kubernetes/Google, Terraform/HashiCorp pre-2023).

| Layer | Licence | Rationale |
|---|---|---|
| CLAN Format Specification | Apache 2.0 | Open, patent-protected, commercial-friendly |
| CLAN SDK (Rust core + bindings) | Apache 2.0 | Same — consistent with spec |
| CLAN App (Tauri viewer) | Proprietary | Commercial moat — best viewer, not exclusive |

---

## Why Apache 2.0 for the Spec and SDK

### Maximum adoption without patent risk

Apache 2.0 includes an explicit **patent grant**. Every contributor to the spec or SDK automatically grants all users a royalty-free licence to any patents they hold that are necessary to implement CLAN. This means:

- Teams adopting CLAN have legal clarity — no surprise patent claims later
- Researchers and academics can publish CLAN-based work freely
- Enterprises can build CLAN-compatible products without legal review headaches
- This is the same reason Kubernetes, TensorFlow, Android, and Swift all use Apache 2.0

### Commercial use allowed

Apache 2.0 allows commercial use. Companies can build CLAN-compatible products and charge for them. This drives adoption — if organisations cannot commercialise around CLAN, they will not invest in adopting it.

### Attribution preserved

Apache 2.0 requires preserving copyright notices and attributing the original authors. CLAN spec implementations must acknowledge the origin. This builds brand recognition as adoption grows.

### Why not MIT?

MIT has no patent grant. A competitor could implement CLAN, patent aspects of their implementation, and assert those patents against the CLAN community. Apache 2.0 closes this risk.

### Why not GPL or LGPL?

GPL's viral nature would prevent proprietary implementations and block enterprise adoption. LGPL allows proprietary use but creates ambiguity in linking. Neither fits an open format that needs maximum implementation diversity.

### Why not Creative Commons (CC)?

CC licences (BY, BY-SA, etc.) are designed for creative works — documents, images, music. They are explicitly not recommended for software or specifications. Apache 2.0 is the right tool for a technical specification with an accompanying SDK.

---

## Why Proprietary for the App

The CLAN format being open does not require the best viewer to be open. This is the standard pattern for successful open formats:

- **PDF** is an ISO open standard. Adobe Acrobat Reader is proprietary. Adobe still has a business.
- **HTML** is a W3C open standard. Chrome is built on open source but Google's commercial interests drive it.
- **Markdown** is an open format. Obsidian, Typora, and Bear are proprietary apps. All thrive.

Making the format open creates adoption. Making the viewer proprietary creates a commercial product. These are not in conflict.

The proprietary app wins on quality, not exclusivity. Anyone can build a CLAN viewer — and some will. The goal is to build the one people choose because it is the best, not because it is the only option.

---

## How to Apply the Licences

### Format Specification (this repository)

Add the following to every spec document header:

```
Copyright 2026 [Your Organisation Name]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

Add an `Apache-2.0` `LICENSE` file at the root of the spec repository.

Add to `README.md`:
```markdown
## Licence
CLAN Specification and Reference SDK — Apache License 2.0
CLAN App — Proprietary (© [Your Organisation Name])
```

### SDK Repository

Same Apache 2.0 header in every source file. Standard `LICENSE` file at repo root.

Add to `Cargo.toml`:
```toml
license = "Apache-2.0"
```

Add to `package.json` (TypeScript SDK):
```json
"license": "Apache-2.0"
```

### The .clan Files Themselves

CLAN files that ship as part of the spec (example files, templates) should be explicitly dedicated to the public domain or CC0 — so anyone can use them as a starting point without attribution obligations.

---

## Protecting the App while the Format is Open

Three mechanisms reinforce the app's commercial position without restricting the format:

**1. Trademark**
Register "CLAN" and the CLAN logo as trademarks. Anyone can implement the format, but they cannot call their implementation "CLAN" without a trademark licence. This is how Red Hat operates around Linux — they can't stop people from using Linux, but "Red Hat Enterprise Linux" is theirs.

Cost: ~$250–350 per class at the USPTO (US); ~€850 at the EUIPO (EU).

**2. Quality moat**
The reference app ships features that the format spec enables but that are hard to replicate: the lineage timeline, the multi-webview sandboxed rendering, the edit bridge, the agent panel. Open spec, proprietary execution.

**3. Ecosystem lock-in**
If the app integrates with a cloud context store, a template marketplace, or a team collaboration layer — these become network effects that no spec-compliant alternative can replicate purely by reading the format.

---

## If You Want to Open Source the App Later

The open-core model is a common evolution path: start proprietary, open source once competitive moat is established. HashiCorp, Elastic, GitLab, and others followed this path.

If you open source the app later, use **Apache 2.0** or **AGPL v3**:
- Apache 2.0: allows commercial forks (competitors can take and sell it)
- AGPL v3: requires any server-side deployment to release their modifications — good if the app has cloud components

Decide based on whether cloud-hosted versions of the app are a concern.
