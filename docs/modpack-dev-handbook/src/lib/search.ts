import type { SearchDocument } from "$lib/generated/search";

const MAX_RESULTS = 20;
const SNIPPET_RADIUS = 90;

export type SearchResult = { content: string; title: string; url: string };
type IndexedDocument = SearchDocument & {
  normalized: {
    title: string;
    pageTitle: string;
    sectionTitle: string;
    description: string;
    content: string;
    url: string;
    tags: string[];
  };
};

type ParsedQuery = {
  raw: string;
  terms: string[];
  tags: string[];
};

let documents: IndexedDocument[] | null = null;

export async function ensureSearchReady() {
  if (documents) return;

  const { searchDocuments } = await import("$lib/generated/search");
  documents = searchDocuments.map((doc) => ({
    ...doc,
    normalized: {
      title: normalizeText(doc.title),
      pageTitle: normalizeText(doc.pageTitle),
      sectionTitle: normalizeText(doc.sectionTitle),
      description: normalizeText(doc.description),
      content: normalizeText(doc.content),
      url: normalizeText(doc.url),
      tags: doc.tags.map((tag) => normalizeText(tag)),
    },
  }));
}

export function search(query: string): SearchResult[] {
  if (!documents) {
    return [];
  }

  const parsed = parseQuery(query);
  if (!parsed.raw) {
    return [];
  }

  return documents
    .map((doc) => scoreDocument(doc, parsed))
    .filter((result): result is SearchMatch => result !== null)
    .sort((a, b) => b.score - a.score || a.doc.title.localeCompare(b.doc.title))
    .slice(0, MAX_RESULTS)
    .map(({ doc }) => ({
      title: highlightText(doc.title, parsed.terms),
      url: doc.url,
      content: buildSnippet(doc, parsed.terms),
    }));
}

type SearchMatch = {
  doc: IndexedDocument;
  score: number;
};

function scoreDocument(doc: IndexedDocument, parsed: ParsedQuery): SearchMatch | null {
  if (parsed.tags.length > 0 && !parsed.tags.some((tag) => doc.normalized.tags.includes(tag))) {
    return null;
  }

  if (parsed.terms.length === 0) {
    return {
      doc,
      score: doc.kind === "section" ? 18 : 12,
    };
  }

  let score = 0;
  for (const term of parsed.terms) {
    const termScore = scoreTerm(doc, term);
    if (termScore === 0) {
      return null;
    }
    score += termScore;
  }

  if (doc.kind === "section") {
    score += 8;
  }

  return { doc, score };
}

function scoreTerm(doc: IndexedDocument, term: string) {
  let score = 0;

  score += scoreField(doc.normalized.sectionTitle, term, 150, 90);
  score += scoreField(doc.normalized.pageTitle, term, 115, 60);
  score += scoreField(doc.normalized.title, term, 95, 50);
  score += scoreField(doc.normalized.description, term, 40, 20);
  score += scoreField(doc.normalized.url, term, 36, 18);
  score += scoreField(doc.normalized.content, term, 18, 10);

  if (doc.normalized.tags.some((tag) => tag === term)) {
    score += 60;
  }

  return score;
}

function scoreField(value: string, term: string, containsWeight: number, prefixWeight: number) {
  if (!value) return 0;
  if (value.includes(term)) {
    const position = value.indexOf(term);
    return containsWeight + Math.max(0, 12 - Math.min(position, 12));
  }

  for (const token of value.split(/[^\p{L}\p{N}]+/u)) {
    if (token.startsWith(term)) {
      return prefixWeight;
    }
  }

  return 0;
}

function parseQuery(input: string): ParsedQuery {
  const normalizedInput = normalizeText(input).trim();
  if (!normalizedInput) {
    return { raw: "", terms: [], tags: [] };
  }

  const tags: string[] = [];
  const terms: string[] = [];

  for (const part of normalizedInput.split(/\s+/)) {
    if (part.startsWith("tag:")) {
      for (const tag of part.slice(4).split(",").map((value) => value.trim()).filter(Boolean)) {
        tags.push(tag);
      }
      continue;
    }

    if (part.length > 0) {
      terms.push(part);
    }
  }

  return {
    raw: normalizedInput,
    terms: [...new Set(terms)],
    tags: [...new Set(tags)],
  };
}

function buildSnippet(doc: IndexedDocument, terms: string[]) {
  if (terms.length === 0) {
    const summary = doc.description || doc.content.slice(0, SNIPPET_RADIUS * 2);
    return summary ? highlightText(summary, []) : "";
  }

  const source = doc.content || doc.description;
  const bestMatchIndex = findFirstMatchIndex(source, terms);
  if (bestMatchIndex === -1) {
    const fallback = doc.sectionTitle || doc.description || doc.pageTitle;
    return fallback ? highlightText(fallback, terms) : "";
  }

  const start = Math.max(0, bestMatchIndex - SNIPPET_RADIUS);
  const end = Math.min(source.length, bestMatchIndex + SNIPPET_RADIUS);
  const prefix = start > 0 ? "..." : "";
  const suffix = end < source.length ? "..." : "";
  return `${prefix}${highlightText(source.slice(start, end).trim(), terms)}${suffix}`;
}

function findFirstMatchIndex(text: string, terms: string[]) {
  if (!text) return -1;
  const normalized = normalizeText(text);
  let bestMatch = -1;

  for (const term of terms) {
    const index = normalized.indexOf(term);
    if (index !== -1 && (bestMatch === -1 || index < bestMatch)) {
      bestMatch = index;
    }
  }

  return bestMatch;
}

function highlightText(text: string, terms: string[]) {
  if (!text) return "";
  const ranges = collectHighlightRanges(text, terms);
  if (ranges.length === 0) {
    return escapeHtml(text);
  }

  let cursor = 0;
  let output = "";
  for (const [start, end] of ranges) {
    output += escapeHtml(text.slice(cursor, start));
    output += `<mark>${escapeHtml(text.slice(start, end))}</mark>`;
    cursor = end;
  }
  output += escapeHtml(text.slice(cursor));
  return output;
}

function collectHighlightRanges(text: string, terms: string[]) {
  const normalizedText = normalizeText(text);
  const ranges: Array<[number, number]> = [];

  for (const term of terms) {
    if (!term) continue;
    let cursor = 0;
    while (cursor < normalizedText.length) {
      const index = normalizedText.indexOf(term, cursor);
      if (index === -1) break;
      ranges.push([index, index + term.length]);
      cursor = index + term.length;
    }
  }

  ranges.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  const merged: Array<[number, number]> = [];
  for (const range of ranges) {
    const previous = merged.at(-1);
    if (!previous || range[0] > previous[1]) {
      merged.push([range[0], range[1]]);
      continue;
    }

    previous[1] = Math.max(previous[1], range[1]);
  }

  return merged;
}

function escapeHtml(text: string) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function normalizeText(value: string) {
  return value
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .toLowerCase();
}
