# Runtime Config Templates

Safe checked-in examples for `sdkwork-games`. Local overrides such as `.env.local`,
`.env.postgres`, and `*.local.toml` must stay gitignored.

| File | Purpose |
| --- | --- |
| `games.database.example.toml` | Structured production PostgreSQL template using `SDKWORK_GAMES_DATABASE_*` fields and `password_file`; no committed database secrets. |
| `topology/` | SDKWork v4 deployment profile env templates using `<deploymentProfile>.<environment>.env`. |

Runtime config follows `../../sdkwork-specs/CONFIG_SPEC.md`,
`../../sdkwork-specs/ENVIRONMENT_SPEC.md`, and
`../../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`.
