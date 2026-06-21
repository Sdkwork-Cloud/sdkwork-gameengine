import type { Location } from 'react-router-dom';

import {
  hasSdkworkGameenginePcIamSession,
  type SdkworkGameenginePcSessionSnapshot,
} from './bootstrap/sessionStore';

export type SdkworkGameenginePcAuthGateDecision =
  | { kind: 'product-route' }
  | { kind: 'auth-route' }
  | { kind: 'redirect'; replace: true; to: string };

const AUTH_BASE_PATH = '/auth';
const AUTH_LOGIN_PATH = '/auth/login';
const DEFAULT_HOME_PATH = '/app/games';

export function hasSdkworkGameenginePcAuthenticatedSession(
  snapshot: SdkworkGameenginePcSessionSnapshot,
): boolean {
  return hasSdkworkGameenginePcIamSession(snapshot);
}

export function buildSdkworkGameenginePcAuthLoginRedirect(
  location: Pick<Location, 'pathname' | 'search' | 'hash'>,
): string {
  const returnPath = `${normalizePathname(location.pathname)}${location.search ?? ''}${location.hash ?? ''}`;
  return `${AUTH_LOGIN_PATH}?redirect=${encodeURIComponent(returnPath)}`;
}

export function sanitizeSdkworkGameenginePcAuthRedirect(value: string | null | undefined): string {
  if (!value) {
    return DEFAULT_HOME_PATH;
  }

  let decoded = value;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    return DEFAULT_HOME_PATH;
  }

  if (!decoded.startsWith('/') || decoded.startsWith('//')) {
    return DEFAULT_HOME_PATH;
  }

  const redirectUrl = new URL(decoded, 'http://sdkwork-games.local');
  if (isAuthRoute(redirectUrl.pathname)) {
    return DEFAULT_HOME_PATH;
  }

  return `${redirectUrl.pathname}${redirectUrl.search}${redirectUrl.hash}`;
}

export function resolveSdkworkGameenginePcAuthGateDecision({
  hasSession,
  homePath = DEFAULT_HOME_PATH,
  location,
}: {
  hasSession: boolean;
  homePath?: string;
  location: Pick<Location, 'pathname' | 'search' | 'hash'>;
}): SdkworkGameenginePcAuthGateDecision {
  const pathname = normalizePathname(location.pathname);
  if (isAuthRoute(pathname)) {
    if (!hasSession) {
      return { kind: 'auth-route' };
    }

    const redirect = new URLSearchParams((location.search ?? '').replace(/^\?/u, '')).get(
      'redirect',
    );
    return {
      kind: 'redirect',
      replace: true,
      to: sanitizeSdkworkGameenginePcAuthRedirect(redirect) || normalizePathname(homePath),
    };
  }

  if (!hasSession) {
    return {
      kind: 'redirect',
      replace: true,
      to: buildSdkworkGameenginePcAuthLoginRedirect(location),
    };
  }

  return { kind: 'product-route' };
}

function isAuthRoute(pathname: string): boolean {
  return pathname === AUTH_BASE_PATH || pathname.startsWith(`${AUTH_BASE_PATH}/`);
}

function normalizePathname(pathname: string): string {
  const normalized = pathname.trim();
  if (!normalized) {
    return DEFAULT_HOME_PATH;
  }
  return normalized.startsWith('/') ? normalized : `/${normalized}`;
}
