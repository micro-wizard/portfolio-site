#!/usr/bin/env bash
# Lints .gitea/workflows/*.yml and the templates in workflows/*.yml and
# reviewer/*.yml with actionlint, which also shellchecks every
# run: block (#47), and all tracked ci/*.sh scripts (#109). The runner image has neither tool, so this downloads
# pinned releases and checks their SHA-256 before running them.
#   ci/lint-workflows.sh [tools-dir]
# Run from the repo root. tools-dir defaults to $RUNNER_TEMP/lint-tools (or a
# temp dir); a dir that already holds both tools at the pinned sums is reused.
# The URLs and sums can be overridden through the environment, for the test.
set -euo pipefail

ACTIONLINT_URL=${ACTIONLINT_URL:-https://github.com/rhysd/actionlint/releases/download/v1.7.7/actionlint_1.7.7_linux_amd64.tar.gz}
ACTIONLINT_SHA256=${ACTIONLINT_SHA256:-023070a287cd8cccd71515fedc843f1985bf96c436b7effaecce67290e7e0757}
SHELLCHECK_URL=${SHELLCHECK_URL:-https://github.com/koalaman/shellcheck/releases/download/v0.10.0/shellcheck-v0.10.0.linux.x86_64.tar.xz}
SHELLCHECK_SHA256=${SHELLCHECK_SHA256:-6c881ab0698e4e6ea235245f22832860544f17ba386442fe7e9d629f8cbedf87}

tools=${1:-${RUNNER_TEMP:-$(mktemp -d)}/lint-tools}
mkdir -p "$tools"

# fetch <url> <sha256> <archive-name>: downloads into $tools unless a copy
# with the right sum is already there; fails on a wrong sum.
fetch() {
  local url=$1 sum=$2 file=$tools/$3
  if [ -f "$file" ] && sha256sum -c --status <<< "$sum  $file"; then
    return
  fi
  curl -fsSL --retry 3 --retry-delay 5 --retry-all-errors -o "$file" "$url"
  if ! sha256sum -c --status <<< "$sum  $file"; then
    echo "::error::$url doesn't have the pinned SHA-256 $sum; not running it." >&2
    rm -f "$file"
    exit 1
  fi
}
fetch "$ACTIONLINT_URL" "$ACTIONLINT_SHA256" actionlint.tar.gz
fetch "$SHELLCHECK_URL" "$SHELLCHECK_SHA256" shellcheck.tar.xz
tar -xf "$tools/actionlint.tar.gz" -C "$tools" actionlint
tar -xf "$tools/shellcheck.tar.xz" -C "$tools" --strip-components=1 --wildcards '*/shellcheck'

shopt -s nullglob
workflows=(.gitea/workflows/*.yml workflows/*.yml reviewer/*.yml)
shopt -u nullglob
if [ "${#workflows[@]}" = 0 ]; then
  echo "::error::No workflows in .gitea/workflows/ or workflows/ to lint." >&2
  exit 1
fi
PATH="$tools:$PATH" actionlint -no-color -config-file ci/actionlint.yaml "${workflows[@]}"
echo "actionlint: no problems in ${#workflows[@]} workflows."

# NUL-separated tracked paths exclude local scratch scripts and preserve spaces.
mapfile -d '' -t scripts < <(git ls-files -z -- 'ci/*.sh')
"$tools/shellcheck" "${scripts[@]}"
echo "shellcheck: no problems in ${#scripts[@]} scripts."
