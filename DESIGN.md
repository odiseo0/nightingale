# HealthTech Design System — Light Mode

> **Category:** HealthTech / Digital Health  
> **Design direction:** Light-mode-first, clinical clarity without a hospital aesthetic, precise minimalism, cool off-whites, blue accent.  
> **Foundation:** The typographic rhythm, spacing, component geometry, and restraint of the supplied Linear-inspired design system.  
> **Theme status:** Approved light palette; this document is the adapted implementation specification, not a redesign of the underlying component architecture.

## 1. Visual Theme & Atmosphere

The product uses a **native light visual system**. Its canvas is an intentionally soft, blue-tinted off-white (`#F8FAFC`) rather than a stark white. Information is organized through restrained changes in surface color: near-white elevated cards (`#FCFDFE`), cool secondary areas (`#F1F5F9`), and interaction fills (`#E9EFF5`). Fine borders (`#DCE5EE`) and light, diffuse shadows provide definition without visual noise.

The design should convey **trust, calm, clarity, privacy, and precision**. It should feel appropriate for reviewing appointments, personal health information, activity metrics, and care tasks, while avoiding sterile hospital cues, decorative gradients, or excessive visual urgency. Color reinforces the hierarchy but must not be the sole carrier of a health-related meaning.

Typography remains rooted in **Inter Variable** with OpenType features `"cv01"` and `"ss03"` enabled. The signature weights remain 400 for reading, 510 for product UI emphasis, and 590 for strong emphasis; 300 is limited to intentionally de-emphasized content. Display sizes preserve the compact negative letter-spacing of the source system. **Berkeley Mono** remains the companion for code and technical labels, with system monospace fallbacks.

The chromatic identity is a **controlled blue accent**: `#2563EB` for primary actions and interactive emphasis; `#1D4ED8` for hover and text on pale blue; `#1E40AF` for pressed interactions; and `#DBEAFE` for selected backgrounds. Blues should appear with intent, rather than covering large background areas. Green is retained **only as a semantic status color**, never as a second brand accent.

**Key Characteristics:**
- Light-mode-native surfaces: `#F8FAFC` canvas, `#F1F5F9` panels, `#FCFDFE` elevated cards.
- Soft but visible edge separation: `#DCE5EE` standard borders, plus very subtle translucent dark strokes where appropriate.
- Inter Variable with `"cv01", "ss03"` enabled globally; Berkeley Mono for technical information.
- Signature 510 weight for navigation, controls, and ordinary emphasis.
- Negative display letter-spacing (for example, -1.584px at 72px, -1.056px at 48px).
- Brand blue: `#2563EB`; hover `#1D4ED8`; pressed `#1E40AF`; selected fill `#DBEAFE`.
- Foreground hierarchy based on dark navy-gray text rather than pure black.
- Subtle dark-on-light shadows in place of the original light-on-dark luminance effects.
- Radix UI primitives remain a suitable component foundation, as in the source description.
- Success greens (`#27A644`, `#10B981`) are limited to non-text indicators; accessible success text uses `#166534`.
- Never introduce a pure-white surface or text token. Use the off-whites defined below instead.

## 2. Color Palette & Roles

### Background Surfaces

- **App Background / Canvas** (`#F8FAFC`): Primary page background; the visual baseline of the application.
- **Elevated Surface** (`#FCFDFE`): Cards, dropdowns, dialogs, highlighted reading surfaces. This is the brightest approved off-white.
- **Secondary Surface** (`#F1F5F9`): Sidebars, section wells, nested layouts, grouped panels.
- **Interaction Surface** (`#E9EFF5`): Neutral hover/pressed fills, control group highlights, subdued interactive backgrounds.
- **Selected Surface** (`#DBEAFE`): Active navigation, selected filters, emphasized blue-linked panels. Pair with an appropriately dark foreground.
- **Transparent** (`transparent`): Appropriate for unfilled buttons and overlays on known, tested surfaces; do not replace a necessary visible boundary with transparency.

**Surface layering:** Page (`#F8FAFC`) → panel (`#F1F5F9`) → card (`#FCFDFE`) → optional interaction (`#E9EFF5`) or selected blue (`#DBEAFE`). A card may also sit directly on the page canvas. The correct hierarchy comes from context, not a rule that every nested region must be brighter.

### Text & Content

- **Primary Text** (`#111827`): Page titles, headings, patient-entered values, metrics, and primary body copy.
- **Secondary Text** (`#475569`): Descriptions, secondary labels, menu items, and general supporting copy.
- **Tertiary Text** (`#5B6778`): Metadata, secondary timestamps, captions, and nonessential supporting content. Chosen to remain readable on the specified light neutral surfaces.
- **Quaternary / Disabled Text** (`#94A3B8`): Placeholders and genuinely disabled controls only. Do **not** use it for clinical measurements, form instructions, ordinary body text, or information users must read.
- **Inverse Text on Brand Blue** (`#F8FAFC`): Text and icons on filled blue controls; never substitute a brighter, pure-white value.
- **Text on Selected Blue** (`#1D4ED8`): Active labels and text shown on `#DBEAFE`.

### Brand & Accent

- **Brand / Primary Accent** (`#2563EB`): Primary CTAs, links on approved neutral surfaces, active control emphasis, key progress strokes.
- **Accent Hover** (`#1D4ED8`): Hover state of primary filled controls; preferred foreground for labels on pale-blue backgrounds.
- **Accent Pressed** (`#1E40AF`): Pressed state of filled controls and higher-emphasis blue foreground when needed.
- **Accent Soft / Selection** (`#DBEAFE`): Selected navigation items, filter states, soft callouts, unobtrusive progress tracks.
- **Security / Privacy Emphasis** (`#1D4ED8` on `#DBEAFE`): Security and privacy-related labels, only when blue is semantically appropriate; use text and icons to communicate meaning.
- **Accent Focus Ring** (`#2563EB`): Visible 2px focus outline with 2px offset; translucent blue halo is optional, not a replacement.

**Usage constraint:** `#2563EB` works for standard-sized text on `#F8FAFC` and `#FCFDFE`, but it does not provide a 4.5:1 contrast ratio on `#DBEAFE`. On soft-blue surfaces, use `#1D4ED8` or `#1E40AF` for normal-size text.

### Status Colors

The source system already distinguishes success with green. Preserve this as a **limited semantic exception** to the blue brand palette:

- **Success Indicator** (`#27A644`): Success/active dots and non-text status markers where geometry or a label also identifies the state.
- **Emerald Indicator** (`#10B981`): Secondary success dot, completed-step marker, or progress indicator. Do not place small pale text directly on this color.
- **Success Text** (`#166534`): Readable status labels and messages on light surfaces.
- **Success Tint** (`#DCFCE7`): Optional background for a success badge with `#166534` text.
- **Clinical semantics:** An administrative completion or connected-device status is not automatically a medical result. Never rely on green alone to say that a health reading is clinically normal. Error/warning/critical-state colors are intentionally **not defined** in this approved palette; define and validate them separately if those states are needed.

### Border & Divider

- **Border Standard** (`#DCE5EE`): Default 1px border for cards, controls, sections, and input fields.
- **Border Subtle** (`rgba(17,24,39,0.06)`): Hairline separators in low-emphasis contexts where a standard border is excessive.
- **Border Emphasis** (`#DCE5EE` at 2px or paired with a shadow): Stronger structural boundary without introducing a competing gray token.
- **Line / Divider** (`#DCE5EE`): Table separators, list row lines, sidebar boundaries.
- **Focus Border** (`#2563EB`): Keyboard focus and input focus.
- **Selected Border** (`#2563EB`): Selected interactive controls when shape and context need a clear selection edge.

### Light Mode Neutrals (Default Theme)

- **Page Neutral** (`#F8FAFC`): Default page fill.
- **Card Neutral** (`#FCFDFE`): Raised or emphasized off-white surface.
- **Panel Neutral** (`#F1F5F9`): Cool tinted section fill.
- **Interaction Neutral** (`#E9EFF5`): Hover and contextual control fill.
- **Structural Neutral** (`#DCE5EE`): Border and divider color.
- **Dark Neutral Foreground** (`#111827`): Primary contrast anchor.

### Overlay

- **Overlay Primary** (`rgba(17,24,39,0.40)`): Dialog/modal scrim that dims the interface while leaving its structure perceptible.
- **Overlay Light** (`rgba(17,24,39,0.24)`): Optional less-prominent transient surface overlay. Do not use it to replace proper contrast testing or a distinct dialog boundary.

### Practical Contrast Reference

Approximate WCAG contrast ratios for important color pairs (computed for solid colors; actual rendered contrast can vary with compositing):

| Foreground | Background | Approx. ratio | Intended use |
|---|---|---:|---|
| `#111827` | `#F8FAFC` | 16.96:1 | Primary text |
| `#475569` | `#F8FAFC` | 7.24:1 | Secondary text |
| `#5B6778` | `#F1F5F9` | 5.24:1 | Tertiary labels on panels |
| `#2563EB` | `#F8FAFC` | 4.94:1 | Normal-size blue links |
| `#F8FAFC` | `#2563EB` | 4.94:1 | Primary button label |
| `#1D4ED8` | `#DBEAFE` | 5.49:1 | Selected-item text |
| `#166534` | `#DCFCE7` | 6.49:1 | Success-pill text |

These examples meet WCAG AA **text contrast** thresholds for normal-sized text. They do not establish full component or product accessibility by themselves. Check other combinations, non-text controls, focus indicators, states, and actual rendered typography.

## 3. Typography Rules

### Font Family

- **Primary:** `Inter Variable`, with fallbacks: `SF Pro Display, -apple-system, system-ui, Segoe UI, Roboto, Oxygen, Ubuntu, Cantarell, Open Sans, Helvetica Neue`.
- **Monospace:** `Berkeley Mono`, with fallbacks: `ui-monospace, SF Mono, Menlo`.
- **OpenType Features:** `"cv01", "ss03"` enabled globally. These are retained from the supplied Linear-inspired system for its geometric, controlled character.
- **Default Font Color:** `#111827`; secondary, tertiary, link, and inverted text use semantic tokens instead of arbitrary opacity changes.

### Hierarchy

The original typographic scale is preserved. Roles of 10–12px are appropriate only for noncritical micro-labels where reading conditions allow; **do not use micro type for key clinical values, safety instructions, medication information, or essential form labels**.

| Role | Font | Size | Weight | Line Height | Letter Spacing | Notes |
|------|------|------|--------|-------------|----------------|-------|
| Display XL | Inter Variable | 72px (4.50rem) | 510 | 1.00 (tight) | -1.584px | Hero headlines, maximum impact |
| Display Large | Inter Variable | 64px (4.00rem) | 510 | 1.00 (tight) | -1.408px | Secondary hero text |
| Display | Inter Variable | 48px (3.00rem) | 510 | 1.00 (tight) | -1.056px | Section headlines |
| Heading 1 | Inter Variable | 32px (2.00rem) | 400 | 1.13 (tight) | -0.704px | Major section titles |
| Heading 2 | Inter Variable | 24px (1.50rem) | 400 | 1.33 | -0.288px | Sub-section headings |
| Heading 3 | Inter Variable | 20px (1.25rem) | 590 | 1.33 | -0.24px | Feature titles, card headers |
| Body Large | Inter Variable | 18px (1.13rem) | 400 | 1.60 (relaxed) | -0.165px | Introduction text, explanations |
| Body Emphasis | Inter Variable | 17px (1.06rem) | 590 | 1.60 (relaxed) | normal | Emphasized body, content subheadings |
| Body | Inter Variable | 16px (1.00rem) | 400 | 1.50 | normal | Standard reading text |
| Body Medium | Inter Variable | 16px (1.00rem) | 510 | 1.50 | normal | Navigation, labels |
| Body Semibold | Inter Variable | 16px (1.00rem) | 590 | 1.50 | normal | Strong emphasis |
| Small | Inter Variable | 15px (0.94rem) | 400 | 1.60 (relaxed) | -0.165px | Secondary body text |
| Small Medium | Inter Variable | 15px (0.94rem) | 510 | 1.60 (relaxed) | -0.165px | Emphasized small text |
| Small Semibold | Inter Variable | 15px (0.94rem) | 590 | 1.60 (relaxed) | -0.165px | Strong small text |
| Small Light | Inter Variable | 15px (0.94rem) | 300 | 1.47 | -0.165px | Noncritical de-emphasized body |
| Caption Large | Inter Variable | 14px (0.88rem) | 510–590 | 1.50 | -0.182px | Category and supplemental headers |
| Caption | Inter Variable | 13px (0.81rem) | 400–510 | 1.50 | -0.13px | Noncritical metadata and timestamps |
| Label | Inter Variable | 12px (0.75rem) | 400–590 | 1.40 | normal | Compact button or auxiliary labels only |
| Micro | Inter Variable | 11px (0.69rem) | 510 | 1.40 | normal | Nonessential micro labels |
| Tiny | Inter Variable | 10px (0.63rem) | 400–510 | 1.50 | -0.15px | Auxiliary overline labels |
| Link Large | Inter Variable | 16px (1.00rem) | 400 | 1.50 | normal | Standard links |
| Link Medium | Inter Variable | 15px (0.94rem) | 510 | 2.67 | normal | Spaced navigation links |
| Link Small | Inter Variable | 14px (0.88rem) | 510 | 1.50 | normal | Compact links |
| Link Caption | Inter Variable | 13px (0.81rem) | 400–510 | 1.50 | -0.13px | Footer, auxiliary links |
| Mono Body | Berkeley Mono | 14px (0.88rem) | 400 | 1.50 | normal | Technical text, units, code |
| Mono Caption | Berkeley Mono | 13px (0.81rem) | 400 | 1.50 | normal | Code labels |
| Mono Label | Berkeley Mono | 12px (0.75rem) | 400 | 1.40 | normal | Noncritical technical metadata |

### Principles

- **510 is the signature weight:** Preserve the source system's restrained UI emphasis, between standard regular and medium.
- **Compression at scale:** Use -1.584px at 72px, -1.408px at 64px, -1.056px at 48px, and -0.704px at 32px. Relax letter-spacing toward normal for smaller text.
- **OpenType as identity:** Keep `"cv01", "ss03"` for Inter throughout the product.
- **Three-tier weight system:** 400 (read), 510 (navigate/emphasize), 590 (strong). Use 300 only where readability is not compromised.
- **Health data legibility:** Present measurements with unambiguous units, dates, and context; give critical values at least body-sized typography rather than micro-label styling.
- **No opacity-only text hierarchy:** Prefer the defined foreground tokens. Dimming important text reduces readability, particularly against light panels.

## 4. Component Stylings

### Buttons

**Ghost Button (Default / Secondary Action)**
- Background: `transparent` or `#FCFDFE` on the canvas.
- Text: `#475569`; important action names may use `#111827`.
- Padding: comfortable; maintain a usable target area.
- Radius: 6px.
- Border: `1px solid #DCE5EE`.
- Hover: `#F1F5F9`.
- Pressed: `#E9EFF5`.
- Focus: `2px solid #2563EB` outline with 2px offset; do not suppress the outline without a replacement.
- Use: Standard actions, secondary CTAs.

**Subtle Button**
- Background: `#F1F5F9` or transparent where the parent already supplies the panel fill.
- Text: `#475569`.
- Hover: `#E9EFF5`.
- Pressed: keep a visible state change through border or shape as well as fill.
- Padding: 0px 6px in compact toolbars, but expand the clickable target to the appropriate touch size.
- Radius: 6px.
- Use: Toolbar actions, contextual buttons.

**Primary Brand Button**
- Background: `#2563EB`.
- Text: `#F8FAFC`.
- Padding: 8px 16px minimum, plus adequate minimum control height.
- Radius: 6px.
- Hover: `#1D4ED8`.
- Pressed: `#1E40AF`.
- Focus: visible 2px `#2563EB` outline with separation from the button edge; a supplementary `rgba(37,99,235,0.18)` halo may be used.
- Disabled: `#E9EFF5` background with `#5B6778` text; expose the disabled state semantically, not with low contrast alone.
- Use: Main actions such as “Save changes”, “Continue”, and “View care plan”.

**Icon Button (Circle)**
- Background: `#FCFDFE` or `#F1F5F9`.
- Icon: `#475569`; active icon may use `#1D4ED8` on `#DBEAFE`.
- Radius: 50%.
- Border: `1px solid #DCE5EE`.
- Hover: `#E9EFF5`.
- Focus: `2px solid #2563EB` outline, offset 2px.
- Use: Close, menu toggle, icon-only actions; include an accessible name.

**Pill Button / Filter Chip**
- Background: `transparent` or `#F1F5F9`.
- Text: `#475569`.
- Padding: 0px 10px 0px 5px, plus an adequate interactive target.
- Radius: 9999px.
- Border: `1px solid #DCE5EE`.
- Selected: `#DBEAFE` background, `#1D4ED8` label, optional `#2563EB` selection border.
- Use: Filters, tags, stateful segment choices.

**Small Toolbar Button**
- Background: `#F1F5F9`.
- Text: `#5B6778`.
- Radius: 2px (original compact-control geometry).
- Border: `1px solid #DCE5EE`.
- Shadow: `rgba(17,24,39,0.03) 0px 1px 2px`.
- Font: 12px weight 510 for supplementary toolbar labels only.
- Hover: `#E9EFF5`.
- Use: Toolbar actions, quick-access controls.

### Cards & Containers

- Background: `#FCFDFE` for elevated cards; `#F1F5F9` for section wells; the page remains `#F8FAFC`.
- Border: `1px solid #DCE5EE` standard, or `1px solid rgba(17,24,39,0.06)` for especially subtle subdivisions.
- Radius: 8px (standard), 12px (featured), 22px (large panels), preserving the source scale.
- Shadow: `0 1px 3px rgba(17,24,39,0.04), 0 8px 24px rgba(17,24,39,0.03)` for genuinely elevated cards. Avoid stacking heavy shadows everywhere.
- Hover: `#F1F5F9` or a slightly stronger border; do not invent hover behavior for noninteractive cards.
- Content: Keep labels, values, date/time, units, and actions in a stable hierarchy. Health metrics should not depend on blue/green color alone for interpretation.

### Inputs & Forms

**Text Area**
- Background: `#FCFDFE`.
- Text: `#111827`.
- Label: `#475569` or `#111827`.
- Border: `1px solid #DCE5EE`.
- Padding: 12px 14px.
- Radius: 6px.
- Placeholder: `#94A3B8` (not a substitute for the field label).
- Focus: visible `#2563EB` focus border/outline.

**Search Input**
- Background: `#FCFDFE` or transparent when it sits in an already bounded search panel.
- Text: `#111827`.
- Placeholder: `#94A3B8`.
- Icon: `#5B6778`.
- Padding: 1px 32px in the original icon-aware layout; provide enough overall height for usability.
- Border: `#DCE5EE` where separation is needed.

**Button-style Input**
- Text: `#475569` for the current value; `#94A3B8` only for placeholder/disabled content.
- Background: `#F1F5F9`.
- Padding: 1px 6px, with a larger clickable target when interactive.
- Radius: 5px.
- Focus: `#2563EB` outline rather than an opaque dark focus shadow.

**Form States**
- Default: standard `#DCE5EE` border.
- Hover: subtle `#E9EFF5` fill or a slightly stronger structural border.
- Focus: `#2563EB` outline/border with a visible focus indication.
- Disabled: explicit `disabled` semantics plus a neutral fill and a readable label; avoid low-contrast values for existing information.
- Validation: display understandable text and iconography alongside any error/warning colors added in a future semantic palette.

### Badges & Pills

**Success Pill**
- Background: `#DCFCE7` for a textual success badge; a standalone status dot may use `#10B981`.
- Text: `#166534`.
- Radius: 9999px for a text pill, 50% for a dot.
- Font: 12–13px weight 510 for readable status labels; 10px is reserved for nonessential technical tagging.
- Use: Administrative completion, connected state, finished tasks. Do not imply medical normality without context.

**Neutral Pill**
- Background: `#F1F5F9` or transparent.
- Text: `#475569`.
- Padding: 0px 10px 0px 5px.
- Radius: 9999px.
- Border: `1px solid #DCE5EE`.
- Font: 12px weight 510 for auxiliary category tags; use larger text where meaning is important.
- Use: Tags, filter chips, category labels.

**Subtle Badge**
- Background: `#F1F5F9`.
- Text: `#475569`.
- Padding: 0px 8px 0px 2px.
- Radius: 2px.
- Border: `1px solid #DCE5EE`.
- Font: 10px weight 510 only for low-priority, nonessential labels.
- Use: Version tags, auxiliary inline labels.

**Selected / Informational Badge**
- Background: `#DBEAFE`.
- Text: `#1D4ED8`.
- Radius: 6px or 9999px, depending on badge type.
- Use: Selected state, optional product-level informational tagging; do not treat brand blue as a universal clinical severity color.

### Navigation

- Light sticky header on `#F8FAFC`; sidebar or grouped navigation region on `#F1F5F9` where useful.
- Product-specific HealthTech logomark left-aligned. Do not reuse Linear's trademarked logomark as the identity of this application.
- Links: Inter Variable 13–14px weight 510, `#475569` for secondary navigation.
- Active/selected: `#DBEAFE` background and `#1D4ED8` text; selected state must also be conveyed semantically.
- Hover: `#E9EFF5` on neutral navigation surfaces, or a text shift toward `#111827`.
- CTA: `#2563EB` primary button or a bordered ghost button.
- Separation: `1px solid #DCE5EE` bottom/side borders as needed.
- Mobile: hamburger collapse, with clear labels and adequately sized hit areas.
- Search: command palette trigger (`/` or `Cmd+K`) when appropriate for the product.

### Image Treatment

- Product screenshots sit on `#F8FAFC` or `#F1F5F9`, using `1px solid #DCE5EE` to define their edge.
- Top-rounded images: `12px 12px 0px 0px` radius, preserving the original treatment.
- Dashboard, care overview, activity, or schedule previews can anchor feature sections; use representative or synthetic health data when presenting public examples.
- Subtle shadow beneath screenshots: `0 2px 8px rgba(17,24,39,0.07)`.
- Avoid strong blue filters, over-saturated health imagery, and decorative content that competes with readings or instructions.

## 5. Layout Principles

### Spacing System

- Base unit: 8px.
- Retained scale: 1px, 4px, 7px, 8px, 11px, 12px, 16px, 19px, 20px, 22px, 24px, 28px, 32px, 35px.
- The 7px and 11px values are optical micro-adjustments, not a reason to abandon the core 8px rhythm.
- Primary rhythm: 8px, 16px, 24px, 32px.

### Grid & Container

- Max content width: approximately 1200px.
- Hero: centered single-column with generous vertical padding.
- Feature sections: 2–3 column grids for cards where the reading order remains clear.
- Full-width light sections with internal max-width constraints, separated through soft surface changes or whitespace.
- Changelog or care timeline: single-column timeline layout where sequencing matters.

### Whitespace Philosophy

- **Light as space:** Off-white negative space creates calm and breathing room. The page background remains visible between structured content blocks.
- **Compressed headlines, expanded surroundings:** Retain display weight 510, narrow tracking, and large surrounding margins without making practical health data feel decorative.
- **Section isolation:** Separate feature sections using generous vertical padding (80px+ on roomy layouts), soft panel shifts, or subtle `#DCE5EE` lines only when needed.
- **Reading clarity:** Dense records, tables, and forms can use tighter component spacing, but primary health details need enough whitespace to prevent accidental associations.

### Border Radius Scale

- Micro (2px): Inline badges, toolbar buttons, subtle tags.
- Standard (4px): Small containers, list items.
- Comfortable (6px): Buttons, inputs, functional elements.
- Card (8px): Cards, dropdowns, popovers.
- Panel (12px): Panels, featured cards, section containers.
- Large (22px): Large panel elements.
- Full Pill (9999px): Chips, filter pills, status tags.
- Circle (50%): Icon buttons, avatars, status dots.

## 6. Depth & Elevation

| Level | Treatment | Use |
|-------|-----------|-----|
| Flat (Level 0) | No shadow, `#F8FAFC` background | App page / default canvas |
| Section (Level 1) | `#F1F5F9` fill, optional `#DCE5EE` edge | Sidebars, grouped regions |
| Surface (Level 2) | `#FCFDFE` fill + `1px solid #DCE5EE` | Cards, fields, containers |
| Inset (Level 2b) | `#F1F5F9` fill + `inset 0 1px 2px rgba(17,24,39,0.03)` | Recessed wells and input groups |
| Ring (Level 3) | `0 0 0 1px rgba(17,24,39,0.06)` | Subtle edge as shadow |
| Elevated (Level 4) | `0 2px 8px rgba(17,24,39,0.07)` + standard border | Dropdowns, floating cards |
| Dialog (Level 5) | `0 10px 32px rgba(17,24,39,0.11), 0 2px 8px rgba(17,24,39,0.05)` | Modals, command palettes, popovers |
| Focus | `2px solid #2563EB` outline with `2px` offset; optional `0 0 0 4px rgba(37,99,235,0.14)` | Keyboard focus and active form controls |

**Shadow Philosophy:** The source Linear system used nearly invisible light strokes and luminance stacking on dark surfaces. In the HealthTech light system, separation comes from **cool off-white surface shifts, dark-on-light 1px borders, and small low-opacity shadows**. Use surface fill before elevation; reserve significant shadows for floating objects and dialogs. No component should depend on a shadow alone to remain identifiable against `#F8FAFC`.

**Surface ownership:** Panel fill communicates grouping; a card border communicates containment; shadow communicates elevation. Avoid combining all three at maximum strength for every element.

## 7. Do's and Don'ts

### Do

- Use Inter Variable with `"cv01", "ss03"` globally to retain the source system's typographic character.
- Use weight 510 for restrained default emphasis and preserve the 400/510/590 hierarchy.
- Apply negative display letter-spacing (for example -1.584px at 72px, -1.056px at 48px).
- Build on `#F8FAFC` for the canvas, `#F1F5F9` for panels, and `#FCFDFE` for elevated surfaces.
- Use `#DCE5EE` for most visible structural borders; use subtler translucent strokes only where the edge remains perceivable.
- Reserve `#2563EB` for primary CTAs and interactive emphasis; use `#1D4ED8` where stronger blue text contrast is required.
- Use `#DBEAFE` for selection fills, paired with `#1D4ED8` or `#1E40AF` text.
- Use `#111827`, `#475569`, and `#5B6778` for a controlled text hierarchy.
- Use `#F8FAFC` for text on filled blue buttons.
- Distinguish focus, hover, selected, disabled, and error states through semantic markup and visible cues in addition to color.
- Preserve clear labels, units, dates, and interaction feedback for personal health information.

### Don't

- Don't use a pure-white fill or foreground; the brightest approved surface is `#FCFDFE`.
- Don't invert source dark-theme colors mechanically; translate their **roles** to the light palette.
- Don't use violet/indigo branding; this product's primary brand accent is blue.
- Don't put normal-size `#2563EB` text on `#DBEAFE`; choose a deeper blue foreground.
- Don't use light-gray placeholders or disabled text for essential medical or administrative information.
- Don't apply decorative blue to every card, chart, border, or status.
- Don't use positive tracking on large display headlines.
- Don't omit the OpenType features or replace the 510 signature weight with default 700 bold.
- Don't use heavy dark shadows as the primary elevation language.
- Don't use green status dots without text/icon/context, or conflate a completed workflow with a clinically normal reading.
- Don't introduce additional clinical warning/error colors without defining their meaning, accessible contrast, and consistency across the product.

## 8. Responsive Behavior

### Breakpoints

Keep the original responsive breakpoints. The mobile ranges intentionally overlap at their shared boundary; implementations should use non-overlapping media-query rules.

| Name | Width | Key Changes |
|------|-------|-------------|
| Mobile Small | <600px | Single column, compact padding |
| Mobile | 600–640px | Standard mobile layout |
| Tablet | 640–768px | Two-column grids begin |
| Desktop Small | 768–1024px | Full card grids, expanded padding |
| Desktop | 1024–1280px | Standard desktop, full navigation |
| Large Desktop | >1280px | Full layout, generous margins |

### Touch Targets

- Keep comfortable button padding and a minimum 6px radius for standard buttons.
- Preserve the source's 13–14px compact navigation where appropriate, but enlarge important labels for accessibility.
- Aim for 44px by 44px touch targets on mobile for primary controls and icon-only buttons; do not confuse visual icon size with tap target size.
- Pill tags retain modest internal text padding while receiving an adequate clickable area.
- Circular icon buttons remain easy to distinguish against pale surfaces through a visible border or hit-area treatment.
- Make search controls discoverable and give form fields persistent labels.

### Collapsing Strategy

- Hero: 72px → 48px → 32px display text, adjusting tracking proportionally.
- Navigation: horizontal links + CTAs → hamburger/menu at 768px.
- Feature cards: 3-column → 2-column → single-column stack.
- Data summaries: keep labels paired with values and units when cards collapse.
- Product screenshots: maintain aspect ratio; reduce peripheral padding before shrinking essential content.
- Care/changelog timeline: maintain readable chronological order through all breakpoints.
- Footer: multi-column → stacked single column.
- Section spacing: 80px+ → approximately 48px on mobile.

### Image Behavior

- Dashboard screenshots retain a consistent `#DCE5EE` edge treatment at all sizes.
- Hero visuals simplify on mobile (fewer floating elements and competing labels).
- Product screenshots use responsive sizing with consistent corner geometry.
- Soft off-white backgrounds allow screenshots to integrate naturally without erasing their boundaries.
- Charts and health metrics must preserve labels, units, and readable contrast at reduced widths.

## 9. Agent Prompt Guide

### Quick Color Reference

- **Primary CTA:** Brand Blue (`#2563EB`) with off-white label (`#F8FAFC`).
- **Primary CTA Hover:** Deep Blue (`#1D4ED8`).
- **Primary CTA Pressed:** Darker Blue (`#1E40AF`).
- **Page Background:** App Off-White (`#F8FAFC`).
- **Panel Background:** Secondary Cool Gray (`#F1F5F9`).
- **Card / Floating Surface:** Elevated Off-White (`#FCFDFE`).
- **Neutral Hover:** Interaction Gray (`#E9EFF5`).
- **Selected Background:** Soft Blue (`#DBEAFE`).
- **Heading / Main Text:** Dark Navy Gray (`#111827`).
- **Body / Secondary Text:** Slate (`#475569`).
- **Metadata Text:** Muted Slate (`#5B6778`).
- **Placeholder / Disabled Only:** `#94A3B8`.
- **Selected Blue Text:** `#1D4ED8`.
- **Border (default):** `#DCE5EE`.
- **Border (subtle):** `rgba(17,24,39,0.06)`.
- **Focus Ring:** 2px `#2563EB` outline with 2px offset.
- **Success:** `#166534` text on `#DCFCE7` tint; optional indicator `#10B981`.

### Example Component Prompts

- **Hero section:** "Create a HealthTech hero on `#F8FAFC`. Set the headline in Inter Variable 48px, weight 510, line-height 1.00, letter-spacing -1.056px, color `#111827`, with `font-feature-settings: 'cv01', 'ss03'`. Use an 18px weight-400 subtitle in `#475569`, line-height 1.60. Add a `#2563EB` primary button with `#F8FAFC` text, 6px radius, 8px 16px padding, and a ghost secondary button with `1px solid #DCE5EE` border."
- **Card:** "Design a patient-friendly summary card on `#F8FAFC`: `#FCFDFE` surface, `1px solid #DCE5EE` border, 8px radius, subtle dark-on-light shadow. Title at 20px Inter Variable weight 590, letter-spacing -0.24px, color `#111827`. Supporting body at 15px weight 400, color `#475569`, letter-spacing -0.165px. Give any displayed measurement an explicit unit."
- **Pill badge:** "Build a neutral pill with `#F1F5F9` background, `#475569` text, 9999px radius, 0px 10px horizontal padding, `1px solid #DCE5EE` border, 12px Inter Variable weight 510. Use `#DBEAFE` and `#1D4ED8` when selected."
- **Navigation:** "Create a light sticky header on `#F8FAFC`. Use Inter Variable 13px weight 510 for navigation labels in `#475569`; apply `#DBEAFE` and `#1D4ED8` for the selected entry. Use a right-aligned `#2563EB` CTA with 6px radius, and a `1px solid #DCE5EE` bottom divider."
- **Command palette:** "Create a command palette with `#FCFDFE` background, `1px solid #DCE5EE` border, 12px radius, and a soft layered shadow. Input at 16px Inter Variable weight 400 and `#111827`. Results use 13–15px weight 510 labels in `#475569` with readable `#5B6778` metadata; preserve keyboard focus indicators."
- **Metric panel:** "Create a simple health activity summary using `#FCFDFE` for the card, `#111827` for the value, `#475569` for its label, and `#2563EB` only for a progress indicator. Show the unit, time range, and descriptive status as text; do not infer clinical thresholds from the brand color."

### Iteration Guide

1. Apply `font-feature-settings: "cv01", "ss03"` to all Inter text.
2. Scale letter-spacing with font size: -1.584px at 72px, -1.056px at 48px, -0.704px at 32px, normal for small text unless specified by the type scale.
3. Preserve weights 400 (read), 510 (emphasize/navigate), 590 (announce).
4. Use native light surface hierarchy: `#F8FAFC` page → `#F1F5F9` panels → `#FCFDFE` elevated cards, with `#DCE5EE` edges where needed.
5. Use `#2563EB` only for primary interactive emphasis; keep hover `#1D4ED8`, pressed `#1E40AF`, and selection background `#DBEAFE`.
6. Use fine dark-on-light borders and subtle shadows, not translucent light-on-dark effects.
7. Keep Berkeley Mono for technical strings and Inter Variable for the product interface.
8. Test text contrast whenever foreground/background combinations change, especially on selected, disabled, and tinted surfaces.
9. Keep semantic health-status messaging separate from brand styling and never communicate important medical meaning by color alone.

## 10. Implementation Tokens & Accessibility Notes

This reference token block translates the approved palette into a light-mode-first CSS theme. The additional dark-blue hover values, supporting text colors, and success semantic values are functional derivatives/extensions of the approved foundation, not alternative brand directions.

```css
:root {
  color-scheme: light;

  /* Core approved palette */
  --color-bg: #F8FAFC;
  --color-surface-elevated: #FCFDFE;
  --color-surface-secondary: #F1F5F9;
  --color-surface-interaction: #E9EFF5;
  --color-border: #DCE5EE;
  --color-accent: #2563EB;
  --color-accent-soft: #DBEAFE;
  --color-text-primary: #111827;

  /* Role-specific extensions */
  --color-text-secondary: #475569;
  --color-text-tertiary: #5B6778;
  --color-text-disabled: #94A3B8;
  --color-accent-hover: #1D4ED8;
  --color-accent-pressed: #1E40AF;
  --color-on-accent: #F8FAFC;
  --color-on-accent-soft: #1D4ED8;
  --color-border-subtle: rgba(17, 24, 39, 0.06);
  --color-overlay: rgba(17, 24, 39, 0.40);
  --color-status-success: #166534;
  --color-status-success-bg: #DCFCE7;
  --color-status-success-indicator: #10B981;

  /* Typography and geometry inherited from Linear-inspired source */
  --font-sans: "Inter Variable", "SF Pro Display", -apple-system,
    system-ui, "Segoe UI", Roboto, Helvetica, sans-serif;
  --font-mono: "Berkeley Mono", ui-monospace, "SF Mono", Menlo, monospace;
  --font-weight-regular: 400;
  --font-weight-medium: 510;
  --font-weight-semibold: 590;
  --radius-control: 6px;
  --radius-card: 8px;
  --radius-panel: 12px;
  --space-unit: 8px;

  /* Depth and keyboard focus */
  --shadow-card: 0 1px 3px rgba(17, 24, 39, 0.04),
    0 8px 24px rgba(17, 24, 39, 0.03);
  --shadow-dialog: 0 10px 32px rgba(17, 24, 39, 0.11),
    0 2px 8px rgba(17, 24, 39, 0.05);
  --focus-outline: 2px solid var(--color-accent);
}

body {
  background: var(--color-bg);
  color: var(--color-text-primary);
  font-family: var(--font-sans);
  font-feature-settings: "cv01", "ss03";
  font-weight: var(--font-weight-regular);
}

.surface-card {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
}

.button-primary {
  background: var(--color-accent);
  color: var(--color-on-accent);
  border: 0;
  border-radius: var(--radius-control);
  font-weight: var(--font-weight-medium);
}
.button-primary:hover { background: var(--color-accent-hover); }
.button-primary:active { background: var(--color-accent-pressed); }

:where(button, a, input, textarea, select, [tabindex]):focus-visible {
  outline: var(--focus-outline);
  outline-offset: 2px;
}

.nav-item[aria-current="page"],
.nav-item[aria-selected="true"] {
  background: var(--color-accent-soft);
  color: var(--color-on-accent-soft);
}
```

### Accessibility & HealthTech Content Checks

- **Normal text:** Check at least 4.5:1 foreground/background contrast (WCAG AA); for large text the relevant threshold is 3:1. Non-text control boundaries and meaningful graphic objects have separate contrast requirements.
- **Focus:** Keep focus indicators visible on every interactive surface, including selected-blue backgrounds and dialogs.
- **Semantic controls:** Use proper labels, button names, validation messages, ARIA state, and keyboard interaction rather than relying on a color cue.
- **Health information:** Show measurement units, date/time or measurement period, and source/context when relevant. Do not interpret health values by an accent color alone.
- **Density:** The source type scale contains 10–12px auxiliary roles; increase size wherever the text is required for a medical decision or to complete a core user flow.
- **Motion and feedback:** Communicate state changes in text where appropriate; honor reduced-motion preferences for transitions.
- **Testing:** Review actual screens on mobile, desktop, and varied display settings. Ratios documented above apply to the specified solid color pairs, not automatically to translucent overlays, shadows, icons, or images.

---

**Design intent:** Maintain Linear's and Attio precision and restraint while making a light, blue-accented HealthTech product feel calm, readable, and trustworthy. The palette intentionally excludes pure-white surfaces and foregrounds.
Follow `DESIGN-RULES.md` too
