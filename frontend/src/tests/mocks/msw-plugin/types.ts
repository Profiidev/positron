import type { DefinePlugin, Plugin } from '@hey-api/openapi-ts';

/**
 * Custom plugin name. `Plugin.Name` only accepts names already registered in
 * `PluginConfigMap`, so for a local plugin we type `name` as a literal instead
 * (it still satisfies `AnyPluginName`).
 */
interface Name {
  name: 'msw';
}

/**
 * User-facing config. The plugin takes no options yet — it generates one MSW
 * handler factory per operation.
 */
export type UserConfig = Name & Plugin.Hooks & Plugin.UserExports;

/** Resolved config (after defaults are applied). */
export type Config = Name & Plugin.Hooks & Plugin.Exports;

export type MswPlugin = DefinePlugin<UserConfig, Config>;
