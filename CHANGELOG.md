# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed
- Upgraded rmcp 1.3 to 3.4, base64 0.22 to 0.23, and the other dependencies. Minimum Rust version is now 1.88.
- Responses are now the Woodpecker server's own JSON (minus avatar URLs) instead of
  fixed models that no longer matched Woodpecker 3.x. Pipeline timestamps and `errors`,
  step errors, and repository fields such as `forge_url`, `trusted`, `require_approval` and
  `allow_pr` are no longer dropped. Output is compact JSON.
- `list_repos` now lists the repositories the user can access (`/user/repos`), with
  `all` and `name` filters, instead of the admin-only `/repos`. The `active`, `page` and
  `per_page` parameters are removed.
- `create_secret` now requires `events`, since Woodpecker rejects secrets without them, and
  accepts `images` and `note`.
- `get_step_logs` returns the last 1000 lines by default when a log is longer than that.
  It strips ANSI escape codes and carriage-return progress redraws, and decodes invalid
  UTF-8 lossily instead of falling back to base64.
- `list_pipeline_steps` also shows pipeline errors and warnings, step types, durations
  and step errors.
- API failures are returned as tool results with `isError` set, naming the request that
  failed, instead of as JSON-RPC errors.
- The server now reports itself as `woodpecker-ci-mcp` with its own version. It previously
  reported as `rmcp`.

### Added
- `list_pipelines` filters `ref`, `before` and `after`. `get_user_feed` option `latest`.
- Tool annotations (read-only, destructive, idempotent hints).
- `WOODPECKER_SERVER` may include a root path. A trailing `/api` is ignored, and the token is trimmed.

### Fixed
- `get_queue_info` failed whenever the queue was not empty, because task IDs are strings.
- `create_pipeline`, `restart_pipeline` and `approve_pipeline` failed when Woodpecker
  filtered the pipeline out (HTTP 204). They now explain why no pipeline was created.
- Truncating an error body could panic in the middle of a multi-byte UTF-8 character.
- Request bodies, including secret values, were written to the debug log.
- Secret names and repository names are now URL-encoded in request paths.
- Passing both `repo_id` and `org_id` to a secret tool is now an error instead of
  silently using the repository.

### Removed
- Unused model types (agents, cron jobs, registries, organizations and others).

## [0.2.0] - 2026-03-31

### Added
- `get_step_logs` pagination (`offset`/`limit`) and head/tail (`lines`/`head`).
- MIT license; `Cargo.lock` is now tracked.

### Changed
- Upgraded dependencies; Rust edition 2024.

## [0.1.0] - 2026-03-17

### Added
- First release: tools for the system, repositories, pipelines, step logs, secrets and
  the user, over stdio.
- `list_pipeline_steps` tool.
- Woodpecker CI pipelines that build release binaries for Linux, macOS (universal) and Windows.

### Fixed
- `get_step_logs` handles `null` responses and decodes the base64 `data` field.

[Unreleased]: http://10.100.36.15:3000/soren-n/woodpecker-ci-mcp/compare/v0.2.0...main
[0.2.0]: http://10.100.36.15:3000/soren-n/woodpecker-ci-mcp/compare/v0.1.0...v0.2.0
[0.1.0]: http://10.100.36.15:3000/soren-n/woodpecker-ci-mcp/releases/tag/v0.1.0
