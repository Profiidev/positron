import { describe, expect, it, vi } from 'vitest';
import type * as Svelte from 'svelte';
import type { SessionInfo } from '$lib/client';
import {
  formatRelativeOptional,
  formatRelativePast
} from '$routes/account/sessions/session-utils';

vi.mock('@profidev/pleiades/components/ui/data-table', () => ({
  renderComponent: (component: unknown, props: unknown) => ({
    component,
    props
  }),
  // The `createColumn` date cells render through a raw snippet; pass it straight
  // Through so the cell returns the HTML string our `svelte` mock produced.
  renderSnippet: (snippet: unknown) => snippet
}));

vi.mock('svelte', async (importOriginal) => {
  const actual = await importOriginal<typeof Svelte>();
  return {
    ...actual,
    // Execute the snippet's `render()` synchronously so a date cell evaluates to
    // A plain HTML string we can assert on.
    createRawSnippet: (fn: () => { render: () => string }) => fn().render()
  };
});

const { columns } = await import('$routes/account/sessions/table.svelte');

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const cellProps = (cols: any[], key: string, original: SessionInfo) =>
  cols.find((c) => c.accessorKey === key).cell({ row: { original } }).props;

// `createColumn` cells read their value via `row.getValue(key)` and return an
// HTML string; this drives that path and returns the rendered markup.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const cellHtml = (cols: any[], key: string, original: SessionInfo): string =>
  cols
    .find((c) => c.accessorKey === key)
    .cell({
      row: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        getValue: (k: string) => (original as any)[k],
        original
      }
    });

const browserSession: SessionInfo = {
  application: 'Chrome 126',
  created_at: new Date('2024-01-01T00:00:00Z'),
  current: true,
  expires_at: new Date('2024-07-01T00:00:00Z'),
  id: 'session-current',
  is_app: false,
  last_used_at: new Date('2024-06-01T00:00:00Z'),
  name: 'MacBook Pro',
  operating_system: 'macOS 15.1',
  refreshed_at: new Date('2024-06-01T00:00:00Z')
};

const appSession: SessionInfo = {
  ...browserSession,
  application: 'Positron iOS 2.4.0',
  current: false,
  id: 'session-app',
  is_app: true,
  name: 'iPhone 15 Pro',
  operating_system: 'iOS 18.1'
};

describe('sessions table columns', () => {
  it('passes the full session to the session cell', () => {
    const cols = columns({ revoke: () => {} });
    const props = cellProps(cols, 'session', browserSession);
    expect(props.session).toEqual(browserSession);
  });

  it('renders browser and app type badges', () => {
    const cols = columns({ revoke: () => {} });
    expect(cellProps(cols, 'is_app', browserSession)).toEqual({ isApp: false });
    expect(cellProps(cols, 'is_app', appSession)).toEqual({ isApp: true });
  });

  it('wires revoke for non-current sessions only', () => {
    const revoke = vi.fn();
    const cols = columns({ revoke });
    const currentProps = cellProps(cols, 'actions', browserSession);
    const otherProps = cellProps(cols, 'actions', appSession);

    expect(currentProps.session.current).toBe(true);
    otherProps.revoke(appSession);
    expect(revoke).toHaveBeenCalledWith(appSession);
  });

  it('passes the expiry as a Date to the expires cell', () => {
    const cols = columns({ revoke: () => {} });
    const { expiresAt } = cellProps(cols, 'expires_at', browserSession);
    expect(expiresAt).toBeInstanceOf(Date);
    expect((expiresAt as Date).toISOString()).toBe(
      new Date(browserSession.expires_at).toISOString()
    );
  });

  it.each(['created_at', 'last_used_at'] as const)(
    'formats the %s column as a relative past label',
    (key) => {
      const cols = columns({ revoke: () => {} });
      const expected = formatRelativePast(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        new Date((browserSession as any)[key])
      );
      expect(cellHtml(cols, key, browserSession)).toContain(expected);
    }
  );

  it('formats refreshed_at via the optional helper', () => {
    const cols = columns({ revoke: () => {} });
    const expected = formatRelativeOptional(
      new Date(browserSession.refreshed_at!)
    );
    expect(cellHtml(cols, 'refreshed_at', browserSession)).toContain(expected);
  });

  it('renders a dash for a missing refreshed_at', () => {
    const cols = columns({ revoke: () => {} });
    const noRefresh: SessionInfo = {
      ...browserSession,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      refreshed_at: undefined as any
    };
    expect(cellHtml(cols, 'refreshed_at', noRefresh)).toContain('-');
  });
});
