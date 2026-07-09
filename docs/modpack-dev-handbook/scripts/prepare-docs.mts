import matter from "gray-matter";
import fg from "fast-glob";
import { promises as fs } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const repoRoot = path.resolve(appRoot, "..", "..");
const routesRoot = path.join(appRoot, "src", "routes");
const generatedRoot = path.join(appRoot, "src", "lib", "generated");
const docsModulePath = path.join(generatedRoot, "docs.ts");
const searchModulePath = path.join(generatedRoot, "search.ts");

type DocRecord = {
  title: string;
  url: string;
  sourcePath: string;
  description: string;
  tags: string[];
  content: string;
};

type SearchDocument = {
  kind: "page" | "section";
  title: string;
  pageTitle: string;
  sectionTitle: string;
  description: string;
  content: string;
  url: string;
  tags: string[];
};

type NavNode = {
  title: string;
  url: string | null;
  children: NavNode[];
};

type NavSection = {
  title: string;
  children: NavNode[];
};

const sections = [
  { title: "Start Here", prefixes: [{ prefix: "/", title: "Home" }] },
  { title: "Guides", prefixes: [{ prefix: "/guide", title: "Guides" }] },
  {
    title: "Reference",
    prefixes: [
      { prefix: "/wiki/info", title: "Info" },
      { prefix: "/wiki/planning", title: "Planning" },
      { prefix: "/wiki/useful-mods", title: "Useful Mods" },
    ],
  },
  { title: "Pack Management", prefixes: [{ prefix: "/wiki/modpack-management", title: "Pack Management" }] },
  { title: "Contribute", prefixes: [{ prefix: "/contribute", title: "Contribute" }, { prefix: "/credits", title: "Credits" }] },
] as const;

await main();

async function main() {
  const { docs, searchDocuments } = await collectDocs();
  const navSections = buildNavigation(docs);

  await fs.mkdir(generatedRoot, { recursive: true });

  const docsModuleSource = [
    "export type DocMeta = {",
    "  title: string;",
    "  url: string;",
    "  sourcePath: string;",
    "  description: string;",
    "  tags: string[];",
    "};",
    "",
    "export type NavNode = {",
    "  title: string;",
    "  url: string | null;",
    "  children: NavNode[];",
    "};",
    "",
    "export type NavSection = {",
    "  title: string;",
    "  children: NavNode[];",
    "};",
    "",
    `export const docsIndex: DocMeta[] = ${JSON.stringify(docs.map(({ content, ...rest }) => rest), null, 2)};`,
    "",
    `export const navSections: NavSection[] = ${JSON.stringify(navSections, null, 2)};`,
    "",
    "export function normalizeDocUrl(url: string): string {",
    "  if (url.length > 1 && url.endsWith('/')) return url.slice(0, -1);",
    "  return url || '/';",
    "}",
    "",
    "const docsByUrl = new Map(docsIndex.map((doc) => [normalizeDocUrl(doc.url), doc]));",
    "",
    "export function findDocByUrl(url: string): DocMeta | undefined {",
    "  return docsByUrl.get(normalizeDocUrl(url));",
    "}",
    "",
  ].join("\n");
  await fs.writeFile(docsModulePath, docsModuleSource);

  const searchModuleSource = [
    "export type SearchDocument = {",
    "  kind: 'page' | 'section';",
    "  title: string;",
    "  pageTitle: string;",
    "  sectionTitle: string;",
    "  description: string;",
    "  content: string;",
    "  url: string;",
    "  tags: string[];",
    "};",
    "",
    `export const searchDocuments: SearchDocument[] = ${JSON.stringify(searchDocuments, null, 2)};`,
    "",
  ].join("\n");
  await fs.writeFile(searchModulePath, searchModuleSource);

  console.log(`Prepared docs: ${docs.length} indexed pages and ${searchDocuments.length} search documents.`);
}

async function collectDocs(): Promise<{ docs: DocRecord[]; searchDocuments: SearchDocument[] }> {
  const docs: DocRecord[] = [];
  const searchDocuments: SearchDocument[] = [];
  const files = await fg(["**/+page.svx", "**/+page.md"], {
    cwd: routesRoot,
    absolute: true,
    onlyFiles: true,
  });

  for (const file of files) {
    const url = routeUrlFromFile(file);
    if (url === "/sitemap.xml") continue;

    const raw = await fs.readFile(file, "utf8");
    const { data, content } = matter(raw);
    const title = getTitle(data, content, file);
    const description = typeof data.description === "string" ? data.description.trim() : "";
    const tags = Array.isArray(data.tags) ? data.tags.map(String) : [];
    const sourcePath = toPosix(path.relative(repoRoot, file));
    const strippedContent = stripForSearch(content);

    docs.push({ title, url, sourcePath, description, tags, content: strippedContent });
    searchDocuments.push({
      kind: "page",
      title,
      pageTitle: title,
      sectionTitle: "",
      description,
      content: strippedContent,
      url,
      tags,
    });
    searchDocuments.push(...extractSections(content, url, title, description, tags));
  }

  docs.sort((a, b) => a.url.localeCompare(b.url));
  searchDocuments.sort((a, b) => a.url.localeCompare(b.url) || a.title.localeCompare(b.title));
  return { docs, searchDocuments };
}

function extractSections(content: string, baseUrl: string, pageTitle: string, description: string, tags: string[]): SearchDocument[] {
  const matches = [...content.matchAll(/^(#{2,6})\s+(.+)$/gm)];
  if (matches.length === 0) {
    return [];
  }

  const slugCounts = new Map<string, number>();
  return matches.flatMap((match, index) => {
    const headingMarkup = match[2]?.trim();
    if (!headingMarkup) {
      return [];
    }

    const heading = stripInlineMarkdown(headingMarkup);
    if (!heading) {
      return [];
    }

    const start = (match.index ?? 0) + match[0].length;
    const end = matches[index + 1]?.index ?? content.length;
    const sectionContent = stripForSearch(content.slice(start, end));
    const baseSlug = slugifyHeading(heading);
    const count = slugCounts.get(baseSlug) ?? 0;
    slugCounts.set(baseSlug, count + 1);
    const slug = count === 0 ? baseSlug : `${baseSlug}-${count}`;

    return [{
      kind: "section",
      title: `${pageTitle} / ${heading}`,
      pageTitle,
      sectionTitle: heading,
      description,
      content: sectionContent || heading,
      url: `${baseUrl}#${slug}`,
      tags,
    }];
  });
}

function buildNavigation(docs: DocRecord[]): NavSection[] {
  return sections
    .map((section) => ({
      title: section.title,
      children: section.prefixes.map(({ prefix, title }) => buildTreeForPrefix(docs, prefix, title)).filter(Boolean) as NavNode[],
    }))
    .filter((section) => section.children.length > 0);
}

function buildTreeForPrefix(docs: DocRecord[], prefix: string, fallbackTitle: string): NavNode | null {
  const relevant = docs.filter((doc) => doc.url === prefix || doc.url.startsWith(prefix === "/" ? "/" : `${prefix}/`));
  if (prefix === "/") {
    const home = docs.find((doc) => doc.url === "/");
    return home ? { title: home.title, url: home.url, children: [] } : null;
  }
  if (relevant.length === 0) return null;

  const rootPage = relevant.find((doc) => normalizeUrl(doc.url) === normalizeUrl(prefix));
  const root: NavNode = { title: rootPage?.title ?? fallbackTitle, url: rootPage?.url ?? null, children: [] };

  for (const doc of relevant) {
    if (normalizeUrl(doc.url) === normalizeUrl(prefix)) continue;
    const segments = doc.url.slice(prefix.length).replace(/^\//, "").split("/").filter(Boolean);
    let cursor = root;
    for (let currentIndex = 0; currentIndex < segments.length; currentIndex += 1) {
      const segment = segments[currentIndex]!;
      const isLeaf = currentIndex === segments.length - 1;
      const title = isLeaf ? doc.title : titleize(segment);
      let child = cursor.children.find((node) => node.title === title);
      if (!child) {
        child = { title, url: isLeaf ? doc.url : null, children: [] };
        cursor.children.push(child);
      }
      if (isLeaf) child.url = doc.url;
      cursor = child;
    }
  }

  walkNodes(root, (node) => node.children.sort(compareNavNodes));
  return root;
}

function walkNodes(node: NavNode, fn: (node: NavNode) => void) {
  fn(node);
  for (const child of node.children) walkNodes(child, fn);
}

function compareNavNodes(a: NavNode, b: NavNode) {
  if (a.children.length === 0 && b.children.length > 0) return 1;
  if (a.children.length > 0 && b.children.length === 0) return -1;
  return a.title.localeCompare(b.title);
}

function routeUrlFromFile(file: string) {
  const relative = toPosix(path.relative(routesRoot, file));
  const dir = toPosix(path.dirname(relative));
  return normalizeUrl(dir === "." ? "/" : `/${dir}`);
}

function getTitle(data: Record<string, unknown>, content: string, file: string) {
  if (typeof data.title === "string" && data.title.trim()) return data.title.trim();
  const match = content.match(/^#\s+(.+)$/m);
  if (match?.[1]) return stripInlineMarkdown(match[1].trim());
  return titleize(path.basename(path.dirname(file)));
}

function stripForSearch(content: string) {
  return content
    .replace(/^---[\s\S]*?---/m, " ")
    .replace(/<!--[\s\S]*?-->/g, " ")
    .replace(/<script[\s\S]*?<\/script>/g, " ")
    .replace(/<style[\s\S]*?<\/style>/g, " ")
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/<[^>]+>/g, " ")
    .replace(/[>#*_~|-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function stripInlineMarkdown(value: string) {
  return value
    .replace(/`([^`]+)`/g, "$1")
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/<[^>]+>/g, " ")
    .replace(/[\\*_~]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function slugifyHeading(value: string) {
  const cleaned = value
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .trim()
    .replace(/[\s-]+/g, "-");
  return cleaned || "section";
}

function titleize(segment: string) {
  const replacements: Record<string, string> = {
    mcfunction: "MCFunction",
    neoforge: "NeoForge",
    curseforge: "CurseForge",
    modrinth: "Modrinth",
    packwand: "Packwand",
    packwiz: "Packwiz",
    pakku: "Pakku",
  };
  return segment
    .split(/[-_]/g)
    .map((part) => replacements[part.toLowerCase()] ?? `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function normalizeUrl(url: string) {
  if (!url || url === "") return "/";
  return url.length > 1 && url.endsWith("/") ? url.slice(0, -1) : url;
}

function toPosix(value: string) {
  return value.split(path.sep).join("/");
}
