#!/usr/bin/env bash
# Check out the current PR merged into its current base in the working directory.
# Usage: ci/pr-checkout.sh <repository-url>
# Inputs: TOKEN (read-only repository access), BASE_REF (branch), PR_NUMBER.
# Outputs: merged working tree/HEAD, with Git LFS objects in place of their
# pointers (ci/lfs-fetch.sh), and three log entries on stdout. Fetch, merge or
# LFS failures exit nonzero. Credentials are passed only to the fetch commands.
# Rollout: bootstrap this helper and lfs-fetch.sh into the same directory from
# the default branch, never from PR code.
set -euo pipefail
repo=${1:?repository URL required}
: "${TOKEN:?read-only token required}" "${BASE_REF:?base branch required}" "${PR_NUMBER:?PR number required}"
auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
git init -q .
for attempt in 1 2 3; do
  git -c http.extraHeader="Authorization: basic $auth" \
    fetch -q "$repo" \
    "+refs/heads/$BASE_REF:refs/remotes/origin/base" \
    "+refs/pull/$PR_NUMBER/head:refs/remotes/origin/pr" \
    && break
  [ "$attempt" = 3 ] && exit 1
  sleep 5
done
git checkout -q origin/base
git -c user.name=ci -c user.email=ci@localhost \
  merge -q --no-edit origin/pr
git log --oneline -3
# The LFS helper the caller downloaded beside this one, from the same branch.
bash "$(dirname "${BASH_SOURCE[0]}")/lfs-fetch.sh" "$repo"
