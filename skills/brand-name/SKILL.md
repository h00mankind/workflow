---
name: brand-name
description: Generate and screen brand name candidates the way professional naming agencies do, across the full name-type taxonomy (descriptive, suggestive, invented, compound, portmanteau, acronym, synonym-driven, palindrome, reduplicative, foreign, altered-spelling, and more), screening on sound symbolism, syllable/pronounceability, distinctiveness (Abercrombie spectrum), and the standard evaluation criteria. Has two modes — quick (no questions, ~20 candidates, fast) and deep (a 5–10 question grill, then 100+ candidates down to a 20–30-name shortlist); defaults to deep when unspecified. Always outputs an interactive HTML artifact (naming matrix + filterable score cards) where the user stars candidates to build their own shortlist live, with reset and copy-as-markdown. Use whenever the user wants to name a company, product, feature, app, project, startup, band, or anything else; asks for "name ideas", "brand names", "what should I call this", "naming options", a rebrand, or help picking between names they already have. Generates candidates and screens them; stops short of legal trademark clearance and domain registration (it flags risks but is not a substitute for an attorney or a live trademark/domain search).
---

# brand-name

Name things the way Lexicon, Igor, A Hundred Monkeys, and Catchword do: **generate broad, then screen hard.** The breakthrough name is rarely in the first 20 — it's in the second hundred. Your job is to generate a wide, varied candidate set, run it through the screens below, and hand back a shortlist — always as an interactive HTML artifact — with the reasoning attached.

This skill **generates and screens**. It does not clear trademarks or register domains — it flags likely risks and tells the user what to check, but a serious venture still needs a real trademark search (tmsearch.uspto.gov, WIPO Global Brand Database) and an attorney. Say so; don't imply a name is "available."

For deep dives — case studies, the legal doctrine in full, China/global naming, the academic citations, agency philosophies — load `reference.md` in this skill folder. Don't load it for routine naming; the essentials are below.

## Two modes — quick vs. deep

Pick a mode first; it sets how much you ask and how much you generate.

- **`deep` (default).** The full agency process: grill the user with **5–10 questions** to build a real brief, then generate **100+ candidates** across the taxonomy and screen down to a **20–30-name shortlist**. This is the default whenever the mode is ambiguous — naming is a serious exercise and the brief is where quality is won.
- **`quick`.** Fast path: **ask nothing** (or at most one blocker question if the request is unworkable as-is), generate **~20 candidates** across the taxonomy with light screening, and hand back a **~20-name shortlist**. Use when the user wants speed.

**Choosing the mode:** honor an explicit `quick` / `deep` in the request. If neither is stated, infer from signal — a one-line "name my X" with little detail and a hurry-vibe → quick; a richer prompt, a real venture, or "help me name…" → deep. **When genuinely ambiguous, default to deep**, and say so in one line ("Running in deep mode — I'll ask a few questions first; say *quick* if you'd rather I just generate"). The user can switch modes at any time.

Both modes produce the **same kind of deliverable** (the HTML artifact below) — they differ in how much you ask and how many candidates you generate.

## Step 1 — Get a brief

The quality of the names is set here, not in generation. How much you ask depends on the mode.

### Deep mode — grill the user (5–10 questions)

Build a real brief. Ask **5–10 questions** before generating — prefer batching them (the `AskUserQuestion` tool takes up to 4 at once, so 2–3 rounds). Don't ask what the prompt already answers; spend the questions on what's missing and what shapes the name. Draw from:

- **What is it, exactly?** One sentence on what the thing does and what makes it different.
- **Positioning / vibe.** Playful or serious? Premium or accessible? Bold or trustworthy? Warm or sharp? What feeling should it trigger on first hearing?
- **Audience.** Who's it for — demographics, sophistication, region? What do they call this category today?
- **Competitors.** Name 3–5. You'll plot them and name deliberately *away* from their cluster.
- **Name architecture.** Standalone brand, or sits under a parent? Will it need sub-brands / product names later (extendability)?
- **Tone & forbidden territory.** Words/themes that must appear, must be avoided, or that competitors own. Any founder favorites or vetoes?
- **Register / language.** English-only, or should it work across languages? Any cultural markets to respect (see `reference.md` §6, §10)?
- **Scope & constraints.** Global or one market? Domain/TLD flexibility (.com required, or .ai/.io/.co fine)? Trademark classes that matter?
- **Reaction appetite.** Do they want safe-and-clear, or are they open to a polarizing, distinctive name? (This calibrates how far up the taxonomy you push.)

Reflect the brief back in a sentence or two before generating, so the user can correct course cheaply.

### Quick mode — skip the grill

Don't interrogate. Generate from whatever the user gave you and **state your assumptions** inline. Ask **at most one** question, and only if the request is genuinely unworkable without it (e.g. you don't even know what the product is). Otherwise infer vibe, audience, and category from context and move straight to generation.

## Step 2 — Generate broad, across the taxonomy

Generate by mode: **deep → 100+ candidates**, **quick → ~20**. Either way, deliberately spread them across name *types* so the user sees real range — not many variations of one idea. Don't converge on the category-typical pattern; that's the failure mode of lazy naming (and of AI name generators). In deep mode, hit **every** type below — if a type is empty in your output, you under-generated. In quick mode, cover a broad sample of types but don't force all 16. The types:

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

Don't just dump a list. Return a shortlist — **20–30 survivors in deep mode, ~20 in quick** — grouped by type (cover the range — don't let one type dominate the shortlist), each with: the name, its type, a one-line "why it works," its sound-symbolism read, syllable count, Abercrombie tier, and any risk flag. Then call out your **top 3–5 picks** with a sentence each on why they're strongest against the brief.

**A note on reactions:** the right name is often *polarizing*, not instantly-loved. Andy Grove on "Pentium": "I see the polarization here … that tells me there's energy." If the user's team unanimously loves one on first hearing, it may just be the comfortable, category-typical choice. Don't optimize for the safe favorite.

## Output format

**Always produce an HTML artifact** (via the `to-html` skill) — in both modes. A naming result is inherently spatial *and* interactive: the user wants to compare candidates, filter, and curate their own shortlist. So the artifact isn't a static report — it's a **triage interface** the user drives. Build:

- a **naming matrix** (e.g. Descriptive↔Abstract × Real↔Coined, or axes tailored to the brief) with candidates plotted, and the category/competitor cluster drawn as a zone to name *away* from;
- **score cards for every candidate** (name, type, why, sound read, syllable count, Abercrombie tier, risk flag), filterable by type/theme. Show the **whole pool**, not just the pre-picked shortlist — in deep mode the 100+ candidates, in quick mode the ~20. Don't bury the pool in a collapsed drawer; it's the thing the user picks from.

**Interactive shortlisting (required).** Every candidate card has a **star / checkbox** to add or remove it from *the user's* shortlist. The shortlist you screened to is just the **default starred set** — the user overrides it by starring/unstarring any card in the pool. Wire up:

- a **live shortlist panel** (a sticky sidebar or header bar) that updates the instant a card is toggled, showing the current picks and a count;
- a **Reset** button that restores the default starred set (your recommended shortlist);
- a **Copy** button that **copies the user's current shortlist to the clipboard as markdown** — name, type, why, and flag per pick — so it pastes straight back into a prompt. Label it "Copy", not "Save": it copies to the clipboard, it does not persist. (`localStorage` throws in sandboxed-iframe artifacts anyway, so keep all selection state in JS variables for the session; the user copies out before closing.)

Keep selection state in a plain JS structure (e.g. a `Set` of candidate ids); the star buttons, the live panel, the count, Reset, and Copy all read from it. No browser storage.

**Follow the `to-html` skill** for everything else about the artifact — design tokens, theme switcher, single self-contained file, overflow guardrails, no browser storage, **and where to save it.** Don't hardcode a destination here; `to-html` decides the path and reports it. One addition: naming outputs are private working material, so if it lands in a repo, make sure that artifact directory is gitignored (add the rule if it isn't) — candidates shouldn't get published.

Only drop to plain markdown if the user explicitly asks for a chat answer / no artifact.

## When to recommend a rebrand vs. keep

If the user is asking whether to *change* an existing name:
- **Change** when: the name now describes ≤30% of the business (Restoration Hardware → RH); it carries a serious negative association (Tronc, Andersen post-Enron); or it's illegal/unpronounceable/offensive in a market they're entering.
- **Keep** when: it has 5+ years of compounding equity; the pain is reputational not strategic (fix the reputation); or the proposed replacement is no more distinctive than what they have (the Tronc trap).
