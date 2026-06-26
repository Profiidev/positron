import { describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
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

interface SearchRow {
  name: string;
  email: string;
  count: number;
  groups: string[];
}

const searchColumnsDef = (): ColumnDef<SearchRow>[] => [
  {
    accessorKey: 'name',
    cell: ({ row }) => row.original.name,
    header: () => 'Name'
  },
  {
    accessorKey: 'email',
    cell: ({ row }) => row.original.email,
    header: () => 'Email'
  }
];

const searchData: SearchRow[] = [
  { count: 1, email: 'bob@example.com', groups: ['admin'], name: 'Bob' },
  { count: 22, email: 'alice@test.org', groups: ['user'], name: 'Alice' }
];

const searchProps = (searchColumns?: (keyof SearchRow)[]) =>
  ({
    columns: searchColumnsDef,
    data: searchData,
    searchColumns
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  }) as any;

const typeSearch = async (value: string) => {
  const input = screen.getByPlaceholderText('Search...');
  await fireEvent.input(input, { target: { value } });
};

describe('Table search', () => {
  it('renders no search input without searchColumns', () => {
    render(Table, searchProps());
    expect(screen.queryByPlaceholderText('Search...')).not.toBeInTheDocument();
  });

  it('renders a search input when searchColumns is provided', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Search...')).toBeInTheDocument();
  });

  it('filters rows by a matching column value', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    await typeSearch('bob');

    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.queryByText('Alice')).not.toBeInTheDocument();
  });

  it('matches case-insensitively', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Alice')).toBeInTheDocument();

    await typeSearch('ALICE');

    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.queryByText('Bob')).not.toBeInTheDocument();
  });

  it('trims surrounding whitespace from the search term', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    await typeSearch('   bob   ');

    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.queryByText('Alice')).not.toBeInTheDocument();
  });

  it('searches across every configured column', async () => {
    render(Table, searchProps(['name', 'email']));
    expect(await screen.findByText('Alice')).toBeInTheDocument();

    // Matches Alice via her email even though her name does not match.
    await typeSearch('test.org');

    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.queryByText('Bob')).not.toBeInTheDocument();
  });

  it('ignores columns that are not configured for search', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    // "test.org" only appears in the email column, which is not searchable.
    await typeSearch('test.org');

    expect(await screen.findByText('No results.')).toBeInTheDocument();
  });

  it('stringifies non-string column values when matching', async () => {
    render(Table, searchProps(['count']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    await typeSearch('22');

    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.queryByText('Bob')).not.toBeInTheDocument();
  });

  it('matches array column values via their stringified form', async () => {
    render(Table, searchProps(['groups']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    await typeSearch('admin');

    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.queryByText('Alice')).not.toBeInTheDocument();
  });

  it('shows "No results." when nothing matches', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    await typeSearch('nonexistent');

    expect(await screen.findByText('No results.')).toBeInTheDocument();
  });

  it('restores all rows when the search term is cleared', async () => {
    render(Table, searchProps(['name']));
    expect(await screen.findByText('Bob')).toBeInTheDocument();

    await typeSearch('bob');
    expect(screen.queryByText('Alice')).not.toBeInTheDocument();

    await typeSearch('');
    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.getByText('Alice')).toBeInTheDocument();
  });
});
