**Arquivos do Trasno**: Aplicación de escritorio para Windows, de código aberto e en galego, para converter, comprimir, editar e inspeccionar imaxes, vídeo, audio, PDF e modelos 3D.

- **For:** people who use it often, as an everyday app
- **Screens:** not asked yet
- **Runs on:** desktop, web
- **Feel:** friendly, playful (picked for you)
- **Brand color:** Your brand color (#5FCF6E) is on the buttons exactly, with dark text on it, because white text would be too faint.
- **Decided:** 2 answers you gave, 5 picked for you; everything else uses sourced defaults
- **Standards:** 291 house standards followed (locked, house standards v4); see the Standards section
- **Next:** zoom into Colors, Typography, Layout, or stop here: each level works
- **Licence risks:** none found

> Generated from `opendesigner/state.json` and `opendesigner/decisions.md`. Made with OpenDesigner 0.3.1. `opendesigner/tokens/` is the source of truth. Change the system with `engine.py set ...` and `engine.py build`, not by editing this file (notes inside `<!-- od:keep -->` blocks survive). Each section opens with one plain sentence; open **More** for the designer and engineer detail.

## Overview

> Zoom: sketch (0 of 3), 3 decisions in this area. To zoom in, answer Q-dir-01, Q-dir-02.
<!-- od:zoom area=overview level=0 -->

**Arquivos do Trasno feels friendly and playful. Screens are comfortable, in a flat style.**
Designers: visual style: flat, tonal, glass · Code: `preset in state.json`

<details><summary>More</summary>

**Your answers here** (each one either sets tokens or is a rule for this section):
- Where is the project now: an idea, being built, close to launch, or live with real users? **Close to launch** (assumed; owner input; Q-scope-07)

**Intent.** Aplicación de escritorio para Windows, de código aberto e en galego, para converter, comprimir, editar e inspeccionar imaxes, vídeo, audio, PDF e modelos 3D.

- Audience: people who use it often, as an everyday app
- The memorable thing: not recorded yet (Q-brand-02)
- Surfaces: not recorded yet (Q-scope-06)
- Platforms: desktop, web; inputs: pointer, touch
- Direction: preset flat2 (the default until Q-dir-01 is answered); feel friendly, playful (words picked for you: friendly, playful)

**Principles (ranked).**
No principles recorded yet (Q-brand-07). Until they are, the dials act as the tie-breakers below.

**Dial positions.** Three posture dials set the overall stance. Five character dials fine-tune it.

- **Expression 60/100** (productive to expressive: leaning expressive): type ratio 1.25, hero moments: significant-moments, expressive motion: significant-moments, brand color role: accent+signature-surface, containment: mixed.
- **Brand presence 50/100** (native to brand-led: balanced): typeface suggestion: open-neutral-face, brand color placement: accent+signature-surface, native component share about 80%, voice guidance: short.
- **Density 50/100** (spacious to compact: balanced): body 16px on the web, default control 40px, density mode comfortable, icons 20px; hit areas never shrink.
- **Energy 55/100** (calm to energetic: balanced): standard easing [0.77, 0, 0.175, 1], duration multiplier 1.02, spring damping 1.0, heading weight 650, navigation: neutral.
- **Roundness 90/100** (sharp to soft: soft): control radius 12px (the band gives 16px; until someone picks a corner shape, Q-shape-01, corners stay between 6 and 12px), icon corners rounded, caps round.
- **Depth 30/100** (flat to deep: flat): depth model: ring+faint-shadow.
- **Colorfulness 65/100** (monochrome to vivid: leaning vivid): scheme expressive, accent chroma 62.7 (HCT), 3 accent(s), surfaces: tinted-containers.
- **Warmth 80/100** (cool/formal to warm/friendly: warm/friendly): neutral tint chroma 0.015 at hue 58, borders: soft, sentence-case, contractions encouraged.

**Zoom by area** (sketch, broad, defined, detailed). You can stop at any level; each one works.

| Area | Zoom | Next questions |
|---|---|---|
| Colors | sketch | Q-color-02 |
| Typography | sketch | Q-type-01 |
| Layout | sketch | Q-space-01, Q-space-02, Q-space-03 |
| Elevation & Depth | sketch | Q-depth-01 |
| Shapes | sketch | Q-shape-01 |
| Components | sketch | Q-comp-01, Q-comp-02, Q-state-01 |
| Motion | sketch | Q-motion-01 |
| Modes and Themes | sketch | Q-theme-03 |
| Iconography and Imagery | sketch | Q-icon-01, Q-icon-02, Q-icon-06 |
| Content and Voice | sketch | Q-voice-01, Q-voice-02, Q-voice-03 |
| Accessibility | sketch | Q-aud-02, Q-aud-03, Q-aud-04 |
| Platforms and Devices | sketch | Q-plat-05, Q-plat-10, Q-plat-08 |

The tokens in `opendesigner/tokens/` (DTCG 2025.10 with `opendesigner.resolver.json`) are the source of truth. This file is a generated view.

Decisions: D-0007 (answers.Q-scope-07), D-0019 (answers.Q-aud-01), D-0021 (macros).

</details>

**Taste card.** How it looks and feels, in five lines for you, your team and any AI that builds with it. Nothing on it is invented: each line says where it comes from.
Designers: style brief, mood in words · Code: `taste.brief, taste.never`

| Line | What it says | From |
|---|---|---|
| In your words | Not given yet. Describe the look as a picture, not adjectives (for example, "a quiet library at night"). | not asked yet |
| Color | Three accent colors: your brand color `#5fcf6e` (exactly) marks buttons, links and focus; `#00828e`, `#937010` add highlights. Everything else stays neutral. Keep the accent to small areas, about 15% of a screen at most (a rule of thumb, DC-L15-03). | brand color; colorfulness 65/100 |
| Shape | Buttons and fields have 12px corners; cards have 16px corners. Cards get a faint outline and a very soft shadow. | roundness 90/100, depth 30/100 |
| Motion | Balanced. Most movements take 200ms. They turn into fades when a person asks for less motion. | energy 55/100 |
| Never | Nothing yet. Say what it should never look like, or rate real screens in the taste poll. | not asked yet |

## Colors

> Zoom: sketch (0 of 3), 2 decisions in this area. To zoom in, answer Q-color-02.
<!-- od:zoom area=color level=0 -->

**Three accent colors: your brand color `#5fcf6e` (exactly) marks buttons, links and focus; `#00828e`, `#937010` add highlights.** Everything else stays neutral.
Designers: primary or brand accent · Code: `color.bg.accent`

<details><summary>More</summary>

**Intent.** Brand color `#5fcf6e` drives the accent ramp's hue; the Colorfulness dial (65) sets its chroma (scheme expressive, peak HCT chroma 62.67). The brand's role is **accent+signature-surface** and it sits closest to accent step 9. Your brand color #5FCF6E is on the buttons exactly (dark text on it, because white text would be too faint).

**How ramps are built.** Each hue gets twelve steps in OKLCH, placed by contrast. Step 8 reaches at least 3:1 (edges and focus). Steps 10 and 11 reach the text minimum on backgrounds 1-4. Step 12 reaches at least 7:1. Steps 2-6 are spaced evenly in lightness between step 1 and step 7. Light and dark ramps are solved separately. Dark solids are lighter and carry dark text (dark mode is a separate mapping, never an inversion). Text minimum: 4.5:1 (WCAG 2.2 AA).

| Ramp | Light steps 1-12 | Solid carries |
|---|---|---|
| `accent` | `#fafefa` `#eef6ee` `#dff0e0` `#cfead0` `#c0e3c1` `#b0ddb2` `#9fd7a3` `#4d9655` `#5fcf6e` `#5bbe68` `#16742b` `#153b1a` | dark |
| `accent2` | `#fafdfe` `#eef5f6` `#deeef0` `#cee8eb` `#bde1e5` `#acdae0` `#9bd3da` `#44919a` `#00828e` `#007580` `#00616a` `#0f393e` | light |
| `accent3` | `#fefdf9` `#f7f4ee` `#f1ebde` `#ece2cd` `#e6d9bc` `#e1d0ac` `#dcc89b` `#9d8349` `#937010` `#856506` `#6f5400` `#3f3213` | light |
| `danger` | `#fffcfc` `#fff1ef` `#ffe6e3` `#ffdbd6` `#ffcfc9` `#ffc4bc` `#ffb8af` `#d3655b` `#d53e38` `#c33530` `#a42825` `#5a211d` | light |
| `info` | `#fbfdff` `#eaf6ff` `#d9efff` `#c7e7ff` `#b5e0ff` `#a3d8ff` `#8fd1ff` `#008ed1` `#007cb7` `#0070a6` `#005c8a` `#003755` | light |
| `neutral` | `#fcfdfe` `#f2f4f6` `#e9ecef` `#e0e3e7` `#d7dbdf` `#ced2d7` `#c5cad0` `#82878d` `#71767c` `#60656b` `#54585e` `#222428` | light |
| `success` | `#f9fefa` `#ecf6ed` `#daf1de` `#c8ecce` `#b5e6bd` `#a1e1ad` `#8cdb9d` `#21994c` `#00883d` `#007b36` `#00652c` `#003d17` | light |
| `warning` | `#fffcf8` `#f8f4ec` `#f5eada` `#f1e1c6` `#eed8b3` `#ebce9f` `#e7c58b` `#aa7f2d` `#efb23a` `#dca332` `#845d00` `#463002` | dark |

**Step-distance guarantee** (neutral, light; WCAG contrast of a foreground step on background steps 1-5):

| Foreground | on 1 | on 2 | on 3 | on 4 | on 5 |
|---|---|---|---|---|---|
| neutral 8 | 3.56 | 3.28 | 3.05 | 2.81 | 2.60 |
| neutral 9 | 4.50 | 4.15 | 3.86 | 3.56 | 3.29 |
| neutral 10 | 5.77 | 5.33 | 4.96 | 4.57 | 4.22 |
| neutral 11 | 7.03 | 6.49 | 6.03 | 5.56 | 5.14 |
| neutral 12 | 15.26 | 14.10 | 13.11 | 12.07 | 11.17 |

**States.** Hover is at least 1.15:1 from its fill and pressed at least 1.3:1, as shade steps (DC-L01-17). Where a color is unknown at design time, use the state opacities in `opacity.state.*`.

| Fill | Base | Hover | Pressed |
|---|---|---|---|
| `color.bg.action.primary` | `#5fcf6e` | `#5bbe68` (1.18:1) | `#50b35e` (1.34:1) |
| `color.bg.accent.bold` | `#5fcf6e` | `#5bbe68` (1.18:1) | `#50b35e` (1.34:1) |

**Semantic roles** (use these, never the ramps):

| Token | Light | Dark | Use |
|---|---|---|---|
| `color.bg.accent.bold` | `#5fcf6e` | `#5fcf6e` | Solid accent fill. |
| `color.bg.accent.bold-hover` | `#5bbe68` | `#76df82` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.accent.bold-pressed` | `#50b35e` | `#82eb8e` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.accent.subtle` | `#dff0e0` | `#1a2b1b` | Selected rows, accent badges. |
| `color.bg.accent.subtle-hover` | `#cfead0` | `#1a351d` | Hover on accent subtle fills. |
| `color.bg.accent.subtle-pressed` | `#c0e3c1` | `#1a3f1f` | Pressed accent subtle fills. |
| `color.bg.accent2.bold` | `#00828e` | `#39909a` | accent2 solid fill. |
| `color.bg.accent2.bold-hover` | `#007580` | `#489da8` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.accent2.bold-pressed` | `#006c77` | `#53a7b2` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.accent2.subtle` | `#deeef0` | `#1c292b` | accent2 subtle fill. |
| `color.bg.accent3.bold` | `#937010` | `#9b803f` | accent3 solid fill. |
| `color.bg.accent3.bold-hover` | `#856506` | `#a98d4c` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.accent3.bold-pressed` | `#7b5d00` | `#b39656` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.accent3.subtle` | `#f1ebde` | `#2c271c` | accent3 subtle fill. |
| `color.bg.action.primary` | `#5fcf6e` | `#5fcf6e` | Primary action. One per view (DC-L15-03, L15 P08). |
| `color.bg.action.primary-hover` | `#5bbe68` | `#76df82` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.action.primary-pressed` | `#50b35e` | `#82eb8e` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.brand` | `#5fcf6e` | `#5fcf6e` | Brand fill for signature surfaces; carries text.on-brand (brandAnchor rule, DC-L01-09). |
| `color.bg.danger.bold` | `#d53e38` | `#d75e54` | Danger solid fill. |
| `color.bg.danger.bold-hover` | `#c33530` | `#e66c61` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.danger.bold-pressed` | `#b92a27` | `#f1766a` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.danger.subtle` | `#ffe6e3` | `#38221f` | Danger banner and badge background. |
| `color.bg.disabled` | `#e9ecef` | `#2b2e30` | Disabled control fill. |
| `color.bg.field` | `#ffffff` | `#232527` | Text field fill: outlined fields sit on the raised surface (Q-form-01). |
| `color.bg.info.bold` | `#007cb7` | `#008bcd` | Info solid fill. |
| `color.bg.info.bold-hover` | `#0070a6` | `#0099e0` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.info.bold-pressed` | `#006799` | `#00a3ee` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.info.subtle` | `#d9efff` | `#0b2a3d` | Info banner and badge background. |
| `color.bg.inverse` | `#222428` | `#e5e9ee` | Tooltips and toasts: the inverted surface. |
| `color.bg.neutral.subtle` | `#e9ecef` | `#34363a` | Secondary buttons, chips, hovered rows. |
| `color.bg.neutral.subtle-hover` | `#e0e3e7` | `#3c3f43` | Hover: +1 step (DC-L01-17). |
| `color.bg.neutral.subtle-pressed` | `#d7dbdf` | `#45484d` | Pressed or selected: +2 steps (DC-L01-17). |
| `color.bg.success.bold` | `#00883d` | `#009845` | Success solid fill. |
| `color.bg.success.bold-hover` | `#007b36` | `#1da650` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.success.bold-pressed` | `#007132` | `#2eb05a` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.success.subtle` | `#daf1de` | `#182b1c` | Success banner and badge background. |
| `color.bg.warning.bold` | `#efb23a` | `#e6b55e` | Warning solid fill. |
| `color.bg.warning.bold-hover` | `#dca332` | `#f7c56e` | Hover: one step, at least 1.15:1 from the solid (DC-L01-17). |
| `color.bg.warning.bold-pressed` | `#d19923` | `#ffd388` | Pressed: at least 1.3:1 from the solid. |
| `color.bg.warning.subtle` | `#f5eada` | `#2f2618` | Warning banner and badge background. |
| `color.border.accent` | `#4d9655` | `#3a9347` | Selected card or accent outline. |
| `color.border.danger` | `#d3655b` | `#ca6359` | Danger boundary, for example an invalid field (3:1). |
| `color.border.default` | `#ced2d7` | `#3c3f43` | Outlines (soft borders). |
| `color.border.focus` | `#16742b` | `#87e791` | Focus ring (3:1 against every surface, DC-L04-09). |
| `color.border.info` | `#008ed1` | `#0089c9` | Info boundary, for example an invalid field (3:1). |
| `color.border.input` | `#82878d` | `#7c8086` | Text field and select borders (3:1, DC-L01-16). |
| `color.border.separator` | `#e9ecef` | `#2b2e30` | Hairlines between table rows and list items, lighter than border.subtle; decorative, so never the only cue (DC-L23-05). |
| `color.border.strong` | `#82878d` | `#7c8086` | Boundaries that are the only cue: inputs, checkboxes, switches (3:1, WCAG 1.4.11). |
| `color.border.subtle` | `#d7dbdf` | `#34363a` | Dividers (soft borders: Warmth 67+, Linear 2026 S-L06-067). |
| `color.border.success` | `#21994c` | `#27944b` | Success boundary, for example an invalid field (3:1). |
| `color.border.warning` | `#aa7f2d` | `#a37b30` | Warning boundary, for example an invalid field (3:1). |
| `color.icon.accent` | `#16742b` | `#87e791` | Selected and interactive icons. |
| `color.icon.default` | `#54585e` | `#a6abb2` | Icons beside text; same color as secondary text (DC-L05-08). |
| `color.icon.on-accent` | `#222428` | `#141516` | Icons on solid accent fills. |
| `color.icon.subtle` | `#60656b` | `#91969d` | Decorative icons. |
| `color.overlay.scrim` | `#222428 @0.45` | `#000000` | Modal backdrop (DC-L04-18; Q-depth-06). |
| `color.surface.base` | `#fcfdfe` | `#1c1d1f` | The page background. |
| `color.surface.nav` | `#fcfdfe` | `#1c1d1f` | Navigation on the page background. |
| `color.surface.overlay` | `#ffffff` | `#2b2e30` | Menus, popovers, dialogs; always with elevation.floating or elevation.overlay. |
| `color.surface.raised` | `#ffffff` | `#232527` | Cards and panels on the page. |
| `color.surface.sunken` | `#f2f4f6` | `#141516` | Wells, code blocks, page gutters. |
| `color.surface.tinted` | `#eef6ee` | `#192119` | Tinted container (Colorfulness 50+, L09 X rubric; Q-color-05). |
| `color.text.accent` | `#16742b` | `#87e791` | Accent text and selected labels. |
| `color.text.accent2` | `#00616a` | `#6bb8c2` | accent2 text. |
| `color.text.accent3` | `#6f5400` | `#c4a96d` | accent3 text. |
| `color.text.danger` | `#a42825` | `#ff8b7f` | Danger text and icons; pair with an icon or label (DC-L01-15). |
| `color.text.disabled` | `#82878d` | `#7c8086` | Disabled labels; exempt from contrast minimums (WCAG 1.4.3 exception). |
| `color.text.info` | `#005c8a` | `#39b5ff` | Info text and icons; pair with an icon or label (DC-L01-15). |
| `color.text.inverse` | `#ffffff` | `#141516` | Text on color.bg.inverse (tooltips, toasts). |
| `color.text.link` | `#16742b` | `#87e791` | Links. Underline them in running text so color is not the only cue. |
| `color.text.on-accent` | `#222428` | `#141516` | Text and icons on color.bg.accent.bold* (auto-picked light or dark, S-L06-094). |
| `color.text.on-accent2` | `#ffffff` | `#141516` | Text on accent2 solid. |
| `color.text.on-accent3` | `#ffffff` | `#141516` | Text on accent3 solid. |
| `color.text.on-action` | `#222428` | `#141516` | Text on the primary action fill. |
| `color.text.on-brand` | `#222428` | `#141516` | Text on color.bg.brand. |
| `color.text.on-danger` | `#ffffff` | `#141516` | Text on bg.danger.bold. |
| `color.text.on-info` | `#ffffff` | `#141516` | Text on bg.info.bold. |
| `color.text.on-success` | `#ffffff` | `#141516` | Text on bg.success.bold. |
| `color.text.on-warning` | `#222428` | `#141516` | Text on bg.warning.bold. |
| `color.text.primary` | `#222428` | `#e5e9ee` | Body copy, headings, labels (guaranteed >= 7:1 on steps 1-5). |
| `color.text.secondary` | `#54585e` | `#a6abb2` | Supporting text (guaranteed >= the text minimum on surfaces and neutral fills 1-4). |
| `color.text.success` | `#00652c` | `#51c070` | Success text and icons; pair with an icon or label (DC-L01-15). |
| `color.text.tertiary` | `#54585e` | `#a6abb2` | Captions and placeholders: the same grey as secondary, so text keeps three greys (primary, secondary, disabled; owner, 2026-11-05). |
| `color.text.warning` | `#845d00` | `#fecf80` | Warning text and icons; pair with an icon or label (DC-L01-15). |

**Use / avoid.**
- Use one solid accent fill per view for the primary action; spend chroma on small, meaningful elements.
- Avoid gray text on colored fills; use the matching `text.on-*` role.
- Avoid color as the only signal: pair status colors with an icon or label.
- Charts: no chart palette yet (Q-color-19). Take chart colors from the status and accent hues, and check them separately.

**Tailwind classes:** `bg-action-primary`, `hover:bg-action-primary-hover`, `active:bg-action-primary-pressed`, `text-on-action`, `bg-surface-base`, `bg-surface-raised`, `text-fg-primary`, `text-fg-secondary`, `text-fg-link`, `border-line-subtle`, `border-line-input`, `outline-line-focus`, `bg-danger-bold`.

Decisions: D-0004 (raw.brandColor), D-0006 (answers.Q-theme-01).

</details>

## Typography

> Zoom: sketch (0 of 3), 1 decision in this area. To zoom in, answer Q-type-01.
<!-- od:zoom area=typography level=0 -->

**Body text is 16px, and each larger size is about 1.25 times the one below.**
Designers: type scale: base size and ratio · Code: `font.size.14`

<details><summary>More</summary>

**Intent.** Body text is 16px; sizes follow `round(16 x 1.25^n)` for n from -2 to 4, reaching about 2.5x body (product surfaces only). Sizes: 13, 16, 20, 25, 31, 39px.

- Text face: Atkinson Hyperlegible Next Variable, system-ui, -apple-system, Segoe UI, Roboto, Helvetica Neue, Arial, sans-serif. Licence status (hook H-type): **pending**.
- Licence by platform: text: Atkinson Hyperlegible Next Variable: licensed for web, apps, self-hosting.
- Weights: body 400, label 500, heading 650 (two or three distinct weights per view).
- Emphasized variants: off (they turn on at Expression 67 and up, DC-L02-11).
- Line heights snap to a 4px grid: 1.5 up to 17px, 1.4 to 26px, 1.25 to 44px, 1.12 above.
- Letter spacing, one value per job (`font.tracking.*`): +0.02em at 11-12px, 0 for body, -0.01em from 32px, -0.02em from 48px.
- Capitals never get negative tracking: small capital labels +0.06em, capital headings 0 to +0.02em. Set `font` and `letter-spacing` together, or use the `.ds-text-*` classes: the `font` shorthand alone drops tracking.
- Text scaling: CSS sizes are rem, line heights unitless, so text scales to 200% (WCAG 1.4.4); containers must grow with it.
- Scripts: Latin only; add scripts in raw.scripts to get per-script line heights (DC-L02-25).
- Per page: at most 8 sizes, 3 weights (400, 500, 650) and 2 families, whole px only (`limits` in `tokens/opendesigner.meta.json`).

| Style | Size/line (px) | Weight | Tracking | Use |
|---|---|---|---|---|
| `text.body.lg` | 20/28 | 400 | 0em | Long-form reading, onboarding copy. |
| `text.body.md` | 16/24 | 400 | 0em | Default UI text: forms, tables, descriptions. |
| `text.body.sm` | 13/20 | 400 | 0em | Helper text, timestamps, captions. |
| `text.code.md` | 16/24 | 400 | 0em | Code blocks, token names. |
| `text.code.sm` | 13/20 | 400 | 0em | Inline values, dense code rows. |
| `text.headline.md` | 39/48 | 650 | -0.01em | Page and section titles. |
| `text.headline.sm` | 31/40 | 650 | 0em | Page and section titles. |
| `text.label.lg` | 16/24 | 500 | 0em | Button and control labels. |
| `text.label.md` | 13/20 | 500 | 0em | Small control labels, badges, dense tabs. |
| `text.label.sm` | 13/20 | 500 | 0.06em caps | Overlines in capitals with wide tracking (DC-L02-18). |
| `text.numeric.lg` | 25/36 | 650 | 0em | Key numbers: balances, KPIs. |
| `text.numeric.md` | 16/24 | 400 | 0em | Amounts and table figures: tabular digits line up in columns; right-align numeric columns. |
| `text.title.lg` | 25/36 | 650 | 0em | Card, panel and group titles. |
| `text.title.md` | 20/28 | 650 | 0em | Card, panel and group titles. |
| `text.title.sm` | 16/24 | 650 | 0em | Table headers, field-group titles. |

**Use / avoid.**
- Use at most three sizes and two weights in one view; let color and weight carry hierarchy before size.
- Avoid all caps for sentences and in scripts without case; use weight for emphasis there.

**Tailwind classes:** `text-body-md`, `text-label-lg`, `text-title-md`, `text-headline-md`.

Decisions: D-0017 (raw.textFace).

</details>

## Layout

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-space-01, Q-space-02, Q-space-03.
<!-- od:zoom area=layout level=0 -->

**Spacing comes in steps of 4px, and screens use comfortable spacing by default.**
Designers: spacing scale, 0 to 96 · Code: `space.0 to space.96`

<details><summary>More</summary>

**Intent.** Everything sits on a 4px unit: ladder 0, 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96. Density is a mode on the semantic layer (default **comfortable**). Primitives and minimum tap areas never change.

| Density | inset xs/sm/md/lg/xl | section sm/md/lg | control sm/md/lg | row sm/md/lg/two-line | outer:inner |
|---|---|---|---|---|---|
| spacious | 6 / 12 / 16 / 20 / 32 | 64 / 80 / 96 | 40 / 48 / 56 | 52 / 56 / 64 / 80 | 3.5 |
| comfortable (default) | 4 / 8 / 12 / 16 / 24 | 32 / 40 / 64 | 32 / 40 / 48 | 44 / 48 / 56 / 72 | 2.5 |
| compact | 2 / 6 / 8 / 12 / 20 | 16 / 24 / 40 | 24 / 32 / 40 | 36 / 40 / 48 / 64 | 2.0 |

- Target sizes: pointer 24px, touch 44px; `size.target.min` is the floor for the primary input, and `space.target.gap` separates adjacent targets.
- Inner gaps stay below outer gaps (at least half), so grouping reads without borders (L15 P16).
- One container for the whole site: `layout.container.max` (1280px). Columns 4 / 8 / 12 (`layout.columns.*`), gutters 16 / 24px, margins 16 / 24px; paragraphs at most 65ch (`layout.measure.prose`), leads 52ch (DC-L03-15, DC-L03-16, DC-L02-17).
- Alignment: A centred hero is fine; within a section the heading and its content share one alignment.

**Use / avoid.**
- Parents own spacing (padding and gap); children never set outer margins.
- Avoid values off the ladder; if one is needed, add it as a decision.

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## Elevation & Depth

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-depth-01.
<!-- od:zoom area=elevation level=0 -->

**Cards get a faint outline and a very soft shadow.**
Designers: flat, shadowed, tonal or glass · Code: `$extensions.opendesigner.model`

<details><summary>More</summary>

**Intent.** Depth 30/100 selects the **ring+faint-shadow** model. Shadow alpha in light mode is 0.107; dark mode doubles it and lifts surfaces by lightness instead (sunken, base, raised, overlay).

| Token | Layers (x y blur spread) | Use |
|---|---|---|
| `elevation.raised` | 0 0 0 1, 0 1 2 0 | Cards that lift on hover, sticky headers, dragged rows. |
| `elevation.floating` | 0 0 0 1, 0 1 2 0, 0 4 12 0 | Menus, popovers, dropdowns. |
| `elevation.overlay` | 0 0 0 1, 0 1 2 0, 0 12 32 0 | Dialogs and sheets (with color.overlay.scrim). |

- Borders: 1px default, 2px selected and focus, 4px emphasis; width changes use an inset shadow so layout does not jump.
- Stacking: page < raised < floating (menus) < overlay (dialogs with `color.overlay.scrim`).
- No materials or glass in this system.

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## Shapes

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-shape-01.
<!-- od:zoom area=shape level=0 -->

**Buttons and fields have 12px corners; cards have 16px corners.**
Designers: corner radius, sharp to pill · Code: `radius.control, CSS border-radius`

<details><summary>More</summary>

**Intent.** Roundness 90/100 gives controls a **12** radius; details 6px, containers 16px, overlays 24px, people full. Controls under 32px tall use `radius.control-sm` (8).
Buttons are **rounded**; square and pill corners are a choice (Q-shape-01), never a default.

- One corner per kind of thing: `radius.control` for buttons and CTAs, inputs, selects, chips, toggles, tabs, carousel arrows; `radius.container` for cards, panels, photos and media; `radius.overlay` for dialogs, sheets, popovers; `radius.detail` for checkboxes, tags, small badges; `radius.person` for avatars, dots and status pips only.
- Nested radius: inner = max(outer - padding, smallest step). Here it is 2px inside a container with `space.inset.lg` padding. Equal radii on nested shapes look uneven (DC-L04-05).
- Focus ring: 2px, 2px offset, radius = control radius + offset (14).
- Icons: rounded corners, round caps.
- Corners (Q-shape-04, DC-L04-04): the radii are the same everywhere; iOS draws them continuous (`DS.Radius.shape(...)`, `.continuous`), Android circular (`RoundedCornerShape`), the web and Figma circular.
- Signature shapes: none generated; a brand shape library is a designer asset (hook H-motif).

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## Components

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-comp-01, Q-comp-02, Q-state-01.
<!-- od:zoom area=components level=0 -->

**Version 1 plans 25 components, all built from these tokens.**
Designers: core set of about 25 components · Code: `button.* (proposed), APG pattern`

<details><summary>More</summary>

**Intent.** v1 components use the semantic tokens only. Base: not chosen yet (Q-comp-01). Buttons are 40px tall (comfortable), 12 radius, `text.label.lg`; one primary action per view.

| Component | Status |
|---|---|
| button | planned (tokens ready) |
| text-field | planned (tokens ready) |
| textarea | planned (tokens ready) |
| select | planned (tokens ready) |
| checkbox | planned (tokens ready) |
| radio | planned (tokens ready) |
| switch | planned (tokens ready) |
| slider | planned (tokens ready) |
| tabs | planned (tokens ready) |
| tooltip | planned (tokens ready) |
| popover | planned (tokens ready) |
| dialog | planned (tokens ready) |
| menu | planned (tokens ready) |
| progress-bar | planned (tokens ready) |
| spinner | planned (tokens ready) |
| banner | planned (tokens ready) |
| badge | planned (tokens ready) |
| avatar | planned (tokens ready) |
| card | planned (tokens ready) |
| list | planned (tokens ready) |
| table | planned (tokens ready) |
| link | planned (tokens ready) |
| breadcrumbs | planned (tokens ready) |
| side-navigation | planned (tokens ready) |
| accordion | planned (tokens ready) |

**Contested policies:** Validation timing and disabled submit: pending; Field style: pending; Toasts: pending; Undo versus confirm: pending; Danger actions: pending; Button emphasis levels: pending.

**Tables** (defaults; Q-pattern-07 changes them):
- Rows: a 1px `color.border.subtle` line between rows, none around the table (Carbon's default).
- Row heights: `size.row.sm` (dense), `size.row.md` (default), `size.row.lg` (roomy) and `size.row.two-line`; the header row matches the body rows (Carbon, S-L08-092).
- Header: stays in view while the rows scroll (`position: sticky`), with a scroll-edge fade where rows pass under it, never a 1px line (STD-visual-details-15).
- Numbers: right-aligned in `text.numeric.*` (tabular digits), and their column headers right-aligned too, so digits line up and compare at a glance (DC-L19-121, DC-L02-26).
- Text, status chips, row checkboxes and targets follow the same tokens as the rest of the system; a chip carries a word, never color alone (WCAG 1.4.1).

**Use / avoid.**
- Every interactive component shows hover, pressed, focus-visible and disabled states from the tokens.
- Inputs keep a 3:1 boundary (`color.border.input`); signifier strength: balanced.
- Component docs belong in `opendesigner/components/<name>.md` (spec 7.8).

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## States

**Every screen says what is happening while it loads, when it is empty, and when something goes wrong.** The overview draws each one with these tokens.

- **Waiting** (Q-state-08, not decided yet: the default is shown): Nothing under 1 second; a looping sign from 1 to 10 seconds; percent done over 10 seconds. No tokens until it is answered: `engine.py pick Q-state-08 <option>`.
- **Empty screens** (Q-pattern-04, not decided yet: the default is shown): Each empty screen says which kind it is (first use, cleared, no results, no permission, error), why it is empty, and one next step. No tokens until it is answered: `engine.py pick Q-pattern-04 <option>`.
- **Errors** (Q-form-03, not decided yet: the default is shown): Each error sits next to the field it is about. No tokens until it is answered: `engine.py pick Q-form-03 <option>`.
- **Not found (404) and offline**: a page that says what happened in one line and offers a way on (home, search or retry), in the page's own layout, never a dead end.
- **Every screen that shows data** has its loading, empty and error state before it ships; skeletons take the final layout's shape, so nothing jumps when the content arrives. Reduced motion stops the skeleton shimmer.

## Status and Feedback

**Each status colour has one meaning, always with an icon or a word, and every action shows what happened.**

| Status | Means | Use it | Tokens (text and icon, background, border) | Say it with |
|---|---|---|---|---|
| Success | an action finished as asked | after an action that worked, and for a state that is good (paid, verified) | `--ds-color-text-success`, `--ds-color-bg-success-subtle`, `--ds-color-border-success` | the circle-check icon and a word such as "Saved" |
| Warning | something needs attention soon, but nothing is lost yet | for a risk the person can still avoid: a limit near, a date close, a change that is hard to undo | `--ds-color-text-warning`, `--ds-color-bg-warning-subtle`, `--ds-color-border-warning` | the triangle-alert icon and a word such as "Check" |
| Danger | something failed or would destroy data | for errors, failures, and actions that delete or cannot be undone | `--ds-color-text-danger`, `--ds-color-bg-danger-subtle`, `--ds-color-border-danger` | the circle-x icon and a word such as "Error" |
| Info | neutral news the person may want | for neutral facts and tips; never for errors or success | `--ds-color-text-info`, `--ds-color-bg-info-subtle`, `--ds-color-border-info` | the info icon and a word such as "Note" |

- **Never colour alone.** Every status carries an icon or a word as well as its colour (WCAG 2.2 SC 1.4.1).
- **One meaning per colour.** Success, warning and danger colours appear only for that status, never for a brand accent, a primary button or decoration. Delete buttons take danger; everything else takes the action colour.
- **After every action, the person sees what happened.** In place, next to what changed, or in a banner over the section; a toast only for a low-stakes confirmation that carries Undo (default until Q-form-04 is answered).
- **Banners and toasts:** a success banner may close after 4 s; the rare toast (low stakes, with Undo) stays until the person acts or closes it; an error stays until the person dismisses it or fixes the cause, and never closes on its own.
- **Undo:** a confirm step for what cannot be undone, and an Undo that stays until used or closed for the rest (default, from house standard STD-visual-details-40).
- The same map, for tools and agents: `build/status.json`.

## Do's and Don'ts

**These rules keep screens clear and easy to read.** The engine checks the ones it can.
Designers: guardrails and design linting · Code: `engine.py validate, engine.py review`

<details><summary>More</summary>

**Enforced by construction.** The generator cannot break these:
- text contrast 4.5:1, and 3:1 for edges, in every mode
- text on colored fills picked automatically
- tap areas pointer 24px, touch 44px
- a 2px focus ring with a 2px offset
- a reduced-motion mode
- inner spacing smaller than outer spacing, three text colors, and nested radii

**Do**
- Use semantic tokens; never raw hex, px or ms values in components.
- Keep one primary action per view and at most three type sizes per view.
- Pair every status color with an icon or label; underline links in running text.

**Don't**
- Don't shrink hit areas in dense layouts; shrink visuals and extend the hit area.
- Don't use motion as the only signal, or flash more than three times a second.
- Don't copy another brand's identity from a reference (colors, typeface, logo, imagery, copy).

**Current validation findings:** warn: light: color.bg.action.primary (#5fcf6e) on color.surface.base (#fcfdfe) is 1.94:1, needs 3.0:1, warn: light: color.bg.action.primary (#5fcf6e) on color.surface.raised (#ffffff) is 1.97:1, needs 3.0:1

**Waivers:** none.

</details>

## Standards

**Follow these rules when you build.** They come from sources marked non-negotiable, their values are locked, and `engine.py review` checks the ones it can.

House standards v4: 291 rules followed in this project.

**Don't break these:**
- No motion on 100+ times-a-day actions (STD-when-to-animate-05).
- Never use ease-in on UI (STD-easing-duration-03).
- Never enter from scale(0) (STD-enter-exit-origin-01).
- Never use transition: all (STD-performance-properties-06).
- Never disable pinch zoom (STD-accessibility-motion-27).

<details><summary>Every theme (227 rules)</summary>

- **When to animate** (14 rules, 12 must): Decide whether to animate first; Name a valid purpose for every animation; Scale motion down as frequency rises; No motion on 100+ times-a-day actions; Never animate keyboard-initiated actions; Remove or shrink motion seen tens-of-times daily; Spend delight only on rare moments; Explanatory motion only in marketing, onboarding; and 4 more must rules. Locks `motion.duration.instant` 0ms.
- **Easing and duration** (12 rules, 10 must): Pick easing by the motion's job; Strong custom curves, not built-in keywords; Never use ease-in on UI; UI motion under 300ms unless justified; Duration budget for each element; Nothing over 1s unless illustrative; Duration grows with size and distance; Slow deliberate phase, snappy response; and 2 more must rules. Locks `motion.easing.linear` [0, 0, 1, 1], `motion.easing.hover` [0.25, 0.1, 0.25, 1], `motion.easing.enter` [0.23, 1, 0.32, 1], `motion.easing.exit` [0.23, 1, 0.32, 1], `motion.easing.standard` [0.77, 0, 0.175, 1], `motion.duration.micro` 100ms, `motion.duration.short` 150ms, `motion.duration.medium` 200ms, `motion.duration.long` 250ms.
- **Enter, exit and origin** (29 rules, 16 must): Never enter from scale(0); No opacity-only fade entrances; Scale popovers from their trigger; Keep modals scaling from center; Exit the way it entered; Dropdowns and popovers: scale 0.95, 200ms; Tooltips: scale 0.97, 125ms; Delay only the first tooltip; and 8 more must rules. Locks `motion.scale.enter` 0.95, `motion.duration.medium` 200ms (STD-easing-duration-07 wins), `motion.duration.tooltip` 125ms, `motion.scale.enter-tooltip` 0.97, `motion.duration.instant` 0ms (STD-when-to-animate-05 wins), `motion.duration.modal` 250ms, `motion.scale.enter-modal` 0.96, `motion.duration.stagger` 50ms, `motion.duration.reveal` 600ms, `motion.blur.crossfade` 2px, `motion.easing.drawer` [0.32, 0.72, 0, 1], `motion.duration.drawer` 500ms.
- **Springs and gestures** (32 rules, 21 must): Springs for anything a finger drives; Default springs are critically damped; Bounce only after a momentum gesture; Keep bounce subtle, 0.1 to 0.3; Sheet and drawer spring values; Every animation can be grabbed mid-flight; Never block input during transitions; Start from the on-screen value; and 13 more must rules. Locks `motion.spring.spatial.dampingRatio` 1.0, `motion.spring.spatial.stiffness` 246.7, `motion.spring.momentum` a composite value (see standards.md), `motion.spring.sheet` a composite value (see standards.md), `motion.spring.rotation` a composite value (see standards.md), `motion.gesture.deceleration-rate` 0.998, `motion.gesture.dismiss-velocity` 0.11, `motion.gesture.rubberband-constant` 0.55, `motion.gesture.direction-threshold` 10px.
- **Performance and properties** (14 rules, 9 must): Animate only transform and opacity; Only clip-path, accordion height, childless width; Never use transition: all; Motion: animate the full transform string; Keep per-frame values out of React state; Set drag transforms on the moving element; Pick the cheapest animation tool; Treat dropped frames as a regression; and 1 more must rule.
- **Accessibility in motion** (18 rules, 11 must): Ship reduced motion with the animation; Reduced motion is gentler, never zero; Check reduced motion by toggling it; Reduced transparency makes surfaces solid; Increased contrast gets solid, bordered surfaces; Scale layout with the user's text size; Gate hover motion to fine pointers; Build overlays on accessible primitives; and 3 more must rules. Locks `raw.reducedMotion` "replace", `motion.duration.reduced-fade` 200ms, `reduced:motion.transition.enter` a composite value (see standards.md), `reduced:motion.transition.exit` a composite value (see standards.md), `reduced:motion.transition.expand` a composite value (see standards.md).
- **Toasts, drawers and overlays** (23 rules, 7 must): Fill gaps between toasts for hover; Pause toast timers in a hidden tab; Toast classNames need !important; Copy Sonner styles into shadow DOM; Drawer curve: cubic-bezier(0.32, 0.72, 0, 1); Feedback on press, commit on release; Hold to confirm: slow press, fast release. Locks `motion.duration.toast` 400ms, `motion.easing.toast` [0.25, 0.1, 0.25, 1], `component.toast.gap` 14px, `component.toast.visible` 3, `component.toast.duration` 4000ms, `component.toast.offset.desktop` 32px, `component.toast.offset.mobile` 16px, `motion.easing.drawer` [0.32, 0.72, 0, 1] (STD-enter-exit-origin-41 wins), `motion.gesture.drag-lock-after-scroll` 100ms.
- **Mobile and touch** (20 rules, 16 must): Detect touch by capability, never by device; Remove the tap highlight flash globally; Press feedback scales to 0.97, briefly; Remove the tap delay on tappables; Touch targets 44pt on iOS, 48dp on Android; Size full-height layouts with dvh and svh; Form inputs at 16px or larger; Contain overscroll in app layouts; and 8 more must rules. Locks `motion.scale.press` 0.97, `motion.duration.micro` 100ms (STD-easing-duration-07 wins), `size.target.touch` 44px, `size.target.min` 44px, `font.size.input` 16px.
- **Visual details** (51 rules, 21 must): Cap body text near 65ch; Tabular digits for changing numbers; Use the ellipsis character; Loosen tracking on uppercase labels; Make letter-spacing specific to size; Reserve underlines for links; Metric-matched fallback font stack; Never stack light translucent surfaces; and 13 more must rules.
- **Process, review and taste** (14 rules, 8 must): Motion that merely runs is not done; Self-check motion before finishing; Report review findings as one table; Close every motion review with Block or Approve; Make the motion call yourself; Keep the motion note to a few lines; Say when feel can't be judged from code; Judge feel on real hardware.

</details>

Every rule, with its values, reasons and sources: `opendesigner/standards.md` (it also has the 64 rules this file leaves out). Read the themes you touch before you build.

## Motion

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-motion-01.
<!-- od:zoom area=motion level=0 -->

**Most movements take 200ms.** They turn into fades when a person asks for less motion.
Designers: motion system: duration, easing, springs · Code: `motion.duration.*, motion.easing.*`

<details><summary>More</summary>

**Intent.** Energy 55/100: the duration multiplier x1.02 scales the extra step (standards fix medium, long). Standard easing `cubic-bezier(0.77, 0, 0.175, 1)`, enter `cubic-bezier(0.23, 1, 0.32, 1)`, exit `cubic-bezier(0.23, 1, 0.32, 1)`. Spatial spring: damping 1.0, stiffness 246.7 (Apple duration 0.4s, bounce 0.0; web `linear()` sample over 590ms).

| Token | Value | Note |
|---|---|---|
| `motion.duration.instant` | 0ms | set by house standard STD-when-to-animate-05 |
| `motion.duration.micro` | 100ms | set by house standard STD-easing-duration-07 |
| `motion.duration.short` | 150ms | set by house standard STD-easing-duration-07 |
| `motion.duration.medium` | 200ms | set by house standard STD-easing-duration-07 |
| `motion.duration.long` | 250ms | set by house standard STD-easing-duration-07 |
| `motion.duration.extra` | 710ms | illustrative and marketing motion only, never UI (STD-easing-duration-06) |
| `motion.duration.medium-exit` | 160ms | derived: about 0.8 x medium |
| `motion.duration.long-exit` | 200ms | derived: about 0.8 x long |
| `motion.duration.drawer` | 500ms | set by house standard STD-enter-exit-origin-41 |
| `motion.duration.modal` | 250ms | set by house standard STD-enter-exit-origin-11 |
| `motion.duration.reduced-fade` | 200ms | set by house standard STD-accessibility-motion-02 |
| `motion.duration.reveal` | 600ms | set by house standard STD-enter-exit-origin-26 |
| `motion.duration.stagger` | 50ms | set by house standard STD-enter-exit-origin-13 |
| `motion.duration.toast` | 400ms | set by house standard STD-components-toasts-drawers-07 |
| `motion.duration.tooltip` | 125ms | set by house standard STD-enter-exit-origin-09 |

- Exits are derived, not set on their own: each is about 20% shorter than the entrance in force (medium-exit 160ms from medium 200ms, long-exit 200ms from long 250ms; STD-easing-duration-11). Change an entrance and its exit follows. Linear easing only for spinners and progress.
- Reduced motion (`prefers-reduced-motion` or `data-motion="reduced"`): travel becomes opacity, feedback stays, `motion.transition.move` is 0ms.
- Springs are stored as damping and stiffness in `$extensions.opendesigner.spring` because DTCG 2025.10 has no spring type.
- Haptics: medium intensity where the platform has them; sound: hook H-sound is pending.

Decisions: D-0009 (34 values, standards).

</details>

## Modes and Themes

> Zoom: sketch (0 of 3), 1 decision in this area. To zoom in, answer Q-theme-03.
<!-- od:zoom area=modes level=0 -->

**The system has light and dark color modes, three spacing densities and a reduced-motion mode.**
Designers: modes: light, dark, density, motion · Code: `resolver modifiers, Figma collection modes`

<details><summary>More</summary>

**Your answers here** (each one either sets tokens or is a rule for this section):
- Should the app have light and dark modes that follow the device, or just one mode? **Light and dark, matching the device setting** (assumed; tokens; Q-theme-01)

**Intent.** Modes live in the DTCG resolver (`opendesigner/tokens/opendesigner.resolver.json`); modifiers are orthogonal, so no two set the same token.

| Modifier | Contexts | Default |
|---|---|---|
| theme | light, dark | light |
| density | spacious, comfortable, compact | comfortable |
| motion | standard, reduced | standard |

- Web: `prefers-color-scheme` with a `[data-theme]` override; `[data-density]`; `prefers-reduced-motion` with `[data-motion]` (see `opendesigner/build/css/tokens.css`).
- High-contrast themes are not generated; set `raw.contrastTarget` to AAA for 7:1 text everywhere.
- A second brand or a client re-skin may override the color primitives and `color.bg.brand`; semantic names never change.

Decisions: D-0006 (answers.Q-theme-01).

</details>

## Iconography and Imagery

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-icon-01, Q-icon-02, Q-icon-06.
<!-- od:zoom area=iconography level=0 -->

**Icons are 20px by default.** The logo, photos and custom icons come from a person, not from the engine.
Designers: icon system: style, sizes, color · Code: `size.icon.*, icon.stroke.*, color.icon.*`

<details><summary>More</summary>

**Intent.** Default icon size 20px; sizes 16/20/24 pair with 14/16/20px text. Stroke follows the label weight: 1.5px at 16, 2.0px at 20, 2.5px at 24. Rest style: outline.

| Asset | Hook | Status |
|---|---|---|
| Logo and marks | H-logo | pending |
| Favicon set | H-favicon | pending |
| App icon | H-appicon | pending |
| Custom icons | H-icons | pending |
| Illustration | H-illus | pending |
| Photography | H-photo | pending |
| Motifs and brand shapes | H-motif | pending |

- Placeholders are labelled as placeholders. A generated stand-in is never shown as a finished brand asset.
- Icon-only buttons need an accessible name and a hit area of `size.target.min`.

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## Content and Voice

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-voice-01, Q-voice-02, Q-voice-03.
<!-- od:zoom area=content level=0 -->

**Words stay plain and short, and every button says what it does.**
Designers: content: voice, tone, microcopy · Code: `DESIGN.md Content and Voice section`

<details><summary>More</summary>

**Intent.** Draft: voice guidance level **short** (Brand presence 50); **sentence-case**; contractions **encouraged** (Warmth 80). Voice guide hook H-voice: pending.

- Buttons say what they do in two to four words; errors say what happened and how to fix it.
- Reading level: plain language; avoid jargon in UI copy.
- Word list: not recorded yet (Q-voice-03).

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## Accessibility

> Zoom: sketch (0 of 3), 0 decisions in this area. To zoom in, answer Q-aud-02, Q-aud-03, Q-aud-04.
<!-- od:zoom area=accessibility level=0 -->

**Text passes WCAG 2.2 AA contrast in every mode, and focus is always visible.**
Designers: WCAG 2.2 AA target · Code: `raw.contrastTarget`

<details><summary>More</summary>

**Your answers here** (each one either sets tokens or is a rule for this section):
- Who uses it, and how often: all day for work, often, or now and then on the go? **Often, in an everyday app** (chosen; tokens; Q-aud-01)

**Intent.** Standard: WCAG 2.2 AA. APCA is reported as advice only.

**The system guarantees:**
- text and edge contrast in every mode
- visible focus (2px ring, 2px offset, 3:1)
- minimum target sizes for each kind of input
- a reduced-motion mode
- rem-based type, so text scales to 200%
- text on colored fills chosen automatically

**Product teams own:** accessible names and labels, alt text, reading and focus order, and never using color alone. Also: clear error messages, dialogs that can be closed, no pre-checked consent boxes, and captions for media.

**Test on each platform:**
- keyboard only
- a screen reader (VoiceOver, NVDA or TalkBack)
- 200% zoom
- forced colors (Windows High Contrast)
- reduced motion
- touch on a phone, for every surface that allows touch

Decisions: none recorded yet; values are defaults from references/levers.json.

</details>

## Platforms and Devices

> Zoom: sketch (0 of 3), 1 decision in this area. To zoom in, answer Q-plat-05, Q-plat-10, Q-plat-08.
<!-- od:zoom area=platforms level=0 -->

**It runs on desktop, web, and each platform keeps its own navigation.**
Designers: target platforms · Code: `raw.platforms; size.target.min`

<details><summary>More</summary>

**Your answers here** (each one either sets tokens or is a rule for this section):
- Where will it run first: web, iOS, Android or desktop? **Desktop app (macOS, Windows, or built with web tech), Web** (chosen; tokens; Q-plat-01)

**Intent.** Platforms: desktop, web. Inputs: pointer, touch. Brand presence 50: each platform keeps its own shapes and materials: no; each platform keeps its own body text size: no.

- Navigation, back, sheets and pickers stay native on every platform (DC-L10-02).
- Code files in `opendesigner/build/`: css, tailwind (each uses the same token names, and Figma code syntax matches them). Not written for these platforms: swift, compose (`engine.py export --format <name>` writes one).
- Fonts by platform: text: Atkinson Hyperlegible Next Variable: licensed for web, apps, self-hosting.

Decisions: D-0020 (answers.Q-plat-01).

</details>

## Decisions

**22 decisions so far.** Each one says who set it and why.
Designers: decision history with reasons · Code: `opendesigner/decisions.md`

<details><summary>More</summary>

Your own decisions first (the newest value per path), then your changes to standards, then one row per standards batch. Full log: `opendesigner/decisions.md`.

| Id | Path | Value | Set by | Reason |
|---|---|---|---|---|
| D-0002 | `context.product` | "Aplicación de escritorio para Windows, de código aberto e en galego, para converter, comprimir, editar e inspeccionar imaxes, vídeo, audio, PDF e modelos 3D." | assumed | read from the first paragraph of README.md; say if that's wrong |
| D-0004 | `raw.brandColor` | "#5fcf6e" | assumed | read from the code: the colour the CSS uses most on button, buttonBorder, buttonText, control, focus, hover, token (106 uses, high confidence); say if... |
| D-0006 | `answers.Q-theme-01` | "system-light-dark" | assumed | read from prefers-color-scheme: dark in dist-web\index.html; say if that's wrong |
| D-0007 | `answers.Q-scope-07` | "near-launch" | assumed | read from the repo: code and deploy settings (.github/workflows/build-windows.yml, .github/workflows/release.yml, .vercel), but no released version yet; say... |
| D-0008 | `raw.stack` | ["svelte"] | assumed | detected from package.json (svelte); say if the stack is different |
| D-0011 | `context.audience` | "people who use it often, as an everyday app" | assumed | sketch: plain words for Q-aud-01 |
| D-0015 | `raw.neutralTint` | "cool" | assumed | A app e a web usan grafito frío (#131417) con verde; manter esa identidade |
| D-0017 | `raw.textFace` | "Atkinson Hyperlegible Next Variable" | assumed | Fonte xa usada e autoalojada na app e na web (OFL) |
| D-0018 | `raw.fontLicence` | {"web": true, "app": true, "selfHost": true} | assumed | Atkinson Hyperlegible Next: licenza OFL, permite web, app e autoaloxamento |
| D-0019 | `answers.Q-aud-01` | "regular" | chosen | sketch: Level 0 answer |
| D-0020 | `answers.Q-plat-01` | ["desktop", "web"] | chosen | sketch: Level 0 answer |
| D-0021 | `macros` | ["friendly", "playful"] | delegated | sketch: picked for them (feel words: friendly, playful) |
| D-0022 | `taste.feelWords` | ["friendly", "playful"] | delegated | sketch: feel words picked for them |
| D-0009 | standards | House standards v4: 291 applied | standard (locked) | OpenDesigner's house standards (sources marked non-negotiable) are applied without asking and locked. Change one only when the person asks: engine.py... |

</details>

## Open Items

**These parts are still defaults, guesses, or waiting for a person.**

- Still defaults (zoom 0, sketch): Colors, Typography, Layout, Elevation & Depth, Shapes, Components, Motion, Modes and Themes, Iconography and Imagery, Content and Voice, Accessibility, Platforms and Devices
- Assets not final: Logo, wordmark, symbol, lockups (H-logo): pending; App icon (H-appicon): pending; Favicon set (H-favicon): pending; Custom icons, pictograms, spot icons (H-icons): pending; Illustration, characters, empty-state art (H-illus): pending; Photography and art direction (H-photo): pending; Brand typeface files (H-type): pending; Fixed brand colors (H-color): pending; Motion signature and rich media (H-motion): pending; Graphic devices, patterns, textures, brand gradients, signature shape (H-motif): pending; UI sounds and sonic logo (H-sound): pending; Custom haptic patterns (H-haptic): pending; Voice and tone guide (H-voice): pending; Brand guidelines PDF (H-brandbook): pending
- Answers taken on assumption or delegated: context.product (assumed), raw.brandColor (assumed), raw.textFace (assumed), answers.Q-theme-01 (assumed), answers.Q-scope-07 (assumed), raw.stack (assumed), context.audience (assumed), macros (delegated), taste.feelWords (delegated), raw.neutralTint (assumed), raw.fontLicence (assumed)
- Waivers: none.
- Licence risks: none found.
- Validation: 0 errors, 2 warnings.

## For Agents

**AI agents: read this file and the tokens before you change any screen.**

1. Before any visual work, read this file, PRODUCT.md and `opendesigner/tokens/` (start at `opendesigner/tokens/opendesigner.resolver.json`).
2. Use tokens, never raw values: CSS `var(--ds-...)`, Tailwind classes from `opendesigner/build/tailwind/theme.css`.
3. Do not change a locked decision without asking the owner. Keep new screens inside the taste card at the top of the Overview, and never add anything on its Never list without asking.
4. To change the system: `engine.py set <path> <value> --why "..."`, then `engine.py generate`, `engine.py validate`, `engine.py design-md`.
5. To extend: add a decision (never hand-edit tokens or this file). Keep semantic names the same across modes. Deprecate instead of deleting.
6. Every build: open a round (`engine.py ledger start "<what>" --items ...`), build from the tokens, run `engine.py review`, check the rendered page with opendesigner-review and show screenshots at phone and desktop width, mark each ledger item done or waived (with the owner's words), then re-read this file.
7. Status colours (success, warning, danger, info) carry one meaning each and never alone: follow Status and Feedback above (`opendesigner/build/status.json` holds the same map). After every action, show what happened as it says.
8. Follow the Standards section. Change a standard only when the person explicitly asks: `engine.py standard override <id> --why "<their words>"`.

**Tailwind v4 classes** (one import: `@import "tailwindcss"; @import "<path>/opendesigner/build/tailwind/theme.css";`; Tailwind's default colors, text sizes, radii and shadows are switched off, so only these exist). They need a Tailwind build that imports that file. In plain HTML with no build step, use the CSS name beside each class from `opendesigner/build/css/tokens.css`; never load Tailwind from a CDN script. Each row is one token under its two names:

| Area | Tailwind class | Plain CSS (no build step) |
|---|---|---|
| Color | `bg-action-primary` | `var(--ds-color-bg-action-primary)` |
| Color | `hover:bg-action-primary-hover` | `var(--ds-color-bg-action-primary-hover)` |
| Color | `active:bg-action-primary-pressed` | `var(--ds-color-bg-action-primary-pressed)` |
| Color | `text-on-action` | `var(--ds-color-text-on-action)` |
| Color | `bg-surface-base` | `var(--ds-color-surface-base)` |
| Color | `bg-surface-raised` | `var(--ds-color-surface-raised)` |
| Color | `text-fg-primary` | `var(--ds-color-text-primary)` |
| Color | `text-fg-secondary` | `var(--ds-color-text-secondary)` |
| Color | `text-fg-link` | `var(--ds-color-text-link)` |
| Color | `border-line-subtle` | `var(--ds-color-border-subtle)` |
| Color | `border-line-input` | `var(--ds-color-border-input)` |
| Color | `outline-line-focus` | `var(--ds-color-border-focus)` |
| Color | `bg-danger-bold` | `var(--ds-color-bg-danger-bold)` |
| Layout | `p-inset-md` | `var(--ds-space-inset-md)` |
| Layout | `gap-stack-md` | `var(--ds-space-stack-md)` |
| Layout | `gap-inline-sm` | `var(--ds-space-inline-sm)` |
| Layout | `h-control-md` | `var(--ds-size-control-md)` |
| Layout | `size-icon-md` | `var(--ds-size-icon-md)` |
| Layout | `min-h-target-min` | `var(--ds-size-target-min)` |
| Shape | `rounded-control` | `var(--ds-radius-control)` |
| Shape | `rounded-container` | `var(--ds-radius-container)` |
| Shape | `rounded-overlay` | `var(--ds-radius-overlay)` |
| Shape | `rounded-full` | `var(--ds-radius-full)` |
| Elevation | `shadow-raised` | `var(--ds-elevation-raised)` |
| Elevation | `shadow-floating` | `var(--ds-elevation-floating)` |
| Elevation | `shadow-overlay` | `var(--ds-elevation-overlay)` |
| Type | `text-body-md` | `.ds-text-body-md` (a text style class) |
| Type | `text-label-lg` | `.ds-text-label-lg` (a text style class) |
| Type | `text-title-md` | `.ds-text-title-md` (a text style class) |
| Type | `text-headline-md` | `.ds-text-headline-md` (a text style class) |
| Motion | `transition-feedback` | `var(--ds-motion-transition-feedback-duration)` and `var(--ds-motion-transition-feedback-easing)` |
| Motion | `ease-enter` | `var(--ds-motion-easing-enter)` |
| Motion | `ease-exit` | `var(--ds-motion-easing-exit)` |
| Motion | `ease-hover` | `var(--ds-motion-easing-hover)` |
