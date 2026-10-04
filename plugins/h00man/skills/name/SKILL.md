---
name: name
description: Generate and screen brand names across descriptive to invented types with sound and distinctiveness checks. Use to name a company, product, or feature; quick gives 20 names, deep grills then 100-plus into an HTML shortlist. Not legal clearance.
---

# name

Name things the way Lexicon, Igor, A Hundred Monkeys, and Catchword do: **generate broad, then screen hard.** The breakthrough name is rarely in the first 20 — it's in the second hundred. Your job is to generate a wide, varied candidate set, run it through the screens below, and deliver the **whole pool plus a recommended shortlist** — always as an interactive HTML artifact the user can filter and re-curate — with the reasoning attached.

This skill **generates and screens**. It does not clear trademarks or register domains — it flags likely risks and tells the user what to check, but a serious venture still needs a real trademark search (tmsearch.uspto.gov, WIPO Global Brand Database) and an attorney. Say so; don't imply a name is "available."

For deep dives — case studies, the legal doctrine in full, China/global naming, the academic citations, agency philosophies — load `reference.md` in this skill folder. Don't load it for routine naming; the essentials are below.

## Two modes — quick vs. deep

Same deliverable (the HTML artifact below); they differ only in how much you ask and how many candidates you generate.

- **`deep` (default).** Grill the user with **5–10 questions**, generate **100+ candidates** across the taxonomy, screen down to a **20–30-name shortlist**. The brief is where quality is won, so this is the default whenever the mode is ambiguous.
- **`quick`.** Ask nothing (at most one blocker question), generate **~20 candidates** with light screening, hand back a **~20-name shortlist**.

Honor an explicit `quick` / `deep`. Otherwise infer from signal — terse "name my X" with a hurry-vibe → quick; a richer prompt or real venture → deep. When genuinely ambiguous, default to deep and say so in one line ("Running in deep mode — I'll ask a few questions first; say *quick* to skip"). The user can switch anytime.

## Step 1 — Get a brief

The quality of the names is set here, not in generation.

**Deep mode — grill (5–10 questions).** Batch them (`AskUserQuestion` takes 4 at once, so 2–3 rounds). Don't ask what the prompt already answers; spend questions on what shapes the name:

- **What is it?** One sentence on what it does and what makes it different.
- **Vibe.** Playful/serious, premium/accessible, warm/sharp — what feeling on first hearing?
- **Audience.** Who's it for, and what do they call this category today?
- **Competitors.** Name 3–5 — you'll name deliberately *away* from their pattern.
- **Architecture.** Standalone, or under a parent? Need sub-brands later (extendability)?
- **Forbidden territory.** Words/themes that must appear, must be avoided, or that competitors own. Founder vetoes?
- **Language.** English-only or cross-language? Cultural markets to respect (`reference.md` §6, §10)?
- **Constraints.** One market or global? Domain flexibility (.com required, or .ai/.io/.co fine)? TM classes that matter?
- **Reaction appetite.** Safe-and-clear, or open to a polarizing name? (Calibrates how far up the taxonomy to push.)

Reflect the brief back in a sentence before generating, so the user can correct course cheaply.

**Quick mode — skip the grill.** Generate from what you were given and **state your assumptions inline**. Ask at most one question, only if the request is unworkable without it (e.g. you don't know what the product is).

## Step 2 — Generate broad, across the taxonomy

Generate by mode: **deep → 100+ candidates**, **quick → ~20**. Either way, deliberately spread them across name *types* so the user sees real range — not many variations of one idea. Don't converge on the category-typical pattern; that's the failure mode of lazy naming (and of AI name generators). In deep mode, hit **every** type in the table below — if a type has no candidates in your output, you under-generated. In quick mode, cover a broad sample of types but don't force one of each. The types:

| Type | What it is | Examples |
|---|---|---|
| **Descriptive** | Says what it does | PayPal, The Home Depot |
| **Suggestive / evocative** | Hints at a quality; needs a small leap | Amazon, Slack, Patagonia |
| **Associative / metaphoric** | Borrows imagery from another domain | Nike, Starbucks, Jaguar |
| **Synonym-driven** | A synonym/near-synonym of the core concept used directly | Stripe, Notion, Ramp, Vise |
| **Invented / coined** | Built letter-string, no prior meaning | Kodak, Xerox, Verizon |
| **Abstract / arbitrary** | Real word, unrelated use | Apple, Camel, Shell |
| **Compound** | Two real words fused whole | Facebook, Snowflake, Salesforce |
| **Portmanteau / blend** | Two words *chopped* and merged | Pinterest, Instagram, Groupon, Microsoft |
| **Acronym / initialism** | Reduced to initial letters (pronounced as word or spelled out) | IBM, KFC, NASA, IKEA |
| **Lexical / real-word** | A plain dictionary word | Oracle, Visa, Dove |
| **Foreign / borrowed** | A word from another language | Volvo ("I roll"), Lego ("play well"), Audi |
| **Palindrome** | Reads the same backwards | Sonos, Otto, Honda*, Aviva, Civic |
| **Reduplicative / rhyming** | Repeated or rhyming syllables | Google, TikTok, Kit-Kat, Yoyo, Juju |
| **Mimetic / onomatopoeic** | The sound is part of the meaning | Twitter, Zappos, Yahoo! |
| **Altered spelling** | Familiar word respelled | Lyft, Flickr, Reddit |
| **Affixed** | Root + productive prefix/suffix | Spotify, Calendly, Twilio, Verizon |

(*Honda isn't a true palindrome — use it loosely as a near-symmetric, balanced form.)

Generation tactics that produce range: **morpheme tables** (prefixes `inter-/trans-/ultra-/neo-`, suffixes `-ify/-ly/-io/-ai/-ex/-on/-ium/-os`); **synonym/thesaurus walks** of the core concept; **foreign-language and Latin/Greek roots**; **metaphor mining** ("if this were an animal / a place / a sound / a myth, what would it be?"); **acronym hunts** (write the descriptive phrase, then mine its initials for a pronounceable string); **sound-pattern templates** (CVCV, alliteration, palindromes, reduplication); **etymology walks** from the root word outward.

**Default to suggestive, arbitrary, invented, portmanteau, or palindrome.** Avoid purely descriptive names unless the user insists — they're weak legally and forgettable, and successful ones eventually rebrand (KFC, RH, WW). Acronyms are usually weak as *primary* names (image-free, hard to trademark) unless they spell a pronounceable word (IKEA, NASA) — generate a few, but don't lead with them.

## Step 3 — Screen

Run candidates through these screens. The first pass kills the obvious losers; report the survivors with their scores.

**Sound symbolism (engineer the sound, not just the meaning).** Phonemes carry meaning automatically:
- **Front vowels** (i, ee) → small, light, fast, sharp, friendly, feminine. **Back vowels** (a, o, u) → large, heavy, powerful, serious.
- **Fricatives** (f, s, v, z) → soft, light, fast. **Stops/plosives** (b, d, g, p, t, k) → hard, crisp, energetic.
- Match the sound to the product: front-vowel/fricative names suit light/elegant/fast things (a razor, a fintech app); back-vowel/stop names suit heavy/powerful/durable things (an SUV, a security platform).
- Specific letter signals practitioners use: `v` = vibrant/alive, `b` = reliable, `z` = attention-grabbing, `x` = innovation, `s` = atmospheric/musical.
- **Bouba/kiki:** rounded vowels + voiced labials *feel* soft (Google, Volvo); sharp consonants + front vowels feel spiky (Kit-Kat, Tic Tac). Make the feel match the brand.

**Form.**
- **Syllables:** 1–4, with **2 the sweet spot.** Over 4 and people abbreviate it for you (Federal Express → FedEx).
- **Pronounceable & spellable on first contact.** If you have to explain how to say it (Touareg, Abrdn), it fails. Fluently-pronounceable names are measurably preferred and remembered better.
- **Euphony:** balanced consonant/vowel alternation (CVCV — Honda, Toyota), alliteration (Coca-Cola, PayPal), and palindromes (Sonos, Otto) read well.

**Distinctiveness (Abercrombie spectrum — the legal backbone).** Push *up* this ladder; distinctiveness and legal strength rise together:

| Tier | Protectable? | Examples |
|---|---|---|
| Generic | Never | "Aspirin", "Escalator" |
| Descriptive | Only after years of use | American Airlines, Holiday Inn |
| Suggestive | Yes, inherently | Coppertone, Microsoft, Greyhound |
| Arbitrary | Yes, inherently | Apple, Amazon, Camel |
| Fanciful | Strongest | Kodak, Exxon, Verizon |

**Evaluation criteria (Neumeier's 7 — use as a filter, not a generator):** distinctive · brief · appropriate · easy to spell & pronounce · likable · extendable · protectable. (Watkins' **SMILE**: Suggestive, Memorable, Imagery, Legs, Emotional — avoid **SCRATCH**: Spelling-challenged, Copycat, Random, Annoying, Tame, Curse-of-knowledge, Hard-to-pronounce.)

**Risk flags to surface (not clear, just flag):**
- Obvious trademark collision in the category → tell them to run a knockout search at tmsearch.uspto.gov / WIPO before falling in love.
- Unintended meaning in another language, or an unfortunate substring → for global names, this matters a lot (see `reference.md` §6, §10).
- Genericide risk if the name is too close to the category noun.
- Domain reality: exact-match .com is nice-to-have, not required. `.ai/.io/.co` or a `get-/try-` prefix is fine for most ventures. Don't let domain perfectionism kill a strong name.

## Step 4 — Shortlist with rationale

Return a shortlist — **20–30 survivors in deep, ~20 in quick** — grouped by type so the range shows (don't let one type dominate). Each survivor carries six fields, plus a flag when relevant:

> **Vellum** · Lexical/arbitrary · *Old translucent drawing surface — sketches + text, tactile and premium.* · sound: velvet-soft, calm · 2 syl · Arbitrary tier · ⚠ a Sketch font is named Vellum — check the TM class.

Then call out your **top 3–5 picks**, one sentence each on why they're strongest against the brief.

**On reactions:** the right name is often *polarizing*, not instantly-loved. Andy Grove on "Pentium": "I see the polarization here … that tells me there's energy." Unanimous first-hearing love often just means the comfortable, category-typical choice. Don't optimize for the safe favorite.

## Output format

**Always produce an HTML artifact**, both modes — a **triage interface** the user compares, filters, and curates, not a report. Don't rebuild it from scratch: **start from `template.html` in this skill folder.** It already implements the layout, the AND filters, the single-`Set` state, the sticky shortlist rail, and the clipboard/verification logic — the parts that get re-derived wrong on a blank page.

```
┌─────────────────────────────────────────────────┬────────┐
│  HEADER  project · brief in one line · mode      │        │
├─────────────────────────────────────────────────┤  SHORT │
│  1  COVERAGE BAR  per-type counts, click to filter│  LIST  │
│  2  FILTERS       [type] · [metaphor] · clear     │  PANEL │
│  3  RECOMMENDED    score cards, grouped by type   │ (right │
│  4  THE POOL       compact chips (deep only)      │  rail) │
├─────────────────────────────────────────────────┴────────┤
│  FOOTER  path · how to copy out                           │
└───────────────────────────────────────────────────────────┘
```

### Fill the template

1. **Inject the data.** Replace `BRIEF` (project, one-line brief, `mode`, absolute path) and `CANDIDATES` with your real pool. Each candidate: `{ id, name, type, metaphor, gloss, sound, syllables, tier, flag, rec }`. Set `rec: true` for your recommended shortlist — those start starred and render as cards; the rest fall to the pool tier. Quick mode (`mode: "quick"`) auto-hides the pool, so mark all ~20 `rec: true`.
2. **Retheme, don't restructure.** Per the /html skill, the artifact's personality should fit the *subject* — retheme the `:root` tokens (a finance tool ≠ a kids' app). Keep the layout and JS engine; restyle freely. Don't reach for the generic AI look (Inter + purple gradient).
3. **Save & report.** One self-contained file to `<repo-root>/docs/html/` in a git repo (create if missing), else `~/Downloads/`. Report the absolute path and an open command. Only drop to plain markdown if the user explicitly asks for no artifact.

### Before handoff — verify

The template's own header comment lists how to fill it; these are the behaviors to confirm after you do. Drive a browser tool if available; otherwise read back the counts.

1. Rendered candidate count equals what you generated (the classic bug: only the shortlist renders, or the pool tier collapses).
2. Starring from a card *and* from a pool chip both update the rail count.
3. **Reset** restores the default starred set; **Clear** resets the filters.
4. Type + metaphor filters intersect (AND); the coverage bar counts sum to the full pool.
5. **Copy** writes markdown and fails loudly (the template already `.catch`es the clipboard reject).

## When to recommend a rebrand vs. keep

If the user is asking whether to *change* an existing name:
- **Change** when: the name now describes ≤30% of the business (Restoration Hardware → RH); it carries a serious negative association (Tronc, Andersen post-Enron); or it's illegal/unpronounceable/offensive in a market they're entering.
- **Keep** when: it has 5+ years of compounding equity; the pain is reputational not strategic (fix the reputation); or the proposed replacement is no more distinctive than what they have (the Tronc trap).
