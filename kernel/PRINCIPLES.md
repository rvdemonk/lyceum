---
Document Context:
  Created: 2026-05-01
  Updated: 2026-05-06
  Source: Iterative design conversations while building demo.html — distilled as principles emerge
  Status: REFERENCE
  Purpose: Codify the design principles that govern monodoc documents so future features inherit them automatically
---

# Monodoc Design Principles

Monodoc is the typographic kernel — the system that renders a writeup into a designed HTML document, and the renderer that *lyceum* (the personal-wiki tool) will depend on. See `README.md`, `LYCEUM.md`. The principles below govern *how* a document presents — typography, chrome, palette, theme, interaction, and how all of these survive the move from a desktop to a phone. They are taste hardened by research, not rules invented from scratch. Each principle answers a question we ran into during the demo build, and each was articulated only after we'd seen the alternative fail.

These principles are a living document, not a crystallised constitution. They are evolved by mutual feedback between Lewis and Claude — proposed, challenged, refined, amended as the work teaches us more. A principle may carry explicit exceptions, and an exception noted today does not foreclose others tomorrow. Each earns its place by use, and keeps it only while it still describes what the work actually wants.

The document is the product. Everything else is infrastructure. Infrastructure must be invisible at rest.

---

## 1. Two surfaces: document and instruments

A primer has two surfaces and they answer to different masters.

The **document** is text — prose, headings, sidenotes, quotations. It belongs to the reader. Reader preferences (font, size, sidenote toggle) only touch this surface.

The **instruments** are everything else — the TOC, the config panel, the future progress indicator, the future register switch. They are tools the reader plays the document on. They have their own fixed scale and never bend with reader preferences.

In practice this means: the document uses `rem` units that cascade from `--base`. The instruments declare their own `font: 13px/1.4 var(--sans)` at their root and use only `px` internally. They are insulated.

The insulation is about *scale*, not *colour*. A theme — warm dark, cool dark, parchment — recolours both surfaces at once, document and instruments together, because chrome that stayed warm-dark while the document turned to parchment would read as a foreign object pasted on top. Scale must not cross from document to instruments; colour must. They are different axes, and the insulation applies to only one of them.

> *Why.* When chrome scales with `--base`, the controls visibly shudder every time the user adjusts size — and worse, the controls become a moving target while the user is using them. A control that grows when you select "L" is fighting the user.

## 2. Peripheral vision design

Chrome is collapsed at rest, materialised when reached for. Faint dots mark a TOC's existence; an "Aa" handle marks the config corner; both expand only when the user moves toward them.

The mechanism is opacity + transform, never `max-width` clipping. Layout-driven reveals lock the parent box's width to the collapsed state, which causes children to compute against the wrong constraint and clip unpredictably. Visual reveals (opacity, transform, clip-path) leave the layout settled and animate only what the eye sees.

The pattern assumes a hovering pointer. Touch has no hover — there is no "reaching toward" for the chrome to detect — so on a phone the peripheral pattern cannot hold. Touch gets a single permanent, minimal bar instead: the document title and one settings control, always visible. This is a real concession, not an oversight, and naming it here is what stops a later revision from trying to force the hover-reveal onto a phone, where it fails silently — the chrome simply becomes unreachable, with no error to notice.

> *Why.* The reader's central vision is for prose. Anything in the periphery should *acknowledge* its existence (so the reader knows tools are there) without *competing* for the attention that prose needs. Faint markers acknowledge; full chrome competes.

## 3. State lives in tone, not geometry

Hover and active states change colour and weight. They do not change borders, backgrounds, shadows, or shape. The state ladder is `--text-faint` → `--text-muted` → `--text` — three rungs of luminance against a fixed warm bg.

Geometric affordances (boxes, borders, glows) read as UI in the bad sense — chrome that wants to be seen. Tonal affordances read as text becoming more or less present, which is the same vocabulary the prose uses.

> *Why.* Mixed registers fight each other. If the prose communicates in tone (italic, small-caps, weight) and the chrome communicates in geometry (boxes, borders), the document feels like text glued to a UI. If both communicate in tone, the chrome reads as a quiet typographic continuation of the prose.

## 4. The cursor sits still

No `cursor: pointer`. Hover feedback lives in the UI itself — colour change, opacity rise, transform — never in the pointer.

Selection still gets the I-beam, because selecting text is a real action the cursor should signal. Everything else stays default.

> *Why.* The default↔pointer flicker that fires when the mouse moves across an interactive element is high-frequency visual noise that adds nothing the UI's own response doesn't already convey. Disabling it is a small calm-down with no information loss.

## 5. Type whispers, hit areas don't

Visual weight and physical weight are independent axes. A control can be quiet to look at *and* generous to click. Resolve the two separately.

In practice: chrome typography stays small (13px) so it sits peripheral; chrome buttons get padding (~5×9px) so they're a comfortable mouse target. The quiet font stays quiet; the hit zone grows to ~24px tall.

> *Why.* Conflating visual and physical leads to either-or compromises: either the chrome is loud enough to click confidently or it's quiet enough to ignore. Decoupling lets it be both.

## 6. Architecture must be diagnosable

When something breaks, the cause should be locally readable. Avoid relying on intrinsic sizing chains four layers deep. Avoid CSS that requires knowing how flexbox resolves `flex-basis: auto` against block-child max-content of a `display: block` text node. Prefer mechanisms where the failure mode is obvious from the rule itself.

Concrete instance: animating `max-width` from 0 to a large value to "expand" a container couples animation to layout, breaks min-content propagation, and produces clipping bugs that resist debugging. Animating `opacity` and `transform` with the box always at natural width produces no such bugs because layout never participates.

> *Why.* Future maintenance — by Claude, by future-Lewis, by anyone — depends on the rules being legible in isolation. A clever solution that requires reasoning across four cascading layers is a debt that compounds.

## 7. Restraint is a contract

Every primer makes the same promise to the reader: nothing here is decorative. Every visible element earns its presence by serving the prose. If a flourish doesn't survive the question "what does this do for the reader", it goes.

This is the source of the monotone palette, the borderless controls, the absence of icons, the absent gradients, the single-typeface-pair, the unmarked active state on dropdowns. Each is a refusal of decoration.

> *Why.* The reader extends trust in proportion to how little the design appears to be performing for them. Every gratuitous element subtracts trust. Every austere choice that proves correct on second glance adds it.

## 8. A reference is an appendix reached by marginal effort

A footnote, sidenote, or citation is a *colocated appendix* — material that belongs with the document but sits outside its direct scope. What makes it a separate thing is that consuming it costs a small, deliberate effort: a drop of the eye to the foot of the page, a flip to the back of the book, a tap on a number. That effort is not friction to be engineered away. It is the boundary that marks the note as outside the reading line.

This is why the on-demand reveal — tap a reference, the note opens in flow beneath its paragraph; tap again, it closes — is right, and the old "sidenotes off, dump every note inline at full width" was wrong. The inline dump removed the marginal effort, and with it the distinction: the note stopped being an appendix and became an interruption, harder to skip than a margin note and breaking the reading line completely.

The reveal must also stay typographic, not tacky. One gesture; the note arriving in the document's own voice — hairline rule, muted tone, the reading measure preserved — never a floating bordered card with a shadow. A popup that looks like OS chrome announces "software"; a passage that quietly makes room announces "document".

> *Why.* If reference material costs nothing to consume, it stops being reference material — it becomes body text the reader cannot opt out of. The marginal effort is what lets the scanner skip and the committed reader descend, each without fighting the layout. Remove the effort and the two reading modes collapse into one.

## 9. Constraints that manufacture margin are conditional on having margin

Several of monodoc's layout rules exist to *create empty space*: the text measure held to 55% of the body, the 12.5% left gutter, the absolutely-positioned sidenote column. They are not universal goods. They are desktop affordances — they spend horizontal room to buy a calm reading column and a place for margin notes to live.

A phone has no horizontal room to spend. Applied unconditionally there, the same rules clamped body text to roughly 48% of the viewport — a column too narrow to read, manufacturing margin that had nowhere to go. The fix was not to retune the percentages but to recognise the rules as conditional: below the breakpoint the measure lifts, the gutter collapses, the text fills the column, and the sidenote stops being a margin object.

> *Why.* A constraint codified on a desktop carries an invisible premise — *there is width here to give away*. When the premise fails, the constraint inverts from affordance to defect. Any layout rule whose purpose is to produce whitespace must be guarded by the condition that justified it, or it will strangle the content in the context where that premise does not hold.

## 10. Italics belong to prose

Italics are a device of *prose*. They carry emphasis, mark a title, set a word slightly apart — and they signify only because they depart from an upright norm. Spend the slant as the *default* dress of a non-prose element — a catalogue heading, a metadata field, a table row — and it stops meaning "emphasis" and becomes plain decoration. The decoration then dilutes the device for the prose that genuinely needs it.

The lyceum index made this concrete. Collection headings, the count line, and the description column all rendered italic, and the table read as slanted noise. Set upright, the same table read as structure. A catalogue is not prose; it earns no italics.

**A noted exception.** monodoc's writeup `<h2>` section headings are italic, and that stays. The italic subhead is a settled convention of book typography — the tradition monodoc draws on — and a heading inside a reading document sits close enough to prose to keep the slant. The principle governs *non-prose surfaces*; a section heading within a writeup is not one. This is the first exception recorded against this principle; recorded openly, and not assumed to be the last.

> *Why.* A typographic device is only as strong as its scarcity. A slant used everywhere emphasises nothing. Reserving italics for prose keeps the device sharp where the writing actually needs it.

---

## Open questions

These aren't principles yet. They're places where we've seen the tension but haven't committed.

- **Register modes.** Essayistic (current default), analytical (paragraph-spaced, callouts), wiki/hypertext (cross-links, popovers). Claude reads authorial intent and selects the register. The principles above are register-agnostic; the visual primitives that materialise them differ per register.

- **Click-to-pin on desktop.** Desktop chrome is still hover-only — the panel closes when the cursor leaves, which is friction when comparing theme or typeface options. Touch now resolves this its own way (the topbar's settings button is an explicit click-toggle), so the click-toggle mechanism already exists in the code; the open question is only whether desktop should adopt it too, or whether hover-only is right for a pointer.

- **Sidenote contrast.** Sidenote text sits around Lc 55–60 — below the Lc 75 floor for body text (see `RESEARCH.md`). Appropriate for receding marginalia, arguably too faint for a note the reader has deliberately tapped open. Whether the on-demand reveal should lift the note's contrast while it is open is unresolved.

- **Mermaid theme-reactivity.** Diagrams render with a fixed dark mermaid theme; they do not recolour when the reader switches between warm dark, cool dark, and parchment. This breaks Principle 1's "colour crosses to both surfaces" for the diagram surface specifically. Resolving it means re-running mermaid on theme change with per-theme `themeVariables`. Deferred — see `BACKLOG.md`.
