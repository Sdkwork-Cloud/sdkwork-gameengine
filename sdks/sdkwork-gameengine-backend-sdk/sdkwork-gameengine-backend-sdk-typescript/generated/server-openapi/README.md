# sdkwork-gameengine-backend-sdk

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
npm install @sdkwork-internal/gameengine-backend-sdk-generated
# or
yarn add @sdkwork-internal/gameengine-backend-sdk-generated
# or
pnpm add @sdkwork-internal/gameengine-backend-sdk-generated
```

## Quick Start

```typescript
import { SdkworkGameengineBackendClient } from '@sdkwork-internal/gameengine-backend-sdk-generated';

const client = new SdkworkGameengineBackendClient({
  baseUrl: '/backend/v3/api',
  timeout: 30000,
});

// Authentication
client.setAuthToken('your-auth-token');
client.setAccessToken('your-access-token');

// Use the SDK
const params = {
  page: 1,
  page_size: 2,
  status: 'status',
};
const result = await client.games.backend.games.catalog.list(params);
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```typescript
import { SdkworkGameengineBackendClient } from '@sdkwork-internal/gameengine-backend-sdk-generated';

const client = new SdkworkGameengineBackendClient({
  baseUrl: '/backend/v3/api',
  timeout: 30000, // Request timeout in ms
  headers: {      // Custom headers
    'X-Custom-Header': 'value',
  },
});
```

## API Modules

- `client.games` - games API
- `client.rooms` - rooms API

## Usage Examples

### games

```typescript
// GET /backend/v3/api/games
const params = {
  page: 1,
  page_size: 2,
  status: 'status',
};
const result = await client.games.backend.games.catalog.list(params);
```

### rooms

```typescript
// GET /backend/v3/api/games/rooms
const params = {
  game_id: 'game_id',
  status: 'open',
  page: 3,
  page_size: 4,
};
const result = await client.rooms.backend.games.rooms.list(params);
```

## Error Handling

```typescript
import { SdkworkGameengineBackendClient, NetworkError, TimeoutError, AuthenticationError } from '@sdkwork-internal/gameengine-backend-sdk-generated';

try {
  const params = {
    page: 1,
    page_size: 2,
    status: 'status',
  };
  const result = await client.games.backend.games.catalog.list(params);
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Authentication failed:', error.message);
  } else if (error instanceof TimeoutError) {
    console.error('Request timed out:', error.message);
  } else if (error instanceof NetworkError) {
    console.error('Network error:', error.message);
  } else {
    throw error;
  }
}
```

## Publishing

This SDK includes cross-platform publish scripts in `bin/`:
- `bin/publish-core.mjs`
- `bin/publish.sh`
- `bin/publish.ps1`

### Check

```bash
./bin/publish.sh --action check
```

### Publish

```bash
./bin/publish.sh --action publish --channel release
```

```powershell
.\bin\publish.ps1 --action publish --channel test --dry-run
```

> Configure npm registry credentials before release publish.

## License

MIT

## Regeneration Contract

- HTTP/OpenAPI generator-owned files are tracked in `.sdkwork/sdkwork-generator-manifest.json`.
- HTTP/OpenAPI generation also writes `.sdkwork/sdkwork-generator-changes.json` so automation can inspect created, updated, deleted, unchanged, scaffolded, and backed-up files plus the classified impact areas, verification plan, and execution decision for the latest generation.
- HTTP/OpenAPI apply mode also writes `.sdkwork/sdkwork-generator-report.json` with the full execution report, including `schemaVersion`, `generator`, stable artifact paths, and the execution handoff commands that match CLI `--json` output.
- CLI JSON output also includes an execution handoff with concrete next commands, including reviewed apply commands for dry-run flows.
- Put HTTP/OpenAPI hand-written wrappers, adapters, and orchestration in `custom/`.
- Files scaffolded under `custom/` are created once and preserved across HTTP/OpenAPI regenerations.
- If an HTTP/OpenAPI generated-owned file was modified locally, its previous content is copied to `.sdkwork/manual-backups/` before overwrite or removal.
- RPC SDK source workspaces use convention-first evidence by default: RPC SDK family naming, language workspace naming, `rpc/*.manifest.json`, proto source references, generated client source, and native package manifests.
- Use `sdkgen inspect --protocol rpc` to verify RPC convention evidence. Request persisted generator evidence only with `--emit-control-plane` for release, CI, audit, or migration workflows; evidence paths are derived by generator convention.
