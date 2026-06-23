import { describe, expect, it } from 'vitest';
import {
  expiryClass,
  formatExpiry,
  formatRelativeFuture,
  formatRelativeOptional,
  formatRelativePast,
  getExpiryUrgency,
  otherSessionCount,
  sessionDisplayName,
  sessionSubtitle
} from '$routes/account/sessions/session-utils';

const now = new Date('2024-06-01T12:00:00Z');

describe('sessionSubtitle', () => {
  it('joins application and operating system', () => {
    expect(sessionSubtitle('Chrome 126', 'macOS 15.1')).toBe(
      'Chrome 126 - macOS 15.1'
    );
  });

  it('falls back when both values are empty', () => {
    expect(sessionSubtitle('', '')).toBe('Unknown device');
  });
});

describe('sessionDisplayName', () => {
  it('uses the stored name when present', () => {
    expect(sessionDisplayName('MacBook Pro', false)).toBe('MacBook Pro');
  });

  it('falls back to generic browser and app labels', () => {
    expect(sessionDisplayName('', false)).toBe('Browser session');
    expect(sessionDisplayName('  ', true)).toBe('App session');
  });
});

describe('otherSessionCount', () => {
  it('counts only non-current sessions', () => {
    expect(
      otherSessionCount([
        { current: true },
        { current: false },
        { current: false }
      ])
    ).toBe(2);
  });
});

describe('formatRelativePast', () => {
  it('returns just now for recent activity', () => {
    expect(formatRelativePast(new Date('2024-06-01T11:59:30Z'), now)).toBe(
      'just now'
    );
  });

  it('returns a relative past label for older timestamps', () => {
    expect(formatRelativePast(new Date('2024-05-20T12:00:00Z'), now)).toMatch(
      /day/
    );
  });
});

describe('formatRelativeFuture', () => {
  it('returns a relative future label', () => {
    expect(formatRelativeFuture(new Date('2024-06-19T12:00:00Z'), now)).toMatch(
      /day/
    );
  });

  it('returns expired for past expiry timestamps', () => {
    expect(formatRelativeFuture(new Date('2024-05-01T12:00:00Z'), now)).toBe(
      'expired'
    );
  });
});

describe('formatRelativeOptional', () => {
  it('returns a dash when refresh is missing', () => {
    expect(formatRelativeOptional(null, now)).toBe('-');
  });
});

describe('getExpiryUrgency', () => {
  it('marks sessions expiring within a day as critical', () => {
    expect(getExpiryUrgency(new Date('2024-06-01T18:00:00Z'), now)).toBe(
      'critical'
    );
  });

  it('marks sessions expiring within three days as warning', () => {
    expect(getExpiryUrgency(new Date('2024-06-03T12:00:00Z'), now)).toBe(
      'warning'
    );
  });

  it('marks distant expiry as normal', () => {
    expect(getExpiryUrgency(new Date('2024-07-01T12:00:00Z'), now)).toBe(
      'normal'
    );
  });
});

describe('formatExpiry', () => {
  it('prefixes urgent expiry labels with a dot', () => {
    expect(formatExpiry(new Date('2024-06-01T18:00:00Z'), now).text).toMatch(
      /^in 6 hours/
    );
  });

  it('leaves normal expiry labels unprefixed', () => {
    expect(
      formatExpiry(new Date('2024-07-01T12:00:00Z'), now).text
    ).not.toMatch(/^• /);
  });
});

describe('expiryClass', () => {
  it('maps urgency levels to text classes', () => {
    expect(expiryClass('critical')).toBe('text-destructive');
    expect(expiryClass('warning')).toBe('text-amber-500');
    expect(expiryClass('normal')).toBe('');
  });
});

describe('formatRelative branch coverage', () => {
  it.each([
    { date: '2024-06-01T11:30:00Z', unit: /minute/ },
    { date: '2024-06-01T09:00:00Z', unit: /hour/ },
    { date: '2024-05-20T12:00:00Z', unit: /day/ },
    { date: '2024-04-01T12:00:00Z', unit: /month/ },
    { date: '2022-06-01T12:00:00Z', unit: /year/ }
  ])('formats past timestamp $date with the right unit', ({ date, unit }) => {
    expect(formatRelativePast(new Date(date), now)).toMatch(unit);
  });

  it.each([
    { date: '2024-06-01T12:30:00Z', unit: /minute/ },
    { date: '2024-06-01T15:00:00Z', unit: /hour/ },
    { date: '2024-06-19T12:00:00Z', unit: /day/ },
    { date: '2024-08-01T12:00:00Z', unit: /month/ },
    { date: '2026-06-01T12:00:00Z', unit: /year/ }
  ])('formats future timestamp $date with the right unit', ({ date, unit }) => {
    expect(formatRelativeFuture(new Date(date), now)).toMatch(unit);
  });

  it('treats an exactly-now expiry as expired', () => {
    expect(formatRelativeFuture(now, now)).toBe('expired');
  });

  it('formats a present optional date as a relative past label', () => {
    expect(
      formatRelativeOptional(new Date('2024-05-20T12:00:00Z'), now)
    ).toMatch(/day/);
  });
});

describe('getExpiryUrgency boundaries', () => {
  it('treats expiry exactly one day out as critical', () => {
    expect(getExpiryUrgency(new Date('2024-06-02T12:00:00Z'), now)).toBe(
      'critical'
    );
  });

  it('treats expiry exactly three days out as warning', () => {
    expect(getExpiryUrgency(new Date('2024-06-04T12:00:00Z'), now)).toBe(
      'warning'
    );
  });
});

describe('formatExpiry urgency', () => {
  it('carries the urgency alongside the relative text', () => {
    expect(formatExpiry(new Date('2024-06-01T18:00:00Z'), now).urgency).toBe(
      'critical'
    );
    expect(formatExpiry(new Date('2024-07-01T12:00:00Z'), now).urgency).toBe(
      'normal'
    );
  });
});
