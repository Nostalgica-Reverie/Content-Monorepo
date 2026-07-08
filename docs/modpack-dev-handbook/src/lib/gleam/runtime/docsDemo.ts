// @ts-expect-error Generated Gleam JavaScript module does not ship TypeScript declarations.
import { compare_versions as rawCompareVersions, slugify as rawSlugify } from "./docs_demo/docs_demo.mjs";

export const compareVersions: (left: string, right: string) => string = rawCompareVersions;
export const slugifyPackName: (value: string) => string = rawSlugify;