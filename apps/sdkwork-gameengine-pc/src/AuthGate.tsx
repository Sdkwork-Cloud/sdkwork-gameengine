import { lazy, type ReactNode, useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { SdkworkIamAuthRoutes } from '@sdkwork/auth-pc-react';

import {
  resolveSdkworkGameenginePcAuthAppearance,
  resolveSdkworkGameenginePcAuthLocale,
  resolveSdkworkGameenginePcAuthRuntimeConfig,
} from './bootstrap/authConfig';
import type { SdkworkGameenginePcRuntime } from './bootstrap/runtime';
import {
  hasSdkworkGameenginePcAuthenticatedSession,
  resolveSdkworkGameenginePcAuthGateDecision,
} from './authGateLogic';

export interface AuthGateProps {
  children: ReactNode;
  runtime: SdkworkGameenginePcRuntime;
}

export function AuthGate({ children, runtime }: AuthGateProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [snapshot, setSnapshot] = useState(() => runtime.session.getSnapshot());

  useEffect(() => runtime.session.subscribe(setSnapshot), [runtime.session]);

  const decision = useMemo(
    () =>
      resolveSdkworkGameenginePcAuthGateDecision({
        hasSession: hasSdkworkGameenginePcAuthenticatedSession(snapshot),
        homePath: '/app/games',
        location,
      }),
    [location, snapshot],
  );

  useEffect(() => {
    if (decision.kind !== 'redirect') {
      return;
    }
    navigate(decision.to, { replace: true });
  }, [decision, navigate]);

  if (decision.kind === 'redirect') {
    return null;
  }

  if (decision.kind === 'auth-route') {
    const authProps = {
      appearance: resolveSdkworkGameenginePcAuthAppearance(),
      basePath: '/auth',
      getRuntime: () => runtime.iamRuntime,
      homePath: '/app/games',
      locale: resolveSdkworkGameenginePcAuthLocale(runtime.config.i18n.defaultLocale),
      runtimeConfig: resolveSdkworkGameenginePcAuthRuntimeConfig(),
      viewportMode: 'flow' as const,
    };

    return (
      <SdkworkIamAuthRoutes
        {...(authProps as unknown as Parameters<typeof SdkworkIamAuthRoutes>[0])}
      />
    );
  }

  return <>{children}</>;
}
