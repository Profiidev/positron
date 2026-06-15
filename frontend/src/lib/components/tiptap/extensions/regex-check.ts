export const getRegex = (
  searchString: string,
  disableRegex: boolean,
  caseSensitive: boolean
): RegExp => {
  const escapedString = disableRegex
    ? searchString.replace(/[-/\\^$*+?.()|[\]{}]/g, String.raw`\$&`)
    : searchString;
  return new RegExp(escapedString, caseSensitive ? 'gu' : 'gui');
};

export const isValidSearchPattern = (
  searchString: string,
  useRegex: boolean,
  caseSensitive: boolean
): boolean => {
  if (!searchString || !useRegex) {
    return true;
  }

  try {
    getRegex(searchString, false, caseSensitive);
    return true;
  } catch {
    return false;
  }
};
