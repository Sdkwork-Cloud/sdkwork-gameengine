export interface SdkworkGameenginePcSessionSnapshot {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
  sessionId?: string;
  context?: {
    tenantId?: string;
    userId?: string;
    organizationId?: string;
    sessionId?: string;
    appId?: string;
    environment?: string;
    deploymentMode?: string;
  };
  updatedAt?: string;
}

export interface SdkworkGameenginePcSessionStorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export interface SdkworkGameenginePcSessionStore {
  clearSession(): void;
  getSnapshot(): SdkworkGameenginePcSessionSnapshot;
  refreshSession(): SdkworkGameenginePcSessionSnapshot;
  setSession(nextSession: SdkworkGameenginePcSessionSnapshot): void;
  subscribe(listener: (snapshot: SdkworkGameenginePcSessionSnapshot) => void): () => void;
}

export const SDKWORK_GAMEENGINE_PC_SESSION_STORAGE_KEY = 'sdkwork-gameengine-pc-session';

function readInitialSession(
  storage: SdkworkGameenginePcSessionStorageLike | undefined,
  storageKey: string,
): SdkworkGameenginePcSessionSnapshot {
  if (!storage) {
    return {};
  }

  try {
    const raw = storage.getItem(storageKey);
    return raw ? (JSON.parse(raw) as SdkworkGameenginePcSessionSnapshot) : {};
  } catch {
    return {};
  }
}

export function createSdkworkGameenginePcSessionStore(
  storage?: SdkworkGameenginePcSessionStorageLike,
  storageKey = SDKWORK_GAMEENGINE_PC_SESSION_STORAGE_KEY,
): SdkworkGameenginePcSessionStore {
  let snapshot = readInitialSession(storage, storageKey);
  const listeners = new Set<(nextSnapshot: SdkworkGameenginePcSessionSnapshot) => void>();

  const emit = () => {
    for (const listener of listeners) {
      listener(snapshot);
    }
  };

  const persist = () => {
    if (!storage) {
      return;
    }

    if (!snapshot.authToken && !snapshot.accessToken && !snapshot.refreshToken) {
      storage.removeItem(storageKey);
      return;
    }

    storage.setItem(storageKey, JSON.stringify(snapshot));
  };

  return {
    clearSession() {
      snapshot = {};
      persist();
      emit();
    },
    getSnapshot() {
      return snapshot;
    },
    refreshSession() {
      snapshot = readInitialSession(storage, storageKey);
      emit();
      return snapshot;
    },
    setSession(nextSession) {
      snapshot = {
        ...nextSession,
        updatedAt: new Date().toISOString(),
      };
      persist();
      emit();
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

export function hasSdkworkGameenginePcIamSession(snapshot: SdkworkGameenginePcSessionSnapshot): boolean {
  return Boolean(snapshot.authToken && snapshot.accessToken && snapshot.context?.tenantId);
}
