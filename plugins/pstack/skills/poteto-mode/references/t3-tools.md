# T3 Code delegation mapping for pstack

T3 Code is not a runtime of its own. A T3 thread runs on a provider's harness, such as Claude Code or Codex, and that harness's mapping still applies: the skills as written on Claude Code, [`codex-tools.md`](codex-tools.md) on Codex. T3 adds one thing, the `t3-code` MCP server, whose `delegate_task` tool starts a child agent on any provider and model the app has configured. Read this file when the session exposes `delegate_task` and a pstack skill dispatches a subagent on a model the harness's own subagent tool cannot run.

The harness prefixes the tool names. Claude Code shows `mcp__t3-code__delegate_task`, and Codex shows `mcp__t3_code__delegate_task`. This file uses the bare names.

## When to use which tool

| Dispatch | Tool |
|----------|------|
| A subagent on a model the harness's own tool accepts (the `Agent` tool on Claude Code, `spawn_agent` on Codex) | The harness's own tool, as the skill and the harness mapping say. Each dispatch field the skill names keeps the meaning it has on that harness. |
| A subagent on another provider's model, or on a model the harness's own tool does not list | `delegate_task` |
| A subagent that writes files | The harness's own tool, with a worktree for each writer as the harness mapping says. See [Subagent policy](#subagent-policy). |

The harness's own model list is not the full list. `orchestrator_capabilities` returns every provider instance with its models, and it is the only source for a `delegate_task` target. Use a provider only when its `canRunChildTask` is true, and when it is not the session's own provider, only when `canRunCrossProviderChildTask` is true too. A provider that cannot run a child lists the reason in `constraints`, such as a missing login.

## Tool actions

| pstack / Claude action | T3 equivalent |
|------------------------|---------------|
| Dispatch a subagent on another provider (the `Agent` tool with a `model`) | `delegate_task` with `task` (the whole prompt), `target.providerInstanceId`, and `target.model`, both copied from `orchestrator_capabilities` |
| `run_in_background: true` | `mode: "async"`, which is the default. The call returns a `taskId` at once. Keep it. |
| A foreground subagent | `mode: "wait"`. `timeoutMs` is only how long you wait. When it runs out the call returns `waitTimedOut: true` and the child keeps running, so keep the `taskId`. |
| Dispatch N parallel subagents in one turn | N `delegate_task` calls in one response, each with its own `clientRequestId` |
| Wait for a subagent result | The completion arrives as a message that names the `taskId`. Then read the result with `task_status`, whose `summary` is the child's final reply. Do not poll and do not sleep. Call `task_status` earlier only when the turn cannot continue without the result. |
| Stop a subagent | `task_cancel` with the `taskId` |
| A label for the subagent (`description`) | `title` |
| `subagent_type` | No equivalent. See [Subagent policy](#subagent-policy). |
| `readonly: true` | No equivalent. See [Subagent policy](#subagent-policy). |
| `isolation: "worktree"` | No equivalent. See [Subagent policy](#subagent-policy). |
| A role value's `@<level>` | An entry in `target.options`. See [Model names](#model-names). |
| The sheet's `fast mode` line | An entry in `target.options`. See [Fast mode](#fast-mode). |

A `task_status` result is done when `workState` is `result_available`. `waiting_for_children` means the child's turn ended while work it started is still running, so the task is not finished.

Retry a `delegate_task` call with the same `clientRequestId`, so a lost response does not start a second child.

## Subagent policy

poteto-mode's Subagents section applies. These points differ for a `delegate_task` child:

- The child starts with the `task` text and nothing else. It gets no parent history, so the file-pointers rule matters more: name every file by absolute path.
- The child may not see the pstack skills. A provider loads only what is installed for it, and a T3 child saw none of them in the session that verified this mapping. Do not tell a child to invoke a skill by name. Give it the absolute path of the file to read first: `poteto-mode/SKILL.md` for the `pstack:poteto-agent` style, `poteto-mode/references/agents/comment-sicko.md` for `pstack:comment-sicko`, and the reviewer or explorer prompt file the skill names for a panel seat.
- The child runs in the parent's working directory, on the parent's checkout and branch. `delegate_task` has no worktree option, and a child that runs `git worktree add` or `cd` does not move its T3 binding. So use `delegate_task` for seats that edit no source: panel reviewers, critics, judges, explorers, investigators, and verifiers. Keep writers (code delegates, **swarm** workers, the `orchestrate` and autopilot playbooks) on the harness's own tool, each in its own worktree. On Claude Code that is `isolation: "worktree"`. Codex's `spawn_agent` has no such field, so make the worktree first and name its path in the instructions.
- A verifier runs the app and the tests and edits no source, so a `verifiers` value of the form `<providerInstanceId>/<model>` is a `delegate_task` call. Name the exact SHA in the `task`. The child sees no driver skill, so also give it the absolute path of the project's `verify/SKILL.md` to read first, or the commands that run the app and the tests when the project has no such skill. Where the playbook gives the verifier its own worktree, tell it to add a detached worktree at that SHA in a temporary directory outside the checkout, to run there, and to remove the worktree when it is done. Shell commands run in that directory as usual. Only T3's record of the thread stays on the parent's checkout. Writing or changing a test is a write, and it stays with a code role on the harness's own tool.
- Nothing enforces read-only. `interactionMode: "plan"` did not stop a Codex child writing a file when this mapping was verified. Say in the `task` that the child must not change files, and check `git status` when a read-only child returns.
- The child has `delegate_task` too. Tell a panel seat not to delegate, so the fan-out stays the size the skill set.
- Provider, model, runtime mode, and interaction mode inherit from the parent unless the call sets them. Leave `runtimeMode` alone: a child that waits for an approval nobody sees does not finish.
- A separate top-level thread (`t3_thread_launch`, `create_threads`) is not a subagent. Use one only when the user asks for a separate thread.
- Keep the rest of the policy unchanged. Review every child's result yourself, and write your own summary.

## Model names

Skills name models by the family names in their Models sections. On T3 those names still go to the harness's own subagent tool, resolved as on that harness: as written on Claude Code, and replaced by your Codex models on Codex. A role value in the override sheet that has the form `<providerInstanceId>/<model>` goes to `delegate_task`, with the part before the first `/` as `target.providerInstanceId` and the rest as `target.model`. For example, `codex/gpt-6-sol` is the Codex instance and its `gpt-6-sol` model. A model id may contain `/` itself.

A diverse-model panel (`arena`, `architect`, `interrogate`, `reflect`) gets its adversarial signal from model diversity, and T3 is where one panel can cross providers. A sheet line such as `interrogate reviewers: <family name>, codex/gpt-6-sol, <another provider>/<model>` runs the first seat on the harness's own tool and the other two through `delegate_task`. The defaults stay on one provider. Cross-provider seats are the user's choice in the sheet, because each one bills another account.

A value's `@<level>` goes in `target.options`. Each model in `orchestrator_capabilities` lists its options, and the reasoning option has a different id per provider: `reasoningEffort` on a Codex model and `effort` on a Claude model when this mapping was verified. Pass `{"<that id>": "<level>"}`, and only a level the model lists. A value without `@` takes the sheet's `default effort` level in the same way. `session` passes no option, and the child then runs at the provider's own default.

Never write a `<providerInstanceId>/<model>` value that `orchestrator_capabilities` did not list in this session.

## Fast mode

A provider can run a child in a faster mode that uses more of the account's allowance, and a provider can have that mode on by default. The Codex models listed `serviceTier` with `priority`, labelled Fast, as the default when this mapping was verified.

The sheet's `fast mode` line sets this for every `delegate_task` call. With `off`, add the option that turns fast mode off to `target.options`, beside the reasoning option: `"serviceTier": "default"` on a Codex model and `"fastMode": false` on a Claude model. On another provider, use the option that `orchestrator_capabilities` labels as fast mode or service tier, and pass nothing when the model lists none. With `provider`, or with no line, pass no fast-mode option.

The line does not reach the harness's own subagent tool. On Claude Code a native subagent has no fast-mode field, and fast mode there is the session's setting.

When this mapping was verified, T3 recorded `serviceTier: default` in the configuration of a Codex child that was given the option, and recorded no service tier for a child that was not. The child could not read its own tier, so the provider's side was not observed.

## Preset

A sheet for a T3 Code thread that runs on Claude Code with a Codex provider. Most work runs on one Claude model at high effort, judgment and synthesis on another at medium, review on Codex at extra-high effort, and verification on Codex at medium. `setup-pstack` starts from it when the session has `delegate_task` and no sheet exists.

```text
feature, refactoring: sonnet @high
bug-fix: sonnet @high
perf-issue: sonnet @high
hillclimb: sonnet @high
judgment and prose: opus @medium
strongest judgment: opus @medium
verifiers: codex/gpt-6.1-sol @medium
how explorer: sonnet @high
how explainer: opus @medium
why investigators: sonnet @high
why synthesizer: opus @medium
reflect tooling: sonnet @high
reflect judgment, divergent, synthesizer: opus @medium
arena runners: sonnet @high, opus @medium
arena cross-judge pool: codex/gpt-6.1-sol @xhigh
swarm workers: sonnet @high
architect runners: sonnet @high, opus @medium
interrogate reviewers: codex/gpt-6.1-sol @xhigh

default effort: session
fast mode: off
session hook: on
```

The orchestrator is the thread's own model, which pstack does not choose. Set it in the T3 composer, with fast mode off there too.

The arena and architect runners write files, so they stay on Claude models. The review panel has one seat, so it has no model diversity of its own. Its diversity comes from the reviewer being a different provider than the author. Add a second entry to `interrogate reviewers` for a second opinion.

Write a preset value only when `orchestrator_capabilities` lists that provider and model in this session. Where it does not, use the stamped default for that role and tell the user.

## Per-skill notes

| Skill | On T3 |
|-------|-------|
| `setup-pstack` | Detect models with `orchestrator_capabilities` as well as the harness's own list. With no sheet, start from the [Preset](#preset) in place of the stamped defaults. Offer each model of a provider that can run a child as `<providerInstanceId>/<model>`, for the roles that edit no source: `interrogate reviewers`, `arena cross-judge pool`, `verifiers`, and the `how`, `why`, and `reflect` roles. The other roles write files, so they take only the harness's own models. Validate a written value against that list. The sheet path and how it loads are the harness's. Ask for the `fast mode` line: `off` or `provider`. Tell the user that a seat on another provider bills that provider's account. |
| `interrogate` | A reviewer seat with a `<providerInstanceId>/<model>` value is a `delegate_task` call. Put the absolute path of the reviewer prompt file and the diff's range in `task`, and state the read-only rule there. |
| `arena` | Candidates write files, so they stay on the harness's own tool. The cross-judge only reads, so a `<providerInstanceId>/<model>` value in the cross-judge pool is a `delegate_task` call. |
| `architect` | The runner panel goes through the **arena** skill, so the same split applies. |
| `how` | Explorers and the explainer only read, so either tool works. |
| `why` | Investigators and the synthesizer only read, so either tool works. A child lists MCP servers from its own provider, not the parent's. |
| `reflect` | The reviewers and the synthesizer only read. Give each the transcript path, because a child on another provider cannot find the parent's transcript. |
| `swarm` | Workers write, so they stay on the harness's own tool, each in its own worktree. |
| `no-comments` | `pstack:comment-sicko` stays on the harness's own tool, because it edits the working tree. |
