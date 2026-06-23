export type ExpiryUrgency = 'critical' | 'normal' | 'warning';

const MINUTE_MS = 60_000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;
const MONTH_MS = 30 * DAY_MS;
const YEAR_MS = 365 * DAY_MS;

const CRITICAL_MS = DAY_MS;
const WARNING_MS = 3 * DAY_MS;

export const sessionSubtitle = (
  application: string,
  operatingSystem: string
): string => {
  const parts = [application, operatingSystem].filter((part) => part.trim());
  return parts.join(' - ') || 'Unknown device';
};

export const sessionDisplayName = (name: string, isApp: boolean): string => {
  if (name.trim()) {
    return name;
  }
  return isApp ? 'App session' : 'Browser session';
};

export const otherSessionCount = (sessions: { current: boolean }[]): number =>
  sessions.filter((session) => !session.current).length;

const formatRelativeMs = (diffMs: number, future: boolean): string => {
  const sign = future ? 1 : -1;
  const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });

  if (diffMs < HOUR_MS) {
    return rtf.format(sign * Math.round(diffMs / MINUTE_MS), 'minute');
  }
  if (diffMs < DAY_MS) {
    return rtf.format(sign * Math.round(diffMs / HOUR_MS), 'hour');
  }
  if (diffMs < MONTH_MS) {
    return rtf.format(sign * Math.round(diffMs / DAY_MS), 'day');
  }
  if (diffMs < YEAR_MS) {
    return rtf.format(sign * Math.round(diffMs / MONTH_MS), 'month');
  }
  return rtf.format(sign * Math.round(diffMs / YEAR_MS), 'year');
};

export const formatRelativePast = (date: Date, now = new Date()): string => {
  const diffMs = now.getTime() - date.getTime();
  if (diffMs < MINUTE_MS) {
    return 'just now';
  }
  return formatRelativeMs(diffMs, false);
};

export const formatRelativeFuture = (date: Date, now = new Date()): string => {
  const diffMs = date.getTime() - now.getTime();
  if (diffMs <= 0) {
    return 'expired';
  }
  return formatRelativeMs(diffMs, true);
};

export const formatRelativeOptional = (
  date: Date | null | undefined,
  now = new Date()
): string => {
  if (!date) {
    return '-';
  }
  return formatRelativePast(date, now);
};

export const getExpiryUrgency = (
  expiresAt: Date,
  now = new Date()
): ExpiryUrgency => {
  const diffMs = expiresAt.getTime() - now.getTime();
  if (diffMs <= CRITICAL_MS) {
    return 'critical';
  }
  if (diffMs <= WARNING_MS) {
    return 'warning';
  }
  return 'normal';
};

export const formatExpiry = (
  expiresAt: Date,
  now = new Date()
): { text: string; urgency: ExpiryUrgency } => {
  const urgency = getExpiryUrgency(expiresAt, now);
  const relative = formatRelativeFuture(expiresAt, now);
  return {
    text: relative,
    urgency
  };
};

export const expiryClass = (urgency: ExpiryUrgency): string => {
  switch (urgency) {
    case 'critical': {
      return 'text-destructive';
    }
    case 'warning': {
      return 'text-amber-500';
    }
    default: {
      return '';
    }
  }
};
