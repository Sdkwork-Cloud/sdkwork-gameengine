import { Navigate, Route, Routes, useLocation } from 'react-router-dom';
import { GamesAppShell } from 'sdkwork-gameengine-pc-shell';

import {
  buildSdkworkGameenginePcAuthLoginRedirect,
  hasSdkworkGameenginePcAuthenticatedSession,
} from './authGateLogic';
import type { SdkworkGameenginePcRuntime } from './bootstrap/runtime';

export function AppRoutes({ runtime }: { runtime: SdkworkGameenginePcRuntime }) {
  return (
    <Routes>
      <Route
        element={
          <ProtectedRoute runtime={runtime}>
            <GamesAppShell
              onLogout={async () => {
                await runtime.iamRuntime.service.auth.sessions.current.delete();
              }}
            />
          </ProtectedRoute>
        }
        path="/app/games/*"
      />
      <Route element={<Navigate replace to="/app/games" />} path="/" />
      <Route element={<Navigate replace to="/app/games" />} path="*" />
    </Routes>
  );
}

function ProtectedRoute({
  children,
  runtime,
}: {
  children: React.ReactNode;
  runtime: SdkworkGameenginePcRuntime;
}) {
  const location = useLocation();
  const snapshot = runtime.session.getSnapshot();
  if (!hasSdkworkGameenginePcAuthenticatedSession(snapshot)) {
    return <Navigate replace to={buildSdkworkGameenginePcAuthLoginRedirect(location)} />;
  }
  return <>{children}</>;
}
