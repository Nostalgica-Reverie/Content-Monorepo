#!/usr/bin/env sh
# Commit paths supplied as arguments and push through the normal pipeline.
set -eu

: "${TANGLED_CI_SSH_KEY:?configure this Tangled repository secret}"
: "${TANGLED_PUSH_URL:?configure this Tangled repository secret}"

key_dir="${RUNNER_TEMP:-/tmp}/tangled-ci"
mkdir -p "$key_dir"
key_file="$key_dir/id_ed25519"
printf '%s\n' "$TANGLED_CI_SSH_KEY" > "$key_file"
chmod 600 "$key_file"

git config user.name "tangled-ci[bot]"
git config user.email "tangled-ci@noreply.invalid"
git add -- "$@"
git diff --cached --quiet && exit 0
git commit -m "${TANGLED_COMMIT_MESSAGE:?set TANGLED_COMMIT_MESSAGE}"
GIT_SSH_COMMAND="ssh -i $key_file -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new" \
  git push "$TANGLED_PUSH_URL" "HEAD:refs/heads/${TANGLED_REPO_DEFAULT_BRANCH}"
