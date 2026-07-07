# Tools

Generation and materialization utilities for APIs, SDKs, and database contracts.

- `games_openapi_export.mjs`: export authored OpenAPI authorities into `generated/openapi`; use `--check` to fail on stale generated exports without rewriting them
- `games_route_manifest_check.mjs`: validate route manifest metadata
- `games_sdk_generate.mjs`: SDK generation and alignment checks
- `materialize_games_database_contract.mjs`: regenerate database contract artifacts

Do not hand-edit generated output under `sdks/` or `database/ddl/generated/`.
