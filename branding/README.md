# trRUSTt Branding Assets

## Logo Concept

The trRUSTt logo combines three core narratives into a single, minimalist mark:

| Element | Visual | Meaning |
|---|---|---|
| **Shield** | Geometric badge outline | Trust, reliability, protection |
| **Ascending Bars** | Three vertical bars, left→right increasing | Data growth, dashboards, analytics |
| **Sparkle** | 4-point star above tallest bar | AI intelligence, insight, amplification |

The wordmark emphasizes `RUST` in bold violet to reinforce the product name's unique styling while maintaining a clean, modern SaaS aesthetic.

---

## Color Palette

| Role | Hex | Tailwind | Usage |
|---|---|---|---|
| Primary | `#2563EB` | blue-600 | Shield, wordmark body, bar 1 & 2 |
| Accent | `#7C3AED` | violet-600 | "RUST" text, tallest bar, sparkle |
| Dark BG | `#0F172A` | slate-900 | Dark mode backgrounds |
| Muted | `#94A3B8` | slate-400 | Tagline, secondary text |

---

## File Inventory

| File | Type | Use Case |
|---|---|---|
| `icon.svg` | Icon only (256×256) | App icon, favicon base, generic mark |
| `icon-dark.svg` | Icon on dark BG (256×256) | Dark mode app icon, social avatar |
| `logo-horizontal.svg` | Icon + wordmark (620×180) | Website header, README,横幅 |
| `logo-vertical.svg` | Icon + wordmark stacked (300×380) | Splash screen, login page, docs cover |
| `wordmark.svg` | Text only (340×80) | Watermark, inline branding, terminal banner |

---

## Typography

- **Preferred:** [Inter](https://rsms.me/inter/) — modern geometric sans-serif
- **Fallbacks:** Plus Jakarta Sans, Satoshi, system UI fonts
- **Weight:** 400 (regular) for body, 700 (bold) for "RUST" emphasis
- **Letter-spacing:** -0.5px for the wordmark (tight, modern feel)

---

## Usage Guidelines

### Clear Space
Maintain padding equal to at least **25% of the icon height** around all sides of the logo.

### Minimum Sizes
- **Icon:** 24×24px (never smaller)
- **Horizontal logo:** 140px wide
- **Vertical logo:** 80px wide

### What NOT To Do
- ❌ Don't recolor the shield or sparkle
- ❌ Don't stretch, skew, or rotate
- ❌ Don't add gradients, shadows, or 3D effects
- ❌ Don't place on busy backgrounds — prefer solid dark or light
- ❌ Don't change the "trRUSTt" letter casing

### One-Color Variant
For single-color contexts (watermarks, terminal output, receipts), use `icon.svg` with all fills set to a single color. The shield outline and sparkle should remain visible.

---

## Product Name Styling

| Context | Styling | Example |
|---|---|---|
| UI, docs, marketing | **trRUSTt** | "Welcome to trRUSTt" |
| Code, env vars, CLI | `TRRUSTT` | `TRRUSTT_API_KEY` |
| Binary name | `TRRUSTT.exe` | `./TRRUSTT.exe --help` |
| Config dir | `~/.trrustt/` | `~/.trrustt/config.toml` |

---

## Design Inspiration

The mark draws from modern SaaS logos (Notion, Vercel, Linear, Supabase) — geometric, flat, solid colors, no gradients. The shield + bars + sparkle combination is unique to trRUSTt and creates a distinctive silhouette at any size.

---

> **trRUSTt your data. One binary. Infinite dashboards.**
