import { describe, expect, it, vi } from 'vitest';
import type { SessionInfo } from '$lib/client';

vi.mock('@profidev/pleiades/components/ui/data-table', () => ({
  renderComponent: (component: unknown, props: unknown) => ({
    component,
    props
  })
}));

const { columns } = await import('$routes/account/sessions/table.svelte');

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const cellProps = (cols: any[], key: string, original: SessionInfo) =>
  cols.find((c) => c.accessorKey === key).cell({ row: { original } }).props;

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
});
