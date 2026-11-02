#!/usr/bin/env bash
# Checks ci/lint-workflows.sh with a stub curl serving fake actionlint and
# ShellCheck tarballs: it runs actionlint on the workflows with shellcheck
# on PATH, refuses a download whose SHA-256 isn't the pinned one, reuses
# verified downloads, and fails when actionlint finds a problem. It counts
# failures instead of using set -e, so one failed check doesn't hide the rest.
#   ci/lint-workflows-test.sh
# check() evals its quoted assertions later, which shellcheck can't follow.
# shellcheck disable=SC2016
set -uo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
cd "$T" || exit 2

# A repo with two workflows and the config, as the script expects to run in.
mkdir -p repo/ci repo/.gitea/workflows bin srv/shellcheck-v0.10.0
cp "$root/ci/lint-workflows.sh" repo/ci/
git -C repo init -q
git -C repo add ci/lint-workflows.sh
touch repo/ci/untracked.sh
touch repo/ci/actionlint.yaml repo/.gitea/workflows/a.yml repo/.gitea/workflows/b.yml
# The fake actionlint logs its arguments and whether shellcheck is on PATH,
# and exits with the code in $T/lint-exit.
cat > srv/actionlint << 'STUB'
#!/usr/bin/env bash
echo "actionlint $* shellcheck=$(command -v shellcheck > /dev/null && echo yes || echo no)" >> "$STUB_DIR/lint.log"
exit "$(cat "$STUB_DIR/lint-exit")"
STUB
cat > srv/shellcheck-v0.10.0/shellcheck << 'STUB'
#!/usr/bin/env bash
printf 'shellcheck %s\n' "$*" >> "$STUB_DIR/lint.log"
exit "$(cat "$STUB_DIR/shellcheck-exit")"
STUB
chmod +x srv/actionlint srv/shellcheck-v0.10.0/shellcheck
tar -czf srv/al.tar.gz -C srv actionlint
# gzip, not xz, which this test may not have; the script lets tar detect it.
tar -czf srv/sc.tar.xz -C srv shellcheck-v0.10.0
# The stub curl serves srv/<basename of the URL>, logs each request, and
# fails while $T/offline exists.
cat > bin/curl << 'STUB'
#!/usr/bin/env bash
echo "$*" >> "$STUB_DIR/curl.log"
[ ! -e "$STUB_DIR/offline" ] || exit 6
while [ $# -gt 1 ]; do case $1 in -o) out=$2; shift 2 ;; *) shift ;; esac; done
cp "$STUB_DIR/srv/$(basename "$1")" "$out"
STUB
chmod +x bin/curl
export PATH="$T/bin:$PATH" STUB_DIR="$T"
sum() { sha256sum "$1" | cut -d' ' -f1; }
export ACTIONLINT_URL=https://example.test/al.tar.gz SHELLCHECK_URL=https://example.test/sc.tar.xz
ACTIONLINT_SHA256=$(sum srv/al.tar.gz) SHELLCHECK_SHA256=$(sum srv/sc.tar.xz)
export ACTIONLINT_SHA256 SHELLCHECK_SHA256

lint() { : > curl.log; : > lint.log; (cd repo && "$@" ci/lint-workflows.sh "$T/tools") > out.txt 2> err.txt; }
pass=0 failed=0
check() {
  if eval "$2"; then echo "ok   $1"; pass=$((pass + 1))
  else echo "FAIL $1"; sed 's/^/     | out: /' out.txt; sed 's/^/     | err: /' err.txt; failed=$((failed + 1)); fi
}

echo 0 > shellcheck-exit
echo 0 > lint-exit; lint; rc=$?
check "clean workflows: runs actionlint with the config on every workflow, shellcheck on PATH" \
  '[ $rc = 0 ] && grep -qx "actionlint -no-color -config-file ci/actionlint.yaml .gitea/workflows/a.yml .gitea/workflows/b.yml shellcheck=yes" lint.log'
check "clean workflows: says how many it checked" 'grep -q "no problems in 2 workflows" out.txt'

check "standalone lint includes tracked scripts only" 'grep -qx "shellcheck ci/lint-workflows.sh" lint.log'
echo 1 > shellcheck-exit; lint; rc=$?; echo 0 > shellcheck-exit
check "standalone lint failure fails the gate" '[ $rc != 0 ]'

lint; rc=$?
check "tools already downloaded at the pinned sums: no download" '[ $rc = 0 ] && [ ! -s curl.log ] && [ -s lint.log ]'

echo 1 > lint-exit; lint; rc=$?; echo 0 > lint-exit
check "actionlint finds a problem: fails" '[ $rc != 0 ]'

mkdir repo/workflows; touch repo/workflows/t.yml; lint; rc=$?
check "workflow templates are linted after the repo's workflows" \
  '[ $rc = 0 ] && grep -qx "actionlint -no-color -config-file ci/actionlint.yaml .gitea/workflows/a.yml .gitea/workflows/b.yml workflows/t.yml shellcheck=yes" lint.log'
mv repo/.gitea/workflows repo/.gitea/stash; lint; rc=$?
check "templates alone are linted" '[ $rc = 0 ] && grep -qx "actionlint -no-color -config-file ci/actionlint.yaml workflows/t.yml shellcheck=yes" lint.log'
rm -r repo/workflows; lint; rc=$?; mv repo/.gitea/stash repo/.gitea/workflows
check "no workflows at all: fails without running actionlint" '[ $rc != 0 ] && grep -q "No workflows" err.txt && [ ! -s lint.log ]'

rm -rf tools; lint env ACTIONLINT_SHA256=0000000000000000000000000000000000000000000000000000000000000000; rc=$?
check "download with the wrong SHA-256: fails without running it, and removes it" \
  '[ $rc != 0 ] && grep -q "pinned SHA-256" err.txt && [ ! -s lint.log ] && [ ! -e tools/actionlint.tar.gz ]'

rm -rf tools; touch offline; lint
# rc is read by the deferred check assertion below.
# shellcheck disable=SC2034
rc=$?; rm offline
check "download fails: fails without running anything" '[ $rc != 0 ] && [ ! -s lint.log ]'

echo "$pass passed, $failed failed"
[ "$failed" = 0 ]
