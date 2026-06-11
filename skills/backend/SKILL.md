---
name: backend
description: Working discipline for backend work — endpoints, services, data models, migrations, and jobs. Use when building or changing APIs, server logic, database schemas, queues, webhooks, or integrations.
---

# backend

Read before write. Trace one existing endpoint end-to-end first — route, handler, service, data access — and match its layering, error handling, and naming.

## Boundaries

- **Validate at the edge, once.** Parse and validate external input at the boundary; from there, types carry the guarantee inward. Don't re-validate internal calls.
- **Authz is explicit.** Every endpoint has a deliberate answer to "who may call this?" — even when the answer is "any signed-in user". Check object-level access, not just login.

## Errors

- Expected failures are return values or typed errors; exceptions are for the unexpected.
- Error responses say what went wrong without leaking internals or stack traces.
- Log at the boundary with enough context to debug. No secrets or PII in logs.

## Data

- Migrations are reversible — or explicitly flagged destructive and confirmed before running.
- Mutations that can be retried (webhooks, jobs, payments) are idempotent.
- Input never gets interpolated into queries or shell commands — parameterize.

## Verify

Hit the real endpoint before reporting done — the happy path and at least one failure path (bad input, missing auth). A passing typecheck is not verification.
