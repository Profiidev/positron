import { $, applyNaming } from '@hey-api/openapi-ts';
import type { MswPlugin } from './types';

/**
 * Emits `msw.gen.ts` with a typed handler factory per operation, e.g.
 *
 *   export const isSetupMswHandler = wrapMswHandler<
 *     IsSetupResponse,
 *     IsSetupError,
 *     IsSetupData
 *   >('/api/setup', 'get', client.getConfig);
 *
 * Used in tests as:
 *
 *   server.use(isSetupMswHandler(() => HttpResponse.json({ ... })));
 */
export const handler: MswPlugin['Handler'] = ({ plugin }) => {
  // `wrapMswHandler` ships in the repo (not generated) so it can be edited.
  const wrapMswHandler = plugin.symbol('wrapMswHandler', {
    external: '$mocks/msw-runtime'
  });

  // The generated fetch client, used to read the configured `baseUrl`.
  const client = plugin.symbol('client', {
    external: '$mocks/msw-runtime'
  });

  // oxlint-disable-next-line no-array-for-each
  plugin.forEach('operation', (event) => {
    const { operation } = event;

    // Pull the request/response/error types the @hey-api/typescript plugin
    // Already generated for this operation so the factory stays type-safe.
    const responseType = plugin.querySymbol({
      category: 'type',
      resource: 'operation',
      resourceId: operation.id,
      role: 'response'
    });
    const errorType = plugin.querySymbol({
      category: 'type',
      resource: 'operation',
      resourceId: operation.id,
      role: 'error'
    });
    const dataType = plugin.querySymbol({
      category: 'type',
      resource: 'operation',
      resourceId: operation.id,
      role: 'data'
    });

    const factory = plugin.symbol(
      applyNaming(operation.id, { name: '{{name}}MswHandler' }),
      { exported: true }
    );

    const statement = $.const(factory)
      .export()
      .assign(
        $(wrapMswHandler)
          .call(
            $.literal(operation.path),
            $.literal(operation.method),
            $(client).attr('getConfig')
          )
          .generics(
            $.type(responseType ?? 'never'),
            $.type(errorType ?? 'never'),
            $.type(dataType ?? 'never')
          )
      );

    plugin.node(statement);
  });
};
