import * as sitemap from "super-sitemap";
import type { RequestHandler } from "@sveltejs/kit";

export const prerender = true;

export const GET: RequestHandler = async () => {
  return await sitemap.response({
    origin: "https://datapack.wiki",
    excludeRoutePatterns: ["^/sitemap.xml", "^/meta.json", "^/robots.txt", "^/search.json"],
    defaultChangefreq: "weekly",
  });
};

