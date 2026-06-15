import { defineConfig } from '@hey-api/openapi-ts';
import { defineConfig as msw } from './src/tests/mocks/msw-plugin';

export default defineConfig({
  input: 'http://localhost:5175/openapi.json',
  logs: './build',
  output: {
    path: 'src/lib/client',
    postProcess: ['oxfmt']
  },
  plugins: [
    {
      enums: true,
      name: '@hey-api/typescript'
    },
    {
      name: '@hey-api/sdk'
    },
    {
      bigInt: false,
      name: '@hey-api/transformers'
    },
    {
      baseUrl: '',
      name: '@hey-api/client-fetch',
      runtimeConfigPath: '$lib/backend/config'
    },
    msw()
  ]
});
