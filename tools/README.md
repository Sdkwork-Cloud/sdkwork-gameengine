# Tools

Generation and materialization utilities for APIs, SDKs, and database contracts.

- `games_openapi_export.mjs`: materialize OpenAPI authorities from route manifests
- `games_route_manifest_check.mjs`: validate route manifest metadata
- `games_sdk_generate.mjs`: SDK generation and alignment checks
- `materialize_games_database_contract.mjs`: regenerate database contract artifacts

Do not hand-edit generated output under `sdks/` or `database/ddl/generated/`.
