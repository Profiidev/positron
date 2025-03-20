import { UpdateType } from '../ws/types.svelte';
import { create_updater } from '../ws/updater.svelte';
import { passkey_list as list_passkey } from './passkey.svelte';
import type { Passkey } from './types.svelte';

export const passkey_list = create_updater<Passkey[]>(
  UpdateType.Passkey,
  list_passkey
);
