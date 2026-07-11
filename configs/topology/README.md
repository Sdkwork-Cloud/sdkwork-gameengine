# Deployment Topology Env Templates

`sdkwork-gameengine` uses SDKWork v4 topology profile ids:
`<deploymentProfile>.<environment>`. Process layout is an internal orchestration
detail and is not encoded in profile ids, env keys, scripts, or client bootstrap.

| File | deploymentProfile | environment | runtimeTarget |
| --- | --- | --- | --- |
| `standalone.development.env` | standalone | development | server |
| `standalone.production.env` | standalone | production | server |
| `cloud.development.env` | cloud | development | server |
| `cloud.production.env` | cloud | production | container |

The application code is `games`, so runtime topology and database keys use the
`SDKWORK_GAMES_*` prefix. Browser-visible mirrors use `VITE_SDKWORK_GAMES_*` and
contain only non-secret public runtime values.

Production URL ownership:

- Application public ingress: `https://gameengine.sdkwork.com`
- PC frontend origin: `https://games.sdkwork.com`
- Platform API gateway: `https://api.sdkwork.com`

Development profiles may use local PostgreSQL password env values for local
bootstrap. Production profiles use structured PostgreSQL fields with
`SDKWORK_GAMES_DATABASE_PASSWORD_FILE` and `SDKWORK_GAMES_DATABASE_AUTO_MIGRATE=false`.

See `../../../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md` and
`../../../sdkwork-specs/APP_RUNTIME_TOPOLOGY_NAMING.md`.
