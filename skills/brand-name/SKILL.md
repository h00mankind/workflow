---
name: brand-name
description: Generate and screen brand name candidates the way professional naming agencies do — produce a large, varied candidate set across the name-type taxonomy, then screen it on sound symbolism, syllable/pronounceability, distinctiveness (Abercrombie spectrum), and the standard evaluation criteria. Use whenever the user wants to name a company, product, feature, app, project, startup, band, or anything else; asks for "name ideas", "brand names", "what should I call this", "naming options", a rebrand, or help picking between names they already have. Generates candidates and screens them; stops short of legal trademark clearance and domain registration (it flags risks but is not a substitute for an attorney or a live trademark/domain search).
---

# brand-name

Name things the way Lexicon, Igor, A Hundred Monkeys, and Catchword do: **generate broad, then screen hard.** The breakthrough name is rarely in the first 20 — it's in the second hundred. Your job is to generate a wide, varied candidate set and run it through the screens below, then hand back a shortlist with the reasoning attached.

This skill **generates and screens**. It does not clear trademarks or register domains — it flags likely risks and tells the user what to check, but a serious venture still needs a real trademark search (tmsearch.uspto.gov, WIPO Global Brand Database) and an attorney. Say so; don't imply a name is "available."

For deep dives — case studies, the legal doctrine in full, China/global naming, the academic citations, agency philosophies — load `reference.md` in this skill folder. Don't load it for routine naming; the essentials are below.

## Step 1 — Get a brief (don't skip this)

The quality of the names is set here, not in generation. If the user just says "name my app," ask for what you're missing before generating — 2–4 quick questions, not a form:

- **What is it?** One sentence on what the thing does.
- **Positioning / vibe.** Playful or serious? Premium or accessible? Bold or trustworthy? What feeling should it trigger?
- **Audience & category.** Who's it for, and who are the competitors? (You'll deliberately name *away* from the category cluster.)
- **Scope & constraints.** Global or one market? Any words to avoid, must-include, or founder preferences? Domain/TLD flexibility (.com required, or .ai/.io/.co fine)?

If the user wants speed over a brief, generate anyway but state the assumptions you made.

## Step 2 — Generate broad, across the taxonomy

Generate **40–150 candidates** (more for serious projects), deliberately spread across name *types* so the user sees real range — not 50 variations of one idea. Don't converge on the category-typical pattern; that's the failure mode of lazy naming (and of AI name generators). The types:

| Type | What it is | Examples |
|---|---|---|
| **Descriptive** | Says what it does | PayPal, The Home Depot |
| **Suggestive / evocative** | Hints at a quality; needs a small leap | Amazon, Slack, Patagonia |
| **Associative / metaphoric** | Borrows imagery from another domain | Nike, Starbucks, Jaguar |
| **Invented / coined** | Built letter-string, no prior meaning | Kodak, Xerox, Verizon |
| **Abstract / arbitrary** | Real word, unrelated use | Apple, Camel, Shell |
| **Compound** | Two real words fused | Facebook, Snowflake, Salesforce |
| **Blend / portmanteau** | Two words chopped + merged | Pinterest, Instagram, Groupon |
| **Lexical / real-word** | A plain dictionary word | Oracle, Stripe, Notion |
| **Foreign / borrowed** | A word from another language | Volvo ("I roll"), Lego ("play well") |
| **Altered spelling** | Familiar word respelled | Lyft, Flickr, Reddit |

Generation tactics that produce range: morpheme tables (prefixes `inter-/trans-/ultra-`, suffixes `-ify/-ly/-io/-ai/-ex/-on/-ium`); foreign-language and Latin/Greek roots; metaphor mining ("if this were an animal / a place / a sound, what would it be?"); thesaurus/etymology walks; sound-pattern templates (CVCV, alliteration, palindromes).

**Default to suggestive, arbitrary, invented, or compound.** Avoid purely descriptive names unless the user insists — they're weak legally and forgettable, and successful ones eventually rebrand (KFC, RH, WW).

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

Don't just dump a list. Return a **shortlist of ~8–15 survivors**, grouped by type, each with: the name, its type, a one-line "why it works," its sound-symbolism read, syllable count, Abercrombie tier, and any risk flag. Then call out your **top 3–5 picks** with a sentence each on why they're strongest against the brief.

**A note on reactions:** the right name is often *polarizing*, not instantly-loved. Andy Grove on "Pentium": "I see the polarization here … that tells me there's energy." If the user's team unanimously loves one on first hearing, it may just be the comfortable, category-typical choice. Don't optimize for the safe favorite.

## Output format

Default to **markdown** in the conversation — fast, scannable, easy to react to.

Reach for an **HTML artifact** (via the `to-html` skill) when the deliverable is something the user will *compare spatially*: a naming matrix (Descriptive↔Abstract × Real↔Coined 2×2), a competitor taxonomy chart with the user's candidates plotted against the category cluster, or score cards for a finalist set. Those are exactly the "side-by-side / at-a-glance" cases `to-html` is for. If you go that route, follow the `to-html` skill (h00man tokens + theme switcher). Offer it rather than assuming — e.g. "Want this as an interactive naming matrix?"

## When to recommend a rebrand vs. keep

If the user is asking whether to *change* an existing name:
- **Change** when: the name now describes ≤30% of the business (Restoration Hardware → RH); it carries a serious negative association (Tronc, Andersen post-Enron); or it's illegal/unpronounceable/offensive in a market they're entering.
- **Keep** when: it has 5+ years of compounding equity; the pain is reputational not strategic (fix the reputation); or the proposed replacement is no more distinctive than what they have (the Tronc trap).
