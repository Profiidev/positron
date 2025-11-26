import { ResponseType, post } from 'positron-components/backend';

export const email_start_change = async (new_email: string) => {
  return await post<undefined>(
    '/backend/email/manage/start_change',
    ResponseType.None,
    {
      new_email
    }
  );
};

export const email_finish_change = async (
  old_code: string,
  new_code: string
) => {
  return await post<undefined>(
    '/backend/email/manage/finish_change',
    ResponseType.None,
    {
      old_code,
      new_code
    }
  );
};
