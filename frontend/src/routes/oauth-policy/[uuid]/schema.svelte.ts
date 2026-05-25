import type { OAuthScopeInfo } from '$lib/client';
import type { FormValue } from '@profidev/pleiades/components/form/types';
import { z } from 'zod';

export const policySettings = z.object({
  claim: z.string().min(1, 'Claim is required').default(''),
  default: z.string().min(1, 'Default is required').default(''),
  name: z.string().min(1, 'Name is required').default('')
});

export const formatData = (
  scope: OAuthScopeInfo
): FormValue<typeof policySettings> => ({
  ...scope
});
