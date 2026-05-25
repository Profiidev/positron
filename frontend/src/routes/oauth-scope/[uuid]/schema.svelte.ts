import type { OAuthScopeInfo } from '$lib/client';
import type { FormValue } from '@profidev/pleiades/components/form/types';
import { z } from 'zod';

export const scopeSettings = z.object({
  name: z.string().min(1, 'Name is required').default(''),
  policies: z.array(z.string()).default([]),
  scope: z.string().min(1, 'Scope is required').default('')
});

export const formatData = (
  scope: OAuthScopeInfo
): FormValue<typeof scopeSettings> => ({
  ...scope,
  policies: scope.policies.map((user) => user.uuid)
});
