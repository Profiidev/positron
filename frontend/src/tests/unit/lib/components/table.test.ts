import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import type { ColumnDef } from '@tanstack/table-core';
import Table from '$lib/components/table/Table.svelte';

interface Row {
  name: string;
}

const columns = (): ColumnDef<Row>[] => [
  {
    accessorKey: 'name',
    cell: ({ row }) => row.original.name,
    header: () => 'Name'
  }
];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const props = (data: unknown) => ({ columns, data }) as any;

describe('Table', () => {
  it('renders the header', () => {
    render(Table, props([{ name: 'Bob' }]));
    expect(screen.getByText('Name')).toBeInTheDocument();
  });

  it('renders rows from a synchronous array', async () => {
    render(Table, props([{ name: 'Bob' }, { name: 'Alice' }]));
    expect(await screen.findByText('Bob')).toBeInTheDocument();
    expect(screen.getByText('Alice')).toBeInTheDocument();
  });

  it('shows "No results." for an empty array', async () => {
    render(Table, props([]));
    expect(await screen.findByText('No results.')).toBeInTheDocument();
  });

  it('shows "Loading..." while a promise is pending', () => {
    render(Table, props(new Promise<Row[]>(() => {})));
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('renders rows once a promise resolves', async () => {
    render(Table, props(Promise.resolve([{ name: 'Async' }])));
    expect(await screen.findByText('Async')).toBeInTheDocument();
  });

  it('shows "No results." when a promise resolves to undefined', async () => {
    render(Table, props(Promise.resolve(undefined)));
    expect(await screen.findByText('No results.')).toBeInTheDocument();
  });
});
