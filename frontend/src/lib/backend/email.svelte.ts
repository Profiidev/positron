import { ResponseType, post } from 'positron-components/backend';

export const email_start_change = async (new_email: string) => {
  return await post('/backend/email/manage/start_change', {
    body: {
      new_email
    }
  });
};

export const email_finish_change = async (
  old_code: string,
  new_code: string
) => {
  return await post('/backend/email/manage/finish_change', {
    body: {
      old_code,
      new_code
    }
  });
};
