// Cross-site link checker for the three VitePress sites in this repo
// (codex.md §4.3). Each site's own `vitepress build` already fails on dead
// *intra*-site links (relative or root-relative) via its default
// ignoreDeadLinks: false - see the per-site .vitepress/config.mts files.
// What that check can never see is a link written as an absolute URL to one
// of the *other* sites (or even back to its own site), since VitePress
// treats any absolute URL as external and never validates it. Those
// hostnames are already committed (sitemap.hostname in each config.mts), so
// this script resolves any markdown link matching one of them against that
// site's built dist/ output.
//
// Run after `just docs-build` (or `bun run docs:build` in each site) has
// produced dist/ for all three sites - this script only reads already-built
// output, it does not build anything itself.
import { readdir, readFile, stat } from "node:fs/promises";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

const SITES = [
  {
    name: "docs",
    hostname: "https://docs.nostalgica.net/",
    contentDir: join(repoRoot, "docs", "docs"),
    distDir: join(repoRoot, "docs", "docs", ".vitepress", "dist"),
  },
  {
    name: "packwand",
    hostname: "https://packwand.nostalgica.net/",
    contentDir: join(repoRoot, "docs", "packwand", "docs"),
    distDir: join(repoRoot, "docs", "packwand", "docs", ".vitepress", "dist"),
  },
  {
    name: "packwiz",
    hostname: "https://packwiz.nostalgica.net/",
    contentDir: join(repoRoot, "docs", "packwiz", "docs"),
    distDir: join(repoRoot, "docs", "packwiz", "docs", ".vitepress", "dist"),
  },
];

const LINK_PATTERN = /\[[^\]]*\]\(([^)\s]+)\)/g;

async function markdownFiles(dir: string): Promise<string[]> {
  const files: string[] = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    if (entry.name.startsWith(".vitepress") || entry.name === "node_modules") continue;
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await markdownFiles(full)));
    } else if (entry.name.endsWith(".md")) {
      files.push(full);
    }
  }
  return files;
}

async function exists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch {
    return false;
  }
}

// Resolves a link path against a site's built dist/ (cleanUrls: true in all
// three configs, so /foo/bar serves foo/bar.html or foo/bar/index.html).
async function resolvesInDist(distDir: string, path: string): Promise<boolean> {
  const clean = path.replace(/^\/+/, "").replace(/\/+$/, "");
  const candidates = clean === ""
    ? [join(distDir, "index.html")]
    : [
      join(distDir, clean),
      join(distDir, `${clean}.html`),
      join(distDir, clean, "index.html"),
    ];
  for (const candidate of candidates) {
    if (await exists(candidate)) return true;
  }
  return false;
}

function siteForHostname(url: string) {
  return SITES.find((site) => url.startsWith(site.hostname));
}

async function main() {
  const distMissing: string[] = [];
  for (const site of SITES) {
    if (!(await exists(site.distDir))) distMissing.push(site.name);
  }
  if (distMissing.length > 0) {
    console.warn(
      `Skipping cross-site link check: no built dist/ for ${distMissing.join(", ")}. ` +
        "Run `just docs-build` (or each site's `bun run docs:build`) first.",
    );
    process.exit(0);
  }

  type BrokenLink = { from: string; url: string; targetSite: string; path: string; fragment: string | undefined };
  const broken: BrokenLink[] = [];
  let checked = 0;

  for (const site of SITES) {
    for (const file of await markdownFiles(site.contentDir)) {
      const text = await readFile(file, "utf-8");
      for (const match of text.matchAll(LINK_PATTERN)) {
        const url = match[1];
        if (url === undefined) continue;
        const target = siteForHostname(url);
        if (!target) continue;
        checked++;
        const [path = "", fragment] = url.slice(target.hostname.length - 1).split("#");
        if (!(await resolvesInDist(target.distDir, path))) {
          broken.push({
            from: relative(repoRoot, file),
            url,
            targetSite: target.name,
            path,
            fragment,
          });
        }
      }
    }
  }

  if (broken.length > 0) {
    console.error(`Found ${broken.length} broken cross-site link(s) (of ${checked} checked):`);
    for (const b of broken) {
      console.error(`  ${b.from}\n    -> ${b.url}  (no page at ${b.path} on the ${b.targetSite} site)`);
    }
    process.exit(1);
  }

  console.log(`Cross-site link check OK: ${checked} link(s) to sibling sites, all resolved.`);
}

await main();
