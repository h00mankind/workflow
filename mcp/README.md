# mcp

One definition file per MCP server. A secret is the name of an environment
variable, never its value. The file name is the server name: `mcp/<name>.toml`.

A stdio server:

```toml
command = "npx"
args = ["-y", "some-mcp"]
env = ["GITHUB_TOKEN"]
```

`args` and `env` are optional lists. `env` names variables to pass through.
Do not use `KEY=value` or a table of values.

An HTTP server:

```toml
url = "https://example.com/mcp"
bearer_token_env = "EXAMPLE_TOKEN"
```

`bearer_token_env` is optional. It names the variable holding the bearer token.
Use exactly one of `command` and `url`. Fields from the other form are an error.
Variable names use letters, digits, and underscores, and cannot start with a digit.

Assign servers with `mcp = ["<name>", ...]` in a manifest scope. `status` checks
them and `apply` installs them for Claude Code and Codex. See the root README
for the config paths and ownership rules.
