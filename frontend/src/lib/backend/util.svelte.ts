import { PUBLIC_BACKEND_URL, PUBLIC_IS_APP } from '$env/static/public';
import { wait_for } from 'positron-components/util';
import {
  ContentType,
  RequestError,
  ResponseType
} from 'positron-components/backend';

let fetchFn: typeof fetch | undefined = undefined;
const set_fetch = async () => {
  if (PUBLIC_IS_APP === 'true') {
    fetchFn = (await import('@tauri-apps/plugin-http')).fetch;
  } else {
    fetchFn = fetch;
  }
};
set_fetch();

export const post = async <T>(
  path: string,
  res_type: ResponseType,
  content_type: ContentType,
  body: any,
  signal?: AbortSignal
): Promise<T | RequestError> => {
  return await request(path, 'POST', res_type, content_type, body, signal);
};

export const get = async <T>(
  path: string,
  res_type: ResponseType,
  signal?: AbortSignal
): Promise<T | RequestError> => {
  return await request(path, 'GET', res_type, undefined, undefined, signal);
};

const request = async <T>(
  path: string,
  method: string,
  res_type: ResponseType,
  content_type?: ContentType,
  body?: any,
  signal?: AbortSignal
): Promise<T | RequestError> => {
  let headers;
  if (content_type && body) {
    headers = {
      'Content-Type': content_type
    };
  }

  try {
    await wait_for(() => fetchFn !== undefined);

    let res = await fetchFn!(`/backend${path}`, {
      method,
      headers,
      body,
      signal
    });

    switch (res.status) {
      case 200:
        break;
      case 401:
        return RequestError.Unauthorized;
      case 409:
        return RequestError.Conflict;
      case 410:
        return RequestError.Gone;
      default:
        return RequestError.Other;
    }

    if (PUBLIC_IS_APP === 'true') {
      let cookie = res.headers.get('Set-Cookie');
      if (cookie) {
        let invoke = (await import('@tauri-apps/api/core')).invoke;
        invoke('set_cookie', {
          cookie,
          url: BASE_URL
        });
      }
    }

    switch (res_type) {
      case ResponseType.Json:
        let json = await res.json();
        return json as T;
      case ResponseType.Text:
        let text = await res.text();
        return text as T;
      case ResponseType.None:
        return undefined as T;
    }
  } catch (e) {
    console.error('Request error', e);
    return RequestError.Other;
  }
};

let url = new URL(PUBLIC_BACKEND_URL);
url.pathname = '/';

export const BASE_URL = url.toString();
