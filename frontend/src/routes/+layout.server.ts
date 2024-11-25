import { PUBLIC_IS_APP } from "$env/static/public";
import { redirect } from "@sveltejs/kit";

export const load = ({ cookies, url }) => {
  if (PUBLIC_IS_APP === "true") return;

  let cookie = cookies.get("token");

  if (!cookie && url.pathname !== "/login" && url.pathname !== "/oauth") {
    redirect(302, "/login");
  }
};
