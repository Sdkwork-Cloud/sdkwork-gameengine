import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  build: {
    lib: {
      entry: path.resolve(__dirname, 'src/index.ts'),
      name: 'SdkworkGameI18n',
      fileName: 'index',
      formats: ['es']
    },
    rollupOptions: {
      external: ['react', 'react-dom', 'i18next', 'react-i18next']
    }
  }
});
