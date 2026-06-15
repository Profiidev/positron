import { definePluginConfig } from '@hey-api/openapi-ts';
import { handler } from './plugin';
import type { MswPlugin } from './types';

export const defaultConfig: MswPlugin['Config'] = {
  config: {},
  // Ensure the response/request types this plugin references are generated.
  dependencies: ['@hey-api/typescript'],
  handler,
  name: 'msw',
  tags: ['mocker']
};

/**
 * Register in `openapi-ts.config.ts`:
 *
 *   import { defineConfig as msw } from './openapi-ts/msw-plugin';
 *   plugins: [..., msw()]
 */
export const defineConfig = definePluginConfig(defaultConfig);
