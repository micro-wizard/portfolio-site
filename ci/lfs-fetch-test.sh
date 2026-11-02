#!/usr/bin/env bash
# Local Git fixtures and a stub LFS batch API for lfs-fetch.sh.
# No network or credentials; strict failure aborts the test immediately.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
# git-lfs installed globally, as on the runner, with checkouts skipping its
# smudge, as the workflows check out.
if ! command -v git-lfs > /dev/null; then
  echo 'git-lfs is required: the runner image has it, and so must this test' >&2
  exit 1
fi
cat > "$T/gitconfig" <<'CONFIG'
[filter "lfs"]
	process = git-lfs filter-process
	clean = git-lfs clean -- %f
	smudge = git-lfs smudge -- %f
	required = true
CONFIG
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=$T/gitconfig GIT_LFS_SKIP_SMUDGE=1
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@localhost
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@localhost
export TEST_ROOT=$T TOKEN=test-read-only-token
REPO=https://git.example.test/org/repo.git
mkdir "$T/bin" "$T/objects"
# The stub server: requests go to $T/requests, one line each, and each call's
# arguments to $T/curl-args. A batch answers with $T/batch.json, or fails like
# curl -f on an HTTP error when $T/batch-status holds one. A download serves
# $T/objects/<oid>, or fails like that when the object is missing.
cat > "$T/bin/curl" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
echo " $* " >> "$TEST_ROOT/curl-args"
out= url= data= headers=()
while [ $# -gt 0 ]; do
  case $1 in
    -o) out=$2; shift ;;
    -H) headers+=("$2"); shift ;;
    --data-binary) data=${2#@}; shift ;;
    --retry|--retry-delay) shift ;;
    -*) ;;
    *) url=$1 ;;
  esac
  shift
done
auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
if [[ " ${headers[*]} " != *" Authorization: basic $auth "* ]]; then
  echo 'request missing Authorization header' >&2; exit 2
fi
echo "$url" >> "$TEST_ROOT/requests"
case $url in
  */info/lfs/objects/batch)
    cp "$data" "$TEST_ROOT/batch-request.json"
    # What the LFS batch API requires of a download request.
    if ! jq -e '.operation == "download" and (.transfers | index("basic"))
        and (.objects | length > 0 and all(.oid | test("^[0-9a-f]{64}$")) and all(.size | type == "number"))' \
        "$data" > /dev/null; then
      echo 'curl: (22) The requested URL returned error: 422' >&2
      exit 22
    fi
    for h in 'Accept: application/vnd.git-lfs+json' 'Content-Type: application/vnd.git-lfs+json'; do
      if [[ " ${headers[*]} " != *" $h "* ]]; then echo "batch request missing $h" >&2; exit 2; fi
    done
    if [ -s "$TEST_ROOT/batch-status" ]; then
      echo "curl: (22) The requested URL returned error: $(cat "$TEST_ROOT/batch-status")" >&2
      exit 22
    fi
    cp "$TEST_ROOT/batch.json" "$out" ;;
  */info/lfs/objects/*)
    oid=${url##*/}
    if [ ! -f "$TEST_ROOT/objects/$oid" ]; then
      echo 'curl: (22) The requested URL returned error: 404' >&2
      exit 22
    fi
    cp "$TEST_ROOT/objects/$oid" "$out" ;;
  *) echo "unexpected URL $url" >&2; exit 2 ;;
esac
STUB
# mktemp fails in the directory $MKTEMP_FAIL_DIR names, as on a full disk.
export REAL_MKTEMP
REAL_MKTEMP=$(command -v mktemp)
cat > "$T/bin/mktemp" <<'STUB'
#!/usr/bin/env bash
template=${!#}
if [ -n "${MKTEMP_FAIL_DIR:-}" ] && [ "${template%/*}" = "$MKTEMP_FAIL_DIR" ]; then
  echo "mktemp: failed to create file via template '$template': No space left on device" >&2
  exit 1
fi
exec "$REAL_MKTEMP" "$@"
STUB
chmod +x "$T/bin/"*
export PATH="$T/bin:$PATH"

model='glTF binary model bytes'
other='other asset'
oid_of() { printf '%s' "$1" | sha256sum | cut -c1-64; }
pointer_of() {
  printf 'version https://git-lfs.github.com/spec/v1\noid sha256:%s\nsize %s\n' \
    "$(oid_of "$1")" "$(printf '%s' "$1" | wc -c)"
}
MODEL_OID=$(oid_of "$model")
OTHER_OID=$(oid_of "$other")

# Git allows any byte but NUL in a path; this one has a newline, glob
# characters, and ends in a space.
ODD_PATH=$'assets/models/two\nlines [x].glb '
# A checkout as CI makes one: LFS attributes, and pointers where git-lfs
# skipped the smudge. Three paths share the model's object.
fixture() {
  rm -rf "$T/work" "$T/requests" "$T/curl-args" "$T/batch-status" "$T/objects"
  mkdir -p "$T/work/assets/models" "$T/objects"
  cd "$T/work"
  git init -q
  printf 'assets/** filter=lfs diff=lfs merge=lfs -text\n' > .gitattributes
  pointer_of "$model" > assets/models/wizard.glb
  pointer_of "$model" > assets/models/copy.glb
  pointer_of "$model" > "$ODD_PATH"
  pointer_of "$other" > assets/other.bin
  echo 'plain file' > README.md
  git add -A
  git commit -qm fixture
  printf '%s' "$model" > "$T/objects/$MODEL_OID"
  printf '%s' "$other" > "$T/objects/$OTHER_OID"
  # Hrefs on another address than the repository URL, which the helper ignores.
  jq -n --arg m "$MODEL_OID" --arg o "$OTHER_OID" '{transfer: "basic", objects: [$m, $o] | map({
    oid: ., size: 1, authenticated: true,
    actions: {download: {href: "http://10.10.10.228:3001/org/repo.git/info/lfs/objects/\(.)"}}})}' \
    > "$T/batch.json"
}
fetch() {
  (cd "$T/work" && bash "$root/ci/lfs-fetch.sh" "$REPO") > "$T/output" 2>&1
}
expect_failure() {
  if fetch; then echo 'unexpected successful fetch' >&2; cat "$T/output" >&2; exit 1; fi
}

fixture
fetch
cd "$T/work"
test "$(head -c 4 assets/models/wizard.glb)" = glTF
test "$(sha256sum < assets/models/wizard.glb | cut -c1-64)" = "$MODEL_OID"
cmp assets/models/wizard.glb assets/models/copy.glb
cmp assets/models/wizard.glb "$ODD_PATH"
test "$(cat assets/other.bin)" = 'other asset'
test "$(cat README.md)" = 'plain file'
echo 'ok: replaces pointers with objects matching their sha256'

test "$(jq -c '.objects | sort_by(.oid)' "$T/batch-request.json")" \
  = "$(jq -nc --arg m "$MODEL_OID" --arg o "$OTHER_OID" '[{oid: $m, size: 23}, {oid: $o, size: 11}] | sort_by(.oid)')"
test "$(grep -c '/info/lfs/objects/batch$' "$T/requests")" = 1
test "$(grep -c "/info/lfs/objects/$MODEL_OID\$" "$T/requests")" = 1
if grep -q 10.10.10.228 "$T/requests"; then echo 'downloaded from the href' >&2; exit 1; fi
grep -qx "$REPO/info/lfs/objects/$OTHER_OID" "$T/requests"
echo 'ok: asks for each object once and downloads through the repository URL'

# The batch and both downloads, each retried twice, also on a refused connection.
test "$(grep -F -e ' --retry 2 ' "$T/curl-args" | grep -cF -e ' --retry-connrefused ')" = 3
echo 'ok: every request retries, including on a refused connection'

test -z "$(git status --porcelain)"
git add -A
test -z "$(git diff --cached --name-only HEAD)"
echo 'ok: fetched files read as the committed pointers'

# A top-level name that reads as pathspec magic (exclude) unless taken literally.
fixture
printf '*.magic filter=lfs diff=lfs merge=lfs -text\n' >> .gitattributes
pointer_of "$model" > ':!a.magic'
git add -A && git commit -qm 'magic name'
fetch
test "$(head -c 4 ':!a.magic')" = glTF
test -z "$(git status --porcelain)"
echo 'ok: a fetched path that looks like pathspec magic reads as unchanged'

# Top-level names that awk reads as an assignment, and other tools as an option.
fixture
printf '*.glb filter=lfs diff=lfs merge=lfs -text\n' >> .gitattributes
pointer_of "$model" > 'model=copy.glb'
pointer_of "$model" > '-dash.glb'
git add -A && git commit -qm 'awkward names'
fetch
test "$(head -c 4 'model=copy.glb')" = glTF
test "$(head -c 4 ./-dash.glb)" = glTF
test -z "$(git status --porcelain)"
echo 'ok: fetches paths that look like assignments or options'

# Two pointers to one object, one with the wrong size.
fixture
printf 'version https://git-lfs.github.com/spec/v1\noid sha256:%s\nsize 999\n' "$MODEL_OID" > assets/models/zz-wrong-size.glb
git add -A && git commit -qm 'wrong size'
expect_failure
grep -qx "::error::Couldn't fetch the Git LFS object for assets/models/zz-wrong-size.glb: the download doesn't match the pointer's size and sha256" "$T/output"
grep -q "^size 999$" assets/models/zz-wrong-size.glb
test "$(head -c 4 assets/models/wizard.glb)" = glTF
echo "ok: a pointer whose size doesn't match an object shared with another pointer fails"

auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
if grep -Fq -e "$TOKEN" -e "$auth" .git/config "$T/output"; then
  echo 'credentials leaked to config or logs' >&2; exit 1
fi
echo 'ok: leaves no credentials in Git config or logs'

fixture
echo 'outside the checkout' > "$T/outside"
ln -s "$T/outside" assets/models/.lfs-fetch
echo 'committed file' > assets/.lfs-fetch
git add -A && git commit -qm 'files where a fixed temporary name would go'
fetch
test "$(cat "$T/outside")" = 'outside the checkout'
test "$(cat assets/.lfs-fetch)" = 'committed file'
test -L assets/models/.lfs-fetch
test "$(head -c 4 assets/models/wizard.glb)" = glTF
test "$(cat assets/other.bin)" = 'other asset'
echo 'ok: replaces pointers without touching neighbouring files or symlink targets'

: > "$T/requests"
fetch
test ! -s "$T/requests"
echo 'ok: a checkout already fetched makes no requests'

fixture
git rm -q --cached -r assets
rm -r assets
git commit -qm 'no LFS files'
fetch
test ! -e "$T/requests"
echo 'ok: a checkout with no pointers makes no requests'

for status in 401 404; do
  fixture
  echo "$status" > "$T/batch-status"
  expect_failure
  grep -q "^::error::Couldn't fetch the Git LFS object for assets/models/wizard.glb: batch request failed: .*$status" "$T/output"
  test "$(cat assets/models/wizard.glb)" = "$(pointer_of "$model")"
  echo "ok: a batch $status fails naming each file and keeps the pointers"
done

fixture
rm "$T/objects/$OTHER_OID"
expect_failure
grep -q "^::error::Couldn't fetch the Git LFS object for assets/other.bin: download failed: .*404" "$T/output"
test "$(cat assets/other.bin)" = "$(pointer_of "$other")"
test "$(head -c 4 assets/models/wizard.glb)" = glTF
echo 'ok: a download 404 fails naming the file and keeps its pointer'

fixture
jq --arg o "$OTHER_OID" '.objects |= map(if .oid == $o then del(.actions) + {error: {code: 404, message: "Not found"}} else . end)' \
  "$T/batch.json" > "$T/batch.tmp" && mv "$T/batch.tmp" "$T/batch.json"
expect_failure
grep -qx "::error::Couldn't fetch the Git LFS object for assets/other.bin: the server refused it: Not found (404)" "$T/output"
test "$(cat assets/other.bin)" = "$(pointer_of "$other")"
echo 'ok: an object error in the batch response fails naming the file'

fixture
jq --arg o "$OTHER_OID" '.objects |= map(select(.oid != $o))' "$T/batch.json" > "$T/batch.tmp" && mv "$T/batch.tmp" "$T/batch.json"
expect_failure
grep -qx "::error::Couldn't fetch the Git LFS object for assets/other.bin: the batch response has no entry for it" "$T/output"
test "$(cat assets/other.bin)" = "$(pointer_of "$other")"
echo 'ok: an object missing from the batch response fails naming the file'

fixture
printf 'tampered' > "$T/objects/$MODEL_OID"
expect_failure
grep -qx "::error::Couldn't fetch the Git LFS object for assets/models/wizard.glb: the download doesn't match the pointer's size and sha256" "$T/output"
test "$(cat assets/models/wizard.glb)" = "$(pointer_of "$model")"
echo 'ok: an object not matching its pointer fails and keeps the pointer'

# The model's size with one byte changed, so only the sha256 check can catch it.
fixture
printf 'glTF binary model bytez' > "$T/objects/$MODEL_OID"
expect_failure
grep -qx "::error::Couldn't fetch the Git LFS object for assets/models/wizard.glb: the download doesn't match the pointer's size and sha256" "$T/output"
test "$(cat assets/models/wizard.glb)" = "$(pointer_of "$model")"
echo 'ok: an object of the right size but wrong sha256 fails and keeps the pointer'

# An href off the repository, such as object storage: the stub serves no such
# URL, so only a download through the repository URL succeeds.
fixture
jq '.objects[0].actions.download.href = "https://storage.example.test/bucket/object"' \
  "$T/batch.json" > "$T/batch.tmp" && mv "$T/batch.tmp" "$T/batch.json"
fetch
test "$(head -c 4 assets/models/wizard.glb)" = glTF
if grep -q storage.example.test "$T/requests"; then echo 'sent the token to the href' >&2; exit 1; fi
echo 'ok: downloads through the repository URL whatever the href says'

# A name of 255 bytes, the most a filename can have, leaves no room for a
# temporary name built on it.
fixture
long=assets/$(printf 'x%.0s' {1..255})
pointer_of "$other" > "$long"
git add -A && git commit -qm 'long name'
fetch
test "$(cat "$long")" = 'other asset'
test -z "$(git status --porcelain)"
echo 'ok: fetches a file whose name is as long as a filename can be'

# No room beside assets/other.bin, which sorts after the models: those must
# still read as unchanged.
fixture
MKTEMP_FAIL_DIR=./assets expect_failure
grep -qx "::error::Couldn't fetch the Git LFS object for assets/other.bin: couldn't write it in place of its pointer" "$T/output"
test "$(cat assets/other.bin)" = "$(pointer_of "$other")"
test "$(head -c 4 assets/models/wizard.glb)" = glTF
test -z "$(git status --porcelain)"
echo "ok: a pointer that can't be replaced fails naming it, and fetched files read as unchanged"

# An index git can't read, so the files can't be listed.
fixture
printf 'not an index' > .git/index
expect_failure
grep -q '^fatal: .*index' "$T/output"
test "$(cat assets/models/wizard.glb)" = "$(pointer_of "$model")"
test ! -e "$T/requests"
echo "ok: a checkout whose files git can't list fails"
