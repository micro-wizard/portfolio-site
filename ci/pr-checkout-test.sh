#!/usr/bin/env bash
# Local Git fixtures and controlled fetch failures for pr-checkout.sh.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@localhost
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@localhost
export REAL_GIT
REAL_GIT=$(command -v git)
mkdir "$T/bin"
cat > "$T/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ " $* " == *' fetch '* ]]; then
  expected_auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
  if [ "$1" != -c ] || [ "$2" != "http.extraHeader=Authorization: basic $expected_auth" ]; then
    echo 'fetch missing command-scoped Authorization header' >&2
    exit 1
  fi
  echo fetch >> "$FETCH_LOG"
  n=$(cat "$FAIL_FETCHES")
  if [ "$n" -gt 0 ]; then echo $((n - 1)) > "$FAIL_FETCHES"; exit 1; fi
fi
exec "$REAL_GIT" "$@"
STUB
printf '#!/usr/bin/env bash\nexit 0\n' > "$T/bin/sleep"
# lfs-fetch.sh has its own tests; this stub LFS server refuses everything, to
# show the checkout runs it and fails with it.
cat > "$T/bin/curl" <<'STUB'
#!/usr/bin/env bash
echo curl >> "$CURL_LOG"
echo 'curl: (22) The requested URL returned error: 401' >&2
exit 22
STUB
chmod +x "$T/bin/"*
export PATH="$T/bin:$PATH" FETCH_LOG="$T/fetch.log" FAIL_FETCHES="$T/fail-fetches" CURL_LOG="$T/curl.log"
export TOKEN=test-read-only-token BASE_REF=master PR_NUMBER=1

fixture() {
  cd "$T"
  rm -rf "$T/remote" "$T/work"
  git init -q -b master "$T/remote"
  cd "$T/remote"
  echo original > shared
  git add shared
  git commit -qm initial
  git checkout -qb feature
  echo feature > feature
  git add feature
  git commit -qm feature
  git update-ref refs/pull/1/head HEAD
  git checkout -q master
  echo base > base
  git add base
  git commit -qm base
  mkdir "$T/work"
  : > "$FETCH_LOG"
  : > "$CURL_LOG"
  echo 0 > "$FAIL_FETCHES"
}
checkout() {
  (cd "$T/work" && bash "$root/ci/pr-checkout.sh" "$T/remote") > "$T/output" 2>&1
}

fixture
checkout
cd "$T/work"
test -f base
test -f feature
test "$(git rev-list --parents -n 1 HEAD | wc -w)" = 3
if git config --local --get-regexp 'http\..*extraheader|credential\.'; then
  echo 'credentials persisted in Git config' >&2; exit 1
fi
auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
if grep -Fq -e "$TOKEN" -e "$auth" .git/config "$T/output"; then
  echo 'credentials leaked to config or logs' >&2; exit 1
fi
test ! -s "$CURL_LOG"
echo 'ok: merges current base and PR without persisting credentials'
echo 'ok: makes no LFS requests without LFS files'

# Reuse the checkout after both refs are rewritten, including a non-fast-forward PR.
cd "$T/remote"
git checkout -q feature
git reset -q --hard master
echo replacement > replacement
git add replacement && git commit -qm replacement
git update-ref refs/pull/1/head HEAD
git checkout -q master
echo moved > moved
git add moved && git commit -qm moved
checkout
cd "$T/work"
test -f moved
test -f replacement
test ! -f feature
echo 'ok: refetches moved base and rewritten PR refs'

fixture
echo 2 > "$FAIL_FETCHES"
checkout
test "$(wc -l < "$FETCH_LOG")" = 3
echo 'ok: retries transient fetch failures'

fixture
echo 3 > "$FAIL_FETCHES"
if checkout; then echo 'unexpected successful fetch' >&2; exit 1; fi
test "$(wc -l < "$FETCH_LOG")" = 3
if git -C "$T/work" rev-parse --verify HEAD 2>/dev/null; then
  echo 'checkout continued after failed fetches' >&2; exit 1
fi
echo 'ok: exhausted retries fail before checkout'

fixture
cd "$T/remote"
echo master > shared
git add shared && git commit -qm master-conflict
git checkout -q feature
echo feature > shared
git add shared && git commit -qm feature-conflict
git update-ref refs/pull/1/head HEAD
if checkout; then echo 'unexpected successful merge' >&2; exit 1; fi
test -n "$(git -C "$T/work" ls-files -u)"
echo 'ok: merge conflicts fail the checkout'

fixture
cd "$T/remote"
git checkout -q feature
printf '*.glb filter=lfs diff=lfs merge=lfs -text\n' > .gitattributes
printf 'version https://git-lfs.github.com/spec/v1\noid sha256:%064d\nsize 4\n' 0 > model.glb
git add .gitattributes model.glb && git commit -qm model
git update-ref refs/pull/1/head HEAD
git checkout -q master
if checkout; then echo 'unexpected successful LFS fetch' >&2; exit 1; fi
grep -q "^::error::Couldn't fetch the Git LFS object for model.glb" "$T/output"
test -s "$CURL_LOG"
echo 'ok: an LFS object the server refuses fails the checkout'

# The runner image's git-lfs, installed globally: in a checkout, which has
# no credentials, it can't download the PR's object (job 3668 on #148).
if ! command -v git-lfs > /dev/null; then
  echo 'git-lfs is required: the runner image has it, and so must this test' >&2
  exit 1
fi
cat > "$T/lfs-gitconfig" <<'CONFIG'
[filter "lfs"]
	process = git-lfs filter-process
	clean = git-lfs clean -- %f
	smudge = git-lfs smudge -- %f
	required = true
CONFIG
rm -rf "$T/work" && mkdir "$T/work" && : > "$CURL_LOG"
# The workflows set GIT_LFS_SKIP_SMUDGE, so a CI run of this test has it too.
if (unset GIT_LFS_SKIP_SMUDGE; GIT_CONFIG_GLOBAL="$T/lfs-gitconfig" checkout); then
  echo 'git-lfs smudged without an object' >&2; exit 1
fi
grep -q 'smudge filter lfs failed' "$T/output"
echo "ok: with git-lfs's smudge, an LFS file fails the merge (the workflows skip it)"

rm -rf "$T/work" && mkdir "$T/work" && : > "$CURL_LOG"
if GIT_CONFIG_GLOBAL="$T/lfs-gitconfig" GIT_LFS_SKIP_SMUDGE=1 checkout; then
  echo 'unexpected successful LFS fetch' >&2; exit 1
fi
if grep -q 'smudge filter' "$T/output"; then echo 'git-lfs smudged' >&2; exit 1; fi
grep -q "^::error::Couldn't fetch the Git LFS object for model.glb" "$T/output"
echo "ok: skipping git-lfs's smudge leaves the object to lfs-fetch.sh"
