---
Document Context:
  Created: 2026-05-01
  Source: Sources and findings drawn on while developing the monodoc design system
  Status: REFERENCE
  Purpose: Capture the evidence base that informs (but does not dictate) the design principles. Findings live here; commitments live in PRINCIPLES.md.
---

# Research

A living record of the typography, perception, and interaction research underlying monodoc's design choices. Each section presents findings as findings — neutral, with attribution and a candid note on source confidence — and stays out of the prescriptive business. PRINCIPLES.md decides what we *do* with this evidence.

When something here is challenged, return to this doc to test whether the principle still holds, or whether new evidence has shifted the ground.

---

## Typography for screen reading

**Serif vs sans, at modern resolutions, is not a legibility question.** Multiple studies across two decades find no statistically significant comprehension or speed difference between well-designed serif and sans-serif at adequate sizes on screen. Bernard et al. (2003) on Times vs Arial; Subbaram's RIT thesis (2004); summary in Beier (2012). The historical "serif aids horizontal reading by guiding the eye" claim does not survive controlled testing at screen sizes ≥14px on 100+ DPI displays.

**Gwern's own A/B test** on gwern.net cycled four typefaces (Baskerville, Georgia, Trebuchet, Helvetica) across substantial traffic and reported no statistically significant engagement difference (the handoff cites 142,983 visits, p > 0.05; not independently re-verified). Gwern's general view, consistent with the literature, is that beyond a legibility floor the choice is *cultural* — serif signals seriousness, duration, expectation of staying; sans signals modernity, utility, transience.

**Implication for our work**: typeface choice is an act of *register-setting*, not legibility-optimisation. Both have to be well-designed for screen, but the choice between them communicates something about the tone of the document, not its readability.

**Source Serif 4 / Source Sans 3** (Adobe, Frank Grießhammer) are siblings: same design DNA, optimised for screen, available as variable fonts under SIL Open Font License. Pairing them within one document keeps the typographic register coherent across mode toggles.

## Line length and measure

**The classical "66 characters" figure** comes from Robert Bringhurst, *The Elements of Typographic Style* (1992): "Anything from 45 to 75 characters is widely regarded as a satisfactory length of line for a single-column page set in a serifed text face. The 66-character line (counting both letters and spaces) is widely regarded as ideal." Bringhurst presents this as accumulated craft knowledge from book typography, not as the result of a controlled study.

(The previous handoff attributes a "66 books" empirical validation to "Eric Lawson". I cannot verify this attribution from training and could not confirm a typographer or researcher of that name producing that result. Treating as unverified until a source is found. The Bringhurst figure stands on its own historical authority.)

**Butterick's *Practical Typography*** (practicaltypography.com) gives roughly congruent guidance: 45–90 characters per line, with 60–75 a comfortable centre. Butterick is more screen-aware than Bringhurst and explicitly addresses font-size on retina displays (recommending 15–25px depending on context).

**Implication for our work**: a 55%-of-body-width text column at 17px / Source Serif 4 lands the measure roughly in the 60–70 character band. This is the upper-classical range, slightly tighter than Butterick's centre, defensible.

## Contrast and dark mode

**APCA (Advanced Perceptual Contrast Algorithm)** — Andrew Somers, currently a candidate for WCAG 3 — is a perceptually-weighted successor to WCAG 2's relative-luminance contrast formula. It corrects WCAG 2's known failures on dark-mode designs (where the older formula systematically over-rates dark-on-dark and under-rates light-on-dark contrast).

APCA reports a single number, **Lc** (lightness contrast). Rough thresholds in the current draft:

- **Lc 90** — minimum for fluent body text at small sizes
- **Lc 75** — minimum for body text at normal sizes
- **Lc 60** — non-content (interface chrome, headlines)
- **Lc 45** — incidental content
- **Lc 30** — disabled / inactive, lower limit

Body text at #d4c9b6 on #1c1914 lands at roughly **Lc 78–80**, comfortably above the body floor. Sidenote text at #8a7e72 lands around **Lc 55–60**, *below* the body floor — appropriate for receding marginalia but not for sustained reading.

**Why the warm bg** (`#1c1914`) instead of Material Design's `#121212`: cool greys on LED displays read as "off monitor"; warm near-blacks read as "lit room". The choice is aesthetic-physiological, not legibility-driven — both work — but the warm tone reduces the "I am looking at a screen" feeling that competes with sustained attention.

**Halation** on OLED: pure white text on pure black causes chromatic-aberration artefacts ("smearing") at the letterform edges, especially for sub-pixel-sensitive readers. Pulling both ends of the contrast range slightly inward (off-white text, off-black bg) substantially reduces the effect. This is folk-knowledge widely cited in dark-mode design discussion (Material Design guidance, Apple HIG, Google's Android guidelines) but has limited peer-reviewed backing — treat as plausible heuristic.

## Sidenotes and structural references

**Tufte's sidenote system**, codified in his books and ported to the web by Dave Liepmann's *Tufte CSS* (edwardtufte.github.io/tufte-css), positions reference material in the right margin alongside the body, eliminating the eye-jump to footnotes. Numbered superscripts in the body, full text in the margin. Key advantage: the reader who scans never *has* to acknowledge the sidenote; the reader who pauses gets the depth without context-switching to a different region of the page.

**Marginnotes** (Tufte's term, adopted in Tufte CSS) — sidenotes without numbers, used for purely supplementary commentary that isn't being cited from the body. Same position, different epistemological role.

**Gwern's "popups"** (gwern.net) extend the sidenote idea: hover any link and a sidebar reveals the full referenced content (Wikipedia entry, paper abstract, prior gwern essay). This is essentially margin-notes-on-demand for hyperlinks, and forms the basis of what Gwern calls *semantic zoom* — a reader can move between scan-layer (headings + margin notes) and read-layer (prose) and reference-layer (sources) without losing position.

**Implication for our work**: the structural primitive is *layered reference*. Body prose is the primary layer; sidenotes/marginnotes are the secondary layer; (future) hover-popovers for links would be the tertiary layer. Each layer has its own typographic register and its own way of being addressed by the reader.

## Interaction patterns

**Peripheral vision design** is not, as far as I can find, a named design movement with a primary literature — it's a folk principle articulated across several practitioner sources. The core idea: chrome that lives outside the reader's focal region should *acknowledge its presence* (so the reader knows tools exist) without *competing for foveal attention* (so the reader's prose-reading isn't interrupted).

Practical instances: faint TOC dots that expand to labels on hover; collapsed config handle that reveals on hover; progress indicators that are visible but unobtrusive. Each of these renders the chrome at a brightness/size below the foveal-attention threshold while keeping it discoverable in peripheral vision.

**Semantic zoom**, as Gwern uses the term, refers specifically to the *reader's ability to navigate between layers of detail without losing structural position*. The TOC is the headings layer; sidenotes are the supplementary layer; popovers are the source layer; the body prose is the prose layer. A reader at any point can move "outward" (to TOC, to source) and back without re-finding where they were.

**iA Writer** — the minimal text editor by Information Architects (iawriter.com) — codifies a related principle in writing software: *focus mode*, where only the current paragraph (or sentence, in some modes) is rendered at full brightness, and the rest is dimmed. The chrome itself almost vanishes. Their typographic discipline (custom-designed monospaced families, severe restraint in UI) is a useful reference for what "infrastructure that disappears" looks like in practice.

## Reference designs surveyed

- **Tufte CSS** (edwardtufte.github.io/tufte-css) — the canonical web port of Tufte's print-typography design language. Sidenote system, et-book typeface, generous margins. Optimised for light mode; we depart on palette and on register flexibility.
- **gwern.net** — long-form essay site with extensive sidenote/popup system, structural depth, A/B tested typography. Reject the visual aesthetic (cluttered for our purposes), borrow heavily from the structural ideas.
- **Practical Typography** (practicaltypography.com) — Butterick's screen-aware typography manual. Specific recommendations on body size, measure, line-height, font choice. The most directly applicable single source for screen-oriented type decisions.
- **iA Writer** — design-discipline reference for what severe restraint looks like in practice.
- **Robin Sloan's writing tools** — particularly *Spring '83* and his various single-page essay designs. Useful for understanding how a single HTML file can hold both essayistic prose and embedded interactivity.

---

## Open threads

- Verify the "Eric Lawson / 66 books" attribution for the line-length figure, or strike it from the handoff.
- APCA Lc thresholds are draft-state in WCAG 3; periodically check whether the floors have shifted.
- Quantitative comprehension data on dark mode vs light mode is thinner than the popular discussion implies. Most "dark mode is better/worse" claims are not well-supported by controlled studies. A more careful read of the literature (Buchner & Baumgartner 2007 onward) would be useful before committing strong claims.
