import {
  type DefaultBodyType,
  type HttpResponseResolver,
  type PathParams,
  http
} from 'msw';

/**
 * Runtime support for the generated `msw.gen.ts` factories.
 *
 * The hey-api MSW plugin (see `src/lib/msw-plugin`) emits one
 * `<operation>MswHandler` per endpoint that calls `wrapMswHandler` with the
 * operation's path, method and the request/response/error types taken straight
 * from the OpenAPI schema. That keeps the mocks in sync with the API contract:
 * regenerating the client (`npm run api`) regenerates the handlers too.
 *
 * Adapted from the community plugin shared in
 * https://github.com/hey-api/openapi-ts/issues/1486.
 */

interface RequestOptions {
  body?: unknown;
  path?: Record<string, string | readonly string[]>;
}

/**
 * Excludes catch-all index signatures like `{ [key: string]: unknown }` so they
 * don't pollute the resolver's response type.
 */
type ExcludeCatchAll<T> =
  T extends Record<string, unknown> ? (string extends keyof T ? never : T) : T;

/** Transforms OpenAPI-style placeholders `{id}` into MSW-style `:id`. */
const convertOpenApiUrlToMsw = (url: string): string =>
  url.replace(/{(?<name>\w+)}/g, (_match, name) => `:${name}`);

export const wrapMswHandler = <
  TResponse = unknown,
  TError = unknown,
  TData extends RequestOptions = RequestOptions
>(
  path: string,
  method: 'head' | 'get' | 'post' | 'put' | 'delete' | 'patch' | 'options',
  getConfig: () => { baseUrl?: string }
) => {
  type Params = TData extends { path: infer P }
    ? P extends PathParams<keyof P>
      ? P
      : never
    : never;

  type RequestBodyType = TData extends { body: infer B }
    ? B extends DefaultBodyType
      ? B
      : never
    : never;

  type ResponseBodyType = [TResponse | ExcludeCatchAll<TError>] extends [
    DefaultBodyType
  ]
    ? TResponse | ExcludeCatchAll<TError>
    : never;

  return (
    resolver: HttpResponseResolver<Params, RequestBodyType, ResponseBodyType>
  ) =>
    http[method](
      (getConfig().baseUrl ?? '') + convertOpenApiUrlToMsw(path),
      resolver
    );
};
