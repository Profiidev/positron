// Tauri doesn't have a Node.js server to do proper SSR
// So we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps

import { redirect } from '@sveltejs/kit';
import { setupStatus } from '../lib/commands/setup.svelte';
import type { LayoutLoad } from './$types';

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;

export const load: LayoutLoad = async ({ url }) => {
  const status = await setupStatus();

  if ((!status || !status.url_set) && url.pathname !== '/setup') {
    redirect(302, '/setup');
  }

  return { isSetup: status?.url_set ?? false };
};
