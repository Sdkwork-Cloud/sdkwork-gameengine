import { BrowserRouter, Route, Routes } from 'react-router-dom';

import { AppRoutes } from './AppRoutes';
import { AuthGate } from './AuthGate';
import { createSdkworkGameenginePcRuntime } from './bootstrap/runtime';

const runtime = createSdkworkGameenginePcRuntime();

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route
          element={
            <AuthGate runtime={runtime}>
              <AppRoutes runtime={runtime} />
            </AuthGate>
          }
          path="/*"
        />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
