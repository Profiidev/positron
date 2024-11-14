import { redirect } from "@sveltejs/kit";

export const load = ({ cookies }) => {
  let cookie = cookies.get("token");

  if (!cookie) {
    redirect(302, "/login");
  }
};
