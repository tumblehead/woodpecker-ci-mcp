# woodpecker-ci-mcp

An [MCP](https://modelcontextprotocol.io) server for [Woodpecker CI](https://woodpecker-ci.org) 3.x.
It lets an MCP client such as Claude Code inspect repositories, pipelines and step logs,
trigger and control pipelines, and manage secrets. It communicates over stdio.

## Configuration

| Variable | Required | Description |
| --- | --- | --- |
| `WOODPECKER_SERVER` | yes | Server URL, including the root path if Woodpecker runs under one. A trailing `/api` is ignored. |
| `WOODPECKER_TOKEN` | yes | Personal access token, from `<server>/user/cli-and-api`. |
| `RUST_LOG` | no | Log filter; logs go to stderr. Default `woodpecker_ci_mcp=info`. |

Example Claude Code registration:

```sh
claude mcp add woodpecker \
  -e WOODPECKER_SERVER=https://ci.example.com \
  -e WOODPECKER_TOKEN=... \
  -- /path/to/woodpecker-ci-mcp
```

## Tools

Most tools take a numeric `repo_id`; get it with `lookup_repo` (`owner/name`) or `list_repos`.
Pipelines are addressed by their per-repository `pipeline_number`.

| Area | Tool | Notes |
| --- | --- | --- |
| System | `get_server_version`, `health_check` | |
| | `get_queue_info` | Admin token required |
| Repositories | `list_repos` | Repos the user can access; `all` includes inactive ones, `name` filters |
| | `get_repo`, `lookup_repo` | |
| Pipelines | `list_pipelines` | Filters: `branch`, `event`, `status`, `ref`, `before`, `after`, paging |
| | `get_pipeline` | Includes workflows, steps and errors |
| | `create_pipeline` | Manual pipeline on a branch head, with optional variables |
| | `restart_pipeline`, `cancel_pipeline`, `approve_pipeline`, `decline_pipeline` | |
| Logs | `list_pipeline_steps` | Step IDs, states, exit codes, durations and errors |
| | `get_step_logs` | See below |
| Secrets | `list_secrets`, `create_secret`, `delete_secret` | Repo (`repo_id`), org (`org_id`) or global (neither; writes need admin). `create_secret` requires at least one event. |
| User | `get_current_user`, `get_user_feed` | `latest` gives one pipeline per repo |

Responses are the server's JSON with avatar URLs removed; `list_pipelines` also drops
`changed_files`. API failures come back as tool results flagged `isError`, naming the
request that failed and the server's message.

### Step logs

`get_step_logs` returns numbered lines with ANSI colour codes and carriage-return progress
redraws removed. By default it returns the whole log, or the last 1000 lines if the log is
longer. To choose a window:

- `lines: N`: the last N lines, or the first N with `head: true`
- `offset` / `limit`: page through the log (default page size 100)

## Development

```sh
cargo clippy --all-targets -- -D warnings
cargo test
```

Releases are built by the Woodpecker pipelines in `.woodpecker/` when a `v*` tag is pushed.
They attach per-platform binaries to the Gitea release. Artifact names follow
`woodpecker-ci-mcp-v<version>-<platform>`, which the tumbletrove plugin's bootstrap relies on.
