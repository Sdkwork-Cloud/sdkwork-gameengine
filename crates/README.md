# Crates

Rust workspace members for the **sdkwork-gameengine** platform (shared across game applications):

| Crate | Responsibility |
| --- | --- |
| `sdkwork-gameengine-standalone-gateway` | HTTP server with `sdkwork-web-framework` + route assembly |
| `sdkwork-routes-games-support` | Shared `SdkWorkApiResponse`, ProblemDetail, trace correlation |
| `sdkwork-routes-health-app-api` | Health/ready routes |
| `sdkwork-routes-catalog-app-api` | Catalog list/detail app-api routes |
| `sdkwork-routes-catalog-backend-api` | Catalog backend-admin routes |
| `sdkwork-routes-leaderboard-app-api` | Leaderboard list + `me` app-api routes |
| `sdkwork-routes-room-app-api` | Game room list app-api routes |
| `sdkwork-games-database-host` | `sdkwork-database` lifecycle bootstrap |
| `sdkwork-game-catalog-service` | Catalog domain service |
| `sdkwork-game-catalog-repository-sqlx` | Catalog SQLx persistence and feature-gated memory test store |
| `sdkwork-game-mode-service` | Game mode domain service |
| `sdkwork-game-mode-repository-sqlx` | Game mode SQLx persistence and feature-gated memory test store |
| `sdkwork-game-rules-service` | Game ruleset domain service |
| `sdkwork-game-rules-repository-sqlx` | Game ruleset SQLx persistence and feature-gated memory test store |
| `sdkwork-game-leaderboard-service` | Leaderboard domain service |
| `sdkwork-game-leaderboard-repository-sqlx` | Leaderboard SQLx persistence and feature-gated memory test store |
| `sdkwork-game-room-service` | Game room domain service |
| `sdkwork-game-room-repository-sqlx` | Game room SQLx persistence and feature-gated memory test store |

Consumer applications (for example `sdkwork-games`) depend on these crates via `../sdkwork-gameengine/crates/*` path dependencies and compose app-specific gateways on top.
