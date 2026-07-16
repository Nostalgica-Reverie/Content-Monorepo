import * as sitemap from "super-sitemap";
import type { RequestHandler } from "@sveltejs/kit";
import { siteConfig } from "$lib/site";

export const prerender = true;

export const GET: RequestHandler = async () => {
  return await sitemap.response({
    origin: siteConfig.handbook.siteUrl,
    excludeRoutePatterns: ["^/sitemap.xml", "^/robots.txt"],
    defaultChangefreq: "weekly",
  });
};
