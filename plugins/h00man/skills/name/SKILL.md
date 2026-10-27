---
name: name
description: Generate and screen brand names across descriptive to invented types with sound and distinctiveness checks. Use to name a company, product, or feature; quick gives 20 names, deep grills then 100-plus into an HTML shortlist. Not legal clearance.
---

# name

Name things the way Lexicon, Igor, A Hundred Monkeys, and Catchword do: **generate broad, then screen hard.** The breakthrough name is rarely in the first 20 — it's in the second hundred. Your job is to generate a wide, varied candidate set, run it through the screens below, and deliver the **whole pool plus a recommended shortlist** — always as an interactive HTML artifact the user can filter and re-curate — with the reasoning attached.

This skill **generates and screens**. It does not clear trademarks or register domains — it flags likely risks and tells the user what to check, but a serious venture still needs a real trademark search (tmsearch.uspto.gov, WIPO Global Brand Database) and an attorney. Say so; don't imply a name is "available."

For deep dives — case studies, the legal doctrine in full, China/global naming, the academic citations — load `reference.md` in this skill folder. Don't load it for routine naming; the essentials are below. `reference.md` holds background facts only: when it and this file differ on what to do, this file wins.

## Two modes — quick vs. deep

Same deliverable (the HTML artifact below); they differ only in how much you ask and how many candidates you generate.

- **`deep` (default).** Grill the user with **up to 10 questions**, generate **100+ candidates** across the taxonomy, screen down to a **20–30-name shortlist**. The brief is where quality is won, so this is the default whenever the mode is ambiguous.
- **`quick`.** Ask nothing (at most one blocker question), generate **~20 candidates** with light screening, hand back a **~20-name shortlist**.

Honor an explicit `quick` / `deep`. Otherwise infer from signal — terse "name my X" with a hurry-vibe → quick; a richer prompt or real venture → deep. When genuinely ambiguous, default to deep and say so in one line ("Running in deep mode — I'll ask a few questions first; say *quick* to skip"). The user can switch anytime.

## Step 1 — Get a brief

The quality of the names is set here, not in generation.

**Deep mode — grill (up to 10 questions).** Batch them (`AskUserQuestion` takes 4 at once, so 1–3 rounds). Ask only what the prompt leaves open, and stop once you know what it is, the vibe, the audience, the competitors, the markets, and the hard constraints. Spend questions on what shapes the name:

- **What is it?** One sentence on what it does and what makes it different.
- **Vibe.** Playful/serious, premium/accessible, warm/sharp — what feeling on first hearing?
- **Audience.** Who's it for, and what do they call this category today?
- **Competitors.** Name 3–5 — you'll name deliberately *away* from their pattern.
- **Architecture.** Standalone, or under a parent? Need sub-brands later (extendability)?
- **Forbidden territory.** Words/themes that must appear, must be avoided, or that competitors own. Founder vetoes?
- **Language.** English-only or cross-language? Cultural markets to respect (`reference.md` §6, §8)?
- **Constraints.** One market or global? Domain flexibility (.com required, or .ai/.io/.co fine)? TM classes that matter?
- **Reaction appetite.** Safe-and-clear, or open to a polarizing name? (Calibrates how far up the taxonomy to push.)

Reflect the brief back in a sentence before generating, so the user can correct course cheaply.

**Quick mode — skip the grill.** Generate from what you were given and **state your assumptions inline**. Ask at most one question, only if the request is unworkable without it (e.g. you don't know what the product is).

## Step 2 — Generate broad, across the taxonomy

Generate by mode: **deep → 100+ candidates**, **quick → ~20**. Either way, deliberately spread them across name *types* so the user sees real range — not many variations of one idea. Don't converge on the category-typical pattern; that's the failure mode of lazy naming (and of AI name generators). In deep mode, hit **every** type in the table below — if a type has no candidates in your output, you under-generated (the template's coverage bar shows a zero in red). In quick mode, cover a broad sample of types but don't force one of each. Use the bold label before any slash as the candidate's `type` (`Suggestive`, `Invented`, `Altered spelling`); the template checks against these exact labels. The types:

| Type | What it is | Examples |
|---|---|---|
| **Descriptive** | Says what it does | PayPal, The Home Depot |
| **Suggestive / evocative** | Hints at a quality; needs a small leap | Amazon, Slack, Patagonia |
| **Associative / metaphoric** | Borrows imagery from another domain | Nike, Starbucks, Jaguar |
| **Synonym-driven** | A near-synonym or adjacent word for the core concept, used directly | Stripe, Notion, Ramp, Vise |
| **Invented / coined** | Built letter-string, no prior meaning | Kodak, Xerox, Häagen-Dazs |
| **Abstract / arbitrary** | Real word, unrelated use | Apple, Camel, Shell |
| **Compound** | Two real words fused whole | Facebook, Snowflake, Salesforce |
| **Portmanteau / blend** | Two words *chopped* and merged | Pinterest, Instagram, Groupon, Verizon |
| **Acronym / initialism** | Reduced to initial letters (pronounced as word or spelled out) | IBM, KFC, NASA, IKEA |
| **Lexical / real-word** | A plain dictionary word | Oracle, Visa, Dove |
| **Foreign / borrowed** | A word from another language | Volvo (Latin "I roll"), Audi (Latin "listen"), Hulu (Mandarin "gourd") |
| **Palindrome** | Reads the same backwards | Sonos, Otto, Aviva, Civic |
| **Reduplicative / rhyming** | Repeated or rhyming syllables | Google, TikTok, Kit-Kat, Yoyo, Juju |
| **Mimetic / onomatopoeic** | The sound is part of the meaning | Twitter, Bing, Schweppes |
| **Altered spelling** | Familiar word respelled | Lyft, Flickr, Reddit |
| **Affixed** | Root + productive prefix/suffix | Calendly, Grammarly, Shopify |

Generation tactics that produce range: **morpheme tables** (prefixes `inter-/trans-/ultra-/neo-`, suffixes `-ify/-ly/-io/-ai/-ex/-on/-ium/-os`); **synonym/thesaurus walks** of the core concept; **foreign-language and Latin/Greek roots**; **metaphor mining** ("if this were an animal / a place / a sound / a myth, what would it be?"); **acronym hunts** (write the descriptive phrase, then mine its initials for a pronounceable string); **sound-pattern templates** (CVCV, alliteration, palindromes, reduplication); **etymology walks** from the root word outward.

**Default to suggestive, arbitrary, invented, portmanteau, or palindrome.** Avoid purely descriptive names unless the user insists — they're weak legally and forgettable, and successful ones eventually rebrand (KFC, RH, WW). Acronyms are usually weak as *primary* names: they carry no image, and letters that stand for a descriptive phrase stay legally weak. A pronounceable one (IKEA, NASA) is easier to say and remember, but that does not make it legally stronger. Generate a few; don't lead with them.

## Step 3 — Screen

**Deduplicate first.** Normalize each name (lowercase, strip spaces, hyphens, and punctuation) and drop exact repeats. Collapse spelling variants and homophones of one root (Lumen / Lumin / Lumena) to the best one. If the unique count falls under the mode's target, generate more before you screen.

Then run candidates through these screens. The first pass kills the obvious losers; give each survivor a `score` (Step 4).

**Sound symbolism (engineer the sound, not just the meaning).** Sounds carry associations that listeners pick up without trying. These are tendencies — roughly 60:40 to 70:30 in published tests — not rules. Use them to match the feel to the product; never reject a name on sound alone. Judge the spoken sounds, not the letters (Apple starts with a front vowel).
- **Front vowels** (i, ee) → small, light, fast, sharp, friendly, feminine. **Back vowels** (a, o, u) → large, heavy, powerful, serious.
- **Fricatives** (f, s, v, z) → soft, light, fast. **Stops/plosives** (b, d, g, p, t, k) → hard, crisp, energetic.
- Match the sound to the product: front-vowel/fricative names suit light/elegant/fast things (a razor, a fintech app); back-vowel/stop names suit heavy/powerful/durable things (an SUV, a security platform).
- Agency rules of thumb (Lexicon's, not research findings): `v` = vibrant/alive, `b` = reliable, `z` = attention-grabbing, `x` = innovation, `s` = atmospheric/musical.
- **Bouba/kiki:** rounded vowels + voiced labials *feel* soft (Google, Volvo); sharp consonants + front vowels feel spiky (Kit-Kat, Tic Tac). Make the feel match the brand.

**Form.**
- **Syllables:** 1–4, with **2 the sweet spot.** Over 4 and people abbreviate it for you (Federal Express → FedEx).
- **Pronounceable & spellable on first contact.** If you have to explain how to say it (Touareg, Abrdn), it fails. Fluently-pronounceable names are measurably preferred and remembered better.
- **Euphony:** balanced consonant/vowel alternation (CVCV — Lego, Toyota, Yoda), alliteration (Coca-Cola, PayPal), and palindromes (Sonos, Otto) read well.

**Distinctiveness (Abercrombie spectrum — the legal backbone).** Push *up* this ladder; distinctiveness and legal strength rise together. A tier is a provisional call for specific goods in one jurisdiction (US law), not a legal opinion:

| Tier | Protectable? | Examples |
|---|---|---|
| Generic | Never | "Aspirin", "Escalator" |
| Descriptive | Only with proof of acquired distinctiveness (secondary meaning) | American Airlines, Holiday Inn |
| Suggestive | Yes, inherently | Coppertone, Microsoft, Greyhound |
| Arbitrary | Yes, inherently | Apple, Amazon, Camel |
| Fanciful | Strongest | Kodak, Exxon, Verizon |

**Evaluation criteria (Neumeier's 7 — use as a filter, not a generator):** distinctive · brief · appropriate · easy to spell & pronounce · likable · extendable · protectable. (Watkins' **SMILE**: Suggestive, Memorable, Imagery, Legs, Emotional — avoid **SCRATCH**: Spelling-challenged, Copycat, Random, Annoying, Tame, Curse-of-knowledge, Hard-to-pronounce.)

**Knockout search (do it, don't only advise it).** Before handoff, search every top pick — and any shortlist name you suspect — with the web search tool. Run the searches in parallel. For each name, search the exact name plus the category ("Vellum design app"), the bare name as a company or product, and the closest spelling or sound variant. If a tool can reach tmsearch.uspto.gov or the WIPO Global Brand Database, search there too. Classes are not walls: a mark for related goods or services in another class can still conflict. Write what you searched, the date, and what you found into the candidate's `check` field ("web + USPTO, 2026-10-27: live mark VELLUM in fonts, class 9"). Leave `check` empty for names you did not search; the artifact shows them as "not searched yet". A clean search is still not clearance — never call a name "available".

**Risk flags to surface (not clear, just flag):**
- A collision your search found, or one you know of → put it in `flag`.
- Unintended meaning in another language, or an unfortunate substring → for global names, this matters a lot (see `reference.md` §6, §8).
- Genericide risk if the name is too close to the category noun.
- Domain reality: exact-match .com is nice-to-have, not required. `.ai/.io/.co` or a `get-/try-` prefix is fine for most ventures. Don't let domain perfectionism kill a strong name.

## Step 4 — Shortlist with rationale

Return a shortlist — **20–30 survivors in deep, ~20 in quick** — grouped by type so the range shows (don't let one type dominate). Each survivor carries seven fields, plus a flag and a search note when relevant:

> **Vellum** · Lexical · *Old translucent drawing surface — sketches + text, tactile and premium.* · sound: velvet-soft, calm · 2 syl · Arbitrary tier · fit 4/5 · ⚠ a Sketch font is named Vellum — check related software classes.

**Score** each survivor 1–5 for fit to the brief. Legal and language risks stay in `flag`; they are gates, not points.
- **5** — you would lead with it: distinctive in the category, fits the vibe, says and spells itself, extends to sub-brands.
- **4** — strong, with one clear weakness.
- **3** — viable; on the list for range.
- **2** — weak fit, kept only because the user may like the angle.
- **1** — don't shortlist; leave it in the pool.

Then pick your **top 3–5** and give them `rank: 1..5`. Each gets one sentence on why it is strongest against the brief — put it in the `gloss` and repeat it in your chat reply.

**On reactions:** the right name is often *polarizing*, not instantly-loved. Andy Grove on "Pentium": "I see the polarization here … that tells me there's energy." Unanimous first-hearing love often just means the comfortable, category-typical choice. But find out *why* a name divides people: unfamiliarity is fine; a mispronunciation, a bad meaning, or a weak fit is a real objection.

## Output format

**Always produce an HTML artifact**, both modes — a **triage interface** the user compares, filters, and curates, not a report. Don't rebuild it from scratch: **start from `template.html` in this skill folder.** It already implements the layout, the AND filters, the single-`Set` state with saved stars, the shortlist rail, data validation, and the clipboard logic with a fallback — the parts that get re-derived wrong on a blank page.

```
┌─────────────────────────────────────────────────┬────────┐
│  HEADER  project · brief in one line · mode      │        │
├─────────────────────────────────────────────────┤  SHORT │
│  1  COVERAGE BAR  per-type counts, click to filter│  LIST  │
│  2  FILTERS       [type] · [metaphor] · clear     │  PANEL │
│  3  TOP PICKS      ranked cards                   │ (right │
│  4  RECOMMENDED    scored cards, grouped by type  │  rail; │
│  5  THE POOL       compact chips (deep only)      │  below │
│                                                   │ on     │
│                                                   │ mobile)│
├─────────────────────────────────────────────────┴────────┤
│  FOOTER  path · how to copy out                           │
└───────────────────────────────────────────────────────────┘
```

### Fill the template

1. **Inject the data.** Replace `BRIEF` (project, one-line brief, `mode`, absolute path) and `CANDIDATES` with your real pool. Each candidate: `{ id, name, type, metaphor, gloss, sound, syllables, tier, score, rank, flag, check, rec }` — the template's comment block defines each field. Set `rec: true` for your recommended shortlist — those start starred and render as cards; the rest fall to the pool tier. Set `rank` on the top picks. Quick mode (`mode: "quick"`) auto-hides the pool, so mark all ~20 `rec: true`. Write the data as plain JSON-style literals and never put a raw `</` inside a string (write `<\/`).
2. **Retheme, don't restructure.** Per the /html skill, the artifact's personality should fit the *subject* — retheme the `:root` tokens (a finance tool ≠ a kids' app). Keep the layout and JS engine; restyle freely, but keep text at 4.5:1 contrast or better in both themes. Don't reach for the generic AI look (Inter + purple gradient).
3. **Save & report.** One self-contained file to `<repo-root>/docs/html/` in a git repo (create if missing), else `~/Downloads/`. Report the absolute path and an open command. Only drop to plain markdown if the user explicitly asks for no artifact.

### Before handoff — verify

The template writes its own counts to `<body>`: `data-total` (candidates in the data), `data-rendered` (cards plus chips on screen), and `data-errors` (failed data checks, listed in a red banner). Read them with headless Chrome:

```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --dump-dom "file://$OUT" \
  | grep -oE 'data-(total|rendered|errors)="[^"]*"|<li>[^<]*</li>'
```

1. `data-errors="0"`. Otherwise the `<li>` lines name each bad row (duplicate id or name, a `type` not in the taxonomy, a non-boolean `rec`, a pool row in quick mode). Fix the data and rerun.
2. `data-rendered` equals `data-total` (the classic bug: only the shortlist renders, or the pool tier collapses).
3. In deep mode, no coverage row reads 0 — or you can say why that type does not fit.
4. Every top pick has a `check` note. The rest may read "not searched yet".

If a browser tool is available, also confirm the interactions: starring from a card and from a pool chip both update the rail; **Reset** restores the default stars; **Clear** resets the filters; **Copy** gives markdown or, when the clipboard is blocked, a selected text box.

## When to recommend a rebrand vs. keep

If the user is asking whether to *change* an existing name, weigh these in order. The numbers are rules of thumb, not cutoffs.
1. **Must change:** the name is illegal, blocked by a prior mark, or offensive or unpronounceable in a market they're entering.
2. **Lean change:** the name now describes only a small part of the business, about 30% or less (Restoration Hardware → RH); or the name itself carries a lasting negative association (ValuJet → AirTran after the 1996 crash).
3. **Lean keep:** it has years of compounding equity (5+ is a common bar); the pain is the company's reputation, not the name (fix the reputation); or the proposed replacement is no more distinctive than what they have (the Tronc trap).
