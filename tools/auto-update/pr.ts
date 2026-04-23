import { execFileSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

const FORGEJO_URL = 'https://git.nostalgica.net';
const BRANCH = 'auto-update';
const TARGET = 'main';
const PR_TITLE = 'chore: auto-update mods';

const token = process.env.FORGEJO_TOKEN;
const repo = process.env.GITHUB_REPOSITORY;

if (!token) {
  console.error('FORGEJO_TOKEN not set');
  process.exit(1);
}
if (!repo) {
  console.error('GITHUB_REPOSITORY not set');
  process.exit(1);
}

const apiBase = `${FORGEJO_URL}/api/v1/repos/${repo}`;

async function api(
  method: string,
  path: string,
  body?: unknown,
): Promise<Response> {
  const res = await fetch(`${apiBase}${path}`, {
    method,
    headers: {
      Authorization: `token ${token}`,
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  return res;
}

interface ForgejoPR {
  number: number;
  state: string;
  head: { ref: string };
  base: { ref: string };
  title: string;
}

async function findOpenPR(): Promise<ForgejoPR | null> {
  const owner = repo!.split('/')[0];
  const res = await api(
    'GET',
    `/pulls?state=open&head=${owner}:${BRANCH}&base=${TARGET}&limit=50`,
  );
  if (!res.ok) {
    throw new Error(`list PRs failed: ${res.status} ${await res.text()}`);
  }
  const prs = (await res.json()) as ForgejoPR[];
  const match = prs.find(
    (p) => p.head.ref === BRANCH && p.base.ref === TARGET && p.state === 'open',
  );
  return match ?? null;
}

async function postComment(prNumber: number, body: string): Promise<void> {
  const res = await api('POST', `/issues/${prNumber}/comments`, { body });
  if (!res.ok) {
    throw new Error(`comment failed: ${res.status} ${await res.text()}`);
  }
}

async function createPR(body: string): Promise<number> {
  const res = await api('POST', '/pulls', {
    title: PR_TITLE,
    body,
    head: BRANCH,
    base: TARGET,
  });
  if (!res.ok) {
    throw new Error(`create PR failed: ${res.status} ${await res.text()}`);
  }
  const pr = (await res.json()) as ForgejoPR;
  return pr.number;
}

async function updatePR(prNumber: number, body: string): Promise<void> {
  const res = await api('PATCH', `/pulls/${prNumber}`, { body });
  if (!res.ok) {
    throw new Error(`update PR failed: ${res.status} ${await res.text()}`);
  }
}

function runModDiff(
  oldRef: string,
  newRef: string,
  pathPrefix: string,
): string {
  const bin = process.env.MOD_DIFF_BIN || './updater-bin/mod-diff';
  if (!fs.existsSync(bin)) {
    console.warn(`mod-diff bin not found at ${bin}; skipping diff`);
    return '';
  }
  try {
    return execFileSync(bin, [oldRef, newRef, pathPrefix], {
      encoding: 'utf-8',
    }).trim();
  } catch (e) {
    console.warn(
      `mod-diff failed for ${pathPrefix}: ${e instanceof Error ? e.message : e}`,
    );
    return '';
  }
}

function findModpackSubdirs(): Array<[string, string]> {
  const out: Array<[string, string]> = [];
  const root = 'modpacks';
  if (!fs.existsSync(root)) return out;

  for (const pack of fs.readdirSync(root)) {
    const packPath = path.join(root, pack);
    if (!fs.statSync(packPath).isDirectory()) continue;
    for (const sub of fs.readdirSync(packPath)) {
      if (!/-mr$|-cf$/.test(sub)) continue;
      const subPath = path.join(packPath, sub);
      if (fs.statSync(subPath).isDirectory()) {
        out.push([`${pack}/${sub}`, subPath]);
      }
    }
  }
  return out;
}

function buildPRBody(): string {
  const sections: string[] = [];
  sections.push(
    'Automated PR from the auto update action.);

  const subdirs = findModpackSubdirs();
  const diffSections: string[] = [];

  for (const [label, subdir] of subdirs) {
    const md = runModDiff(`origin/${TARGET}`, 'HEAD', subdir);
    if (md) {
      diffSections.push(`### ${label}\n\n${md}`);
    }
  }

  if (diffSections.length === 0) {
    sections.push(
      '\n## Mod Updates\n\n_No mod version changes detected.',
    );
  } else {
    sections.push('\n## Mod Updates\n\n' + diffSections.join('\n\n'));
  }

  const timestamp = new Date().toISOString();
  sections.push(`\n---\n_Last updated: ${timestamp}_`);

  return sections.join('\n');
}

async function prePush(): Promise<void> {
  const existing = await findOpenPR();
  if (!existing) {
    console.log('no existing open PR; nothing to announce.');
    return;
  }
  const timestamp = new Date().toISOString();
  await postComment(
    existing.number,
    `Force-pushing updated auto-update branch (${timestamp}).`,
  );
  console.log(`posted force-push notice to PR #${existing.number}`);
}

async function postPush(): Promise<void> {
  const body = buildPRBody();
  const existing = await findOpenPR();
  if (existing) {
    await updatePR(existing.number, body);
    console.log(`updated PR #${existing.number}`);
  } else {
    const num = await createPR(body);
    console.log(`created PR #${num}`);
  }
}

const phase = process.argv[2];
(async () => {
  try {
    if (phase === 'pre-push') {
      await prePush();
    } else if (phase === 'post-push') {
      await postPush();
    } else {
      console.error('usage: tsx pr.ts <pre-push|post-push>');
      process.exit(1);
    }
  } catch (e) {
    console.error(`pr.ts ${phase} failed:`, e instanceof Error ? e.message : e);
    process.exit(1);
  }
})();