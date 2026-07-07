# Games PC bootstrap

Runtime composition for SDKWork Games PC browser root:

- `environment.ts` resolves typed runtime config from manifest and Vite env.
- `sessionStore.ts` and `sessionTokenManager.ts` own the global TokenManager bridge.
- `sdkClients.ts` and `iamRuntime.ts` wire appbase IAM and generated games app SDK clients.
- `gamesProviders.ts` configures platform domain services (catalog, room, leaderboard).
- `runtime.ts` is the single composition entrypoint used by `App.tsx`.
