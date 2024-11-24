import { redirect } from "@sveltejs/kit";

export const load = ({ cookies, url }) => {
  let cookie = cookies.get("token");

  if (!cookie && url.pathname !== "/login" && url.pathname !== "/oauth") {
    redirect(302, "/login");
  }
};
