---
name: unslop
description: Use to remove AI writing patterns from supplied prose while preserving facts and author style. Skip product, website, and UI copy.
license: MIT
---
# unslop

Line edit without replacing the author's style. Remove AI patterns and keep the style suitable for the author and audience.

## Contract

- Preserve meaning, claims, facts, technical terms, and level of certainty.
- Match the source's formality, dialect, point of view, and intended audience.
- Do not add facts, sources, anecdotes, opinions, slang, or deliberate errors to simulate a human.
- Return the revised text first. Explain edits only when the user asks.

## Process

1. **Read for intent.** Identify the core claim, author, audience, and tone from the source or the user's instructions. Ask only if missing information can materially change the edit.
2. **Mark the patterns.** Find empty language, repeated structures, canned transitions, strained punctuation, and chatbot artifacts. Each planned change must match a pattern below or fix grammar.
3. **Rewrite complete sentences.** Rebuild weak sentences around concrete subjects and actions instead of swapping words one at a time. Every source claim must remain accounted for.
4. **Keep the author's style.** Keep distinctive details and phrases that are not AI patterns. Preserve the source's point of view, formality, and certainty. Every detail in the result must come from the source or the user.
5. **Audit the result.** Compare it with the source. Names, numbers, claims, relationships, and uncertainty must not change. No pattern below may remain unless it is needed for meaning or fits the established voice.

## Patterns to remove

### Empty content

- **Unsupported praise:** "pivotal", "groundbreaking", "a testament to", "sets the stage", "ever-evolving landscape". State what happened.
- **Promotion disguised as description:** "vibrant", "renowned", "breathtaking", "must-visit". Give an observable fact or cut it.
- **Vague authority:** "experts say", "research suggests", "critics argue". Name the source if supplied. If the claim depends on an unavailable source, keep its uncertainty and flag it instead of turning it into a fact.
- **Empty participles:** clauses that end with "highlighting", "showcasing", "fostering", or "underscoring" without adding evidence.
- **Generic framing and conclusions:** "Despite its challenges", "In today's world", "The future looks bright". Start or end with the actual point.

### Inflated language

- Prefer `use`, `help`, `many`, and `if` over `utilize`, `facilitate`, `numerous`, and `in the event that`.
- Replace decorative abstractions such as "tapestry", "interplay", "paradigm", "north star", "flywheel", "substrate", and "scaffolding" with the real mechanism.
- Replace "serves as", "stands as", "boasts", and "features" with `is` or `has` when that is the meaning.
- Remove "not only X, but Y", false "from X to Y" ranges, and forced contrasts. State the point directly.
- Use one term per concept. Do not cycle through synonyms to avoid useful repetition.
- Do not force points into groups of three.

### Repeated structure

- Vary sentence length when the source permits it. Do not make each paragraph the same size or cadence.
- Prefer active voice when the actor matters: "The compiler validates the query" instead of "The query is validated."
- Split sentences that require a second read. Keep one main thought per sentence.
- Use headings only when they help navigation. Use sentence case.
- Remove stacked bold lead-ins, decorative emoji, and unnecessary summary sections.
- Use em dashes, colons, parentheses, and rhetorical questions only when they fit the voice, not as default sentence connectors.

### Chatbot phrases

- Remove greetings, praise, and sign-offs such as "Great question", "Certainly", "I hope this helps", and "Let me know if you need anything else."
- Remove disclaimers about missing context or knowledge limits unless they change what the reader must know.
- Cut filler openings: "It is important to note that", "In order to", "Due to the fact that", "When it comes to".
- Reduce stacked hedges to the source's actual uncertainty. Do not turn `may` into certainty.

## Keep the author's style

Keep concrete details from the source. Use contractions and first person only when they match the source. Vary sentence length when the source permits it. Preserve clear disagreement or mixed opinions instead of replacing them with a neutral list.

Do not add errors or informal language to make the text seem human. Clear and correct prose is acceptable.

## Example

Before: "In today's rapidly evolving software landscape, Acme's groundbreaking platform not only flags duplicate CSV rows but also empowers teams by showing validation errors before upload."

After: "Acme flags duplicate CSV rows and shows validation errors before upload."

The revision removes framing and praise but keeps each supplied fact. It does not add a new mechanism or result.
