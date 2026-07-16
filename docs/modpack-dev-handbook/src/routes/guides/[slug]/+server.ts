import { redirect } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = ({ url }) => {
  return redirect(308, url.pathname.replace(/^\/guides(?=\/)/, "/guide"));
};
