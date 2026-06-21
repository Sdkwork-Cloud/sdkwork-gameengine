# SDKWork Games PC

PC browser application root for the SDKWork Games platform.

## Structure

- `packages/sdkwork-gameengine-pc-core` â€?shared runtime (stores, SDK wiring)
- `packages/sdkwork-gameengine-pc-commons` â€?shared UI components and hooks
- `packages/sdkwork-gameengine-pc-<capability>` â€?user-facing capability modules

Package names follow `APP_PC_ARCHITECTURE_SPEC.md`: `sdkwork-gameengine-pc-*` for app/user modules.

## Run locally

From the repository root:

```bash
pnpm dev
```

From this directory:

```bash
pnpm install
pnpm dev
```

The dev server listens on port 3000.

## Standards

- Architecture: `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- UI: `../../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`
- Manifest: `sdkwork.app.config.json`
