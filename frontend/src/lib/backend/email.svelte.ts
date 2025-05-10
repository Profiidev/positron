import { ContentType, ResponseType } from 'positron-components/backend';
import { post } from './util.svelte';

export const email_start_change = async (new_email: string) => {
  return await post<undefined>(
    '/email/manage/start_change',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      new_email
    })
  );
};

export const email_finish_change = async (
  old_code: string,
  new_code: string
) => {
  return await post<undefined>(
    '/email/manage/finish_change',
    ResponseType.None,
    ContentType.Json,
    JSON.stringify({
      old_code,
      new_code
    })
  );
};
