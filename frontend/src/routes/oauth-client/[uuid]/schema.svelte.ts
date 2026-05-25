import type { OAuthClientInfo } from '$lib/client';
import type { FormValue } from '@profidev/pleiades/components/form/types';
import { z } from 'zod';

export const clientSettings = z.object({
  additional_redirect_uris: z.array(z.url()).default([]),
  group_access: z.array(z.string()).default([]),
  name: z.string().min(1, 'Name is required').default(''),
  redirect_uri: z.url().default(''),
  scope: z.array(z.string()).default([]),
  user_access: z.array(z.string()).default([])
});

export const formatData = (
  client: OAuthClientInfo
): FormValue<typeof clientSettings> => {
  // oxlint-disable-next-line no-unsafe-type-assertion
  const scope = client.default_scope as unknown as string;
  return {
    ...client,
    group_access: client.group_access.map((group) => group.uuid),
    // oxlint-disable-next-line no-unsafe-type-assertion
    scope: scope ? scope.split(' ') : [],
    user_access: client.user_access.map((user) => user.id)
  };
};
