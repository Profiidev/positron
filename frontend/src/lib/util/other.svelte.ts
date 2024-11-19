export const isUrl = (url: string) => {
  try {
    let _ = new URL(url);
    return true;
  } catch (_) {
    return false;
  }
};
