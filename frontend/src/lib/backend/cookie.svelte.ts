const COOKIE_KEYS = "cookie_keys";

interface Cookie {
  key: string;
  value: string;
  maxAge: number;
}

export const setTokenCookie = (cookie_str: string) => {
  let cookie = parseCookie(cookie_str);
  if (!cookie || cookie.key !== "token") {
    return;
  }

  if (cookie.value === "" || cookie.maxAge === 0) {
    removeCookie(cookie.key);
  } else {
    saveCookie(cookie);
  }
};

export const getTokenCookie = () => {
  let storeValue = localStorage.getItem("token");
  if (!storeValue) return;

  let [value, expires_timestamp] = JSON.parse(storeValue);
  if (expires_timestamp < Date.now()) {
    removeCookie("token");
  }

  return value;
};

const saveCookie = (cookie: Cookie) => {
  let cookies: string[] = JSON.parse(localStorage.getItem(COOKIE_KEYS) || "[]");
  if (!cookies.includes(cookie.key)) {
    cookies.push(cookie.key);
    localStorage.setItem(COOKIE_KEYS, JSON.stringify(cookies));
  }

  localStorage.setItem(
    cookie.key,
    JSON.stringify([cookie.value, Date.now() + cookie.maxAge]),
  );
};

const removeCookie = (cookie: string) => {
  let cookies: string[] = JSON.parse(localStorage.getItem(COOKIE_KEYS) || "[]");
  if (cookies.includes(cookie)) {
    cookies = cookies.filter((c) => c !== cookie);
    localStorage.setItem(COOKIE_KEYS, JSON.stringify(cookies));
  }

  localStorage.removeItem(cookie);
};

const parseCookie = (cookie: string): Cookie | undefined => {
  const parts = cookie.split(";").map((part) => part.trim());
  const [key, value] = parts[0].split("=");
  const maxAgePart = parts.find((part) =>
    part.toLowerCase().startsWith("max-age="),
  );

  const maxAge = maxAgePart ? parseInt(maxAgePart.split("=")[1], 10) : null;

  if (maxAge !== null) {
    return { key, value, maxAge };
  }
};
