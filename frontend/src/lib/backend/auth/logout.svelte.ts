import { PUBLIC_BACKEND_URL } from "$env/static/public";

export const logout = async (): Promise<undefined | null> => {
  try {
    let info_res = await fetch(`${PUBLIC_BACKEND_URL}/auth/logout`, {
      method: "POST",
    });

    if (info_res.status !== 200) {
      return;
    }

    return null;
  } catch (_) {
    return;
  }
};
