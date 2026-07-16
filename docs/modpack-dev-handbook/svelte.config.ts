import adapter from "@sveltejs/adapter-static";
import { DOCS_BASE } from "./src/lib/site.ts";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { escapeSvelte, mdsvex } from "mdsvex";
import { createHighlighter } from "shiki";
import mcfunction from "./src/lib/highlighting/mcfunction/mcfunction.ts";
import { theme } from "./src/lib/highlighting/kanagawa-wave.ts";
import rehypeAutolinkHeadings from "rehype-autolink-headings";
import remarkAdmonitions from "remark-admonitions";
import remarkCodeTitles from "remark-code-titles";
import rehypeSlug from "rehype-slug";
import type { Config } from "@sveltejs/kit";
import type { Options as RehypeAutolinkOptions } from "rehype-autolink-headings";

const highlighter = await createHighlighter({
  langs: [mcfunction, "json", "md", "javascript", "toml", "yaml", "bash"],
  themes: [theme],
});

const admonitionsOptions = {
  useDefaultTypes: false,
  customTypes: {
    note: {
      ifmClass: "secondary",
      keyword: "note",
      svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icon-tabler-info-circle"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" /><path d="M12 9h.01" /><path d="M11 12h1v4h1" /></svg>',
    },
    tip: {
      ifmClass: "success",
      keyword: "tip",
      svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icon-tabler-bulb"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M3 12h1m8 -9v1m8 8h1m-15.4 -6.4l.7 .7m12.1 -.7l-.7 .7" /><path d="M9 16a5 5 0 1 1 6 0a3.5 3.5 0 0 0 -1 3a2 2 0 0 1 -4 0a3.5 3.5 0 0 0 -1 -3" /><path d="M9.7 17l4.6 0" /></svg>',
    },
    warning: {
      ifmClass: "danger",
      keyword: "warning",
      svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icon-tabler-alert-triangle"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 9v4" /><path d="M10.363 3.591l-8.106 13.534a1.914 1.914 0 0 0 1.636 2.871h16.214a1.914 1.914 0 0 0 1.636 -2.87l-8.106 -13.536a1.914 1.914 0 0 0 -3.274 0z" /><path d="M12 16h.01" /></svg>',
    },
    info: {
      ifmClass: "info",
      keyword: "info",
      svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icon-tabler-info-circle"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" /><path d="M12 9h.01" /><path d="M11 12h1v4h1" /></svg>',
    },
  },
};

const autoLinkOptions: RehypeAutolinkOptions = {
  behavior: "append",
  content: {
    type: "text",
    value:
      '<svg xmlns="http://www.w3.org/2000/svg" class="inline-block ml-2 stroke-stone-700 focus:stroke-stone-400 hover:stroke-stone-400 z-0 motion-safe:transition-colors" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M9 15l6 -6" /><path d="M11 6l.463 -.536a5 5 0 0 1 7.071 7.072l-.534 .464" /><path d="M13 18l-.397 .534a5.068 5.068 0 0 1 -7.127 0a4.972 4.972 0 0 1 0 -7.071l.524 -.463" /></svg>',
  },
};

const config: Config = {
  preprocess: [
    vitePreprocess(),
    mdsvex({
      extensions: [".svx", ".md"],
      remarkPlugins: [[remarkAdmonitions, admonitionsOptions], [remarkCodeTitles]],
      rehypePlugins: [[rehypeSlug], [rehypeAutolinkHeadings, autoLinkOptions]],
      layout: `${import.meta.dirname}/src/lib/MDLayout.svelte`,
      highlight: {
        highlighter: (code, lang) => {
          const generated = escapeSvelte(highlighter.codeToHtml(code, { lang, theme }));
          return `{@html \`${generated}\` }`;
        },
      },
    }),
  ],
  kit: {
    paths: {
      base: DOCS_BASE,
      relative: false,
    },
    adapter: adapter({ fallback: "404.html" }),
    prerender: {
      crawl: false,
      handleHttpError: "warn",
      handleMissingId: "warn",
      entries: ["*"],
    },
  },
  extensions: [".svelte", ".svx", ".md"],
};

export default config;
