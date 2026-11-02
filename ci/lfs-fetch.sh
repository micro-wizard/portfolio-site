#!/usr/bin/env bash
# Replace the Git LFS pointer files in the current checkout with their objects,
# through the server's LFS batch API, with curl and jq. The runner image's
# git-lfs would fetch with the remote's credentials, which a CI checkout
# doesn't keep, so its checkouts skip the smudge (GIT_LFS_SKIP_SMUDGE=1) and
# leave pointers for this helper.
# Usage: ci/lfs-fetch.sh <repository-url>   (in the work tree)
# Inputs: TOKEN (read-only repository access), passed only to the curl calls,
# as ci/pr-checkout.sh does for git fetch.
# Outputs: the real files in place of their pointers. With git-lfs's clean
# filter installed, as on the runner, `git status` reads them as their
# committed pointers. Any batch or download failure, or an object that doesn't
# match its pointer, logs ::error:: naming the file and exits nonzero, leaving
# that file's pointer. A checkout with no pointers makes no requests.
# Rollout: run the default branch's copy (or the run's own commit's), never a
# PR's, because it holds the token.
set -euo pipefail

repo=${1:?repository URL required}
: "${TOKEN:?read-only token required}"
repo=${repo%/}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Prints "<oid> <size>" if the file is a pointer (spec v1: under 1024 bytes,
# version first, then sorted keys including oid and size). Read on stdin:
# awk takes a file operand like a=b for an assignment.
pointer() {
  [ "$(wc -c < "$1")" -lt 1024 ] || return 1
  awk 'NR == 1 && $0 != "version https://git-lfs.github.com/spec/v1" { bad = 1; exit }
    $1 == "oid" && $2 ~ /^sha256:[0-9a-f]+$/ && length($2) == 71 { oid = substr($2, 8) }
    $1 == "size" && $2 ~ /^[0-9]+$/ { size = $2 }
    END { if (bad || oid == "" || size == "") exit 1; print oid, size }' < "$1"
}

# Tracked files with filter=lfs that still hold a pointer. Arrays, not lines,
# because a path may hold any byte but NUL. Paths given to file tools start
# with ./, so one starting with - isn't read as an option. The list goes
# through a file rather than process substitution, so a git failure fails
# the script instead of looking like a checkout with no pointers.
git ls-files -z | git check-attr --stdin -z filter > "$tmp/attributes"
oids=() sizes=() paths=()
while IFS= read -r -d '' path && IFS= read -r -d '' _ && IFS= read -r -d '' value; do
  if [ "$value" = lfs ] && [ -f "./$path" ] && [ ! -L "./$path" ] && ptr=$(pointer "./$path"); then
    oids+=("${ptr% *}") sizes+=("${ptr#* }") paths+=("$path")
  fi
done < "$tmp/attributes"
[ "${#paths[@]}" -gt 0 ] || exit 0

failed=0
fetched=()
fail() {
  echo "::error::Couldn't fetch the Git LFS object for $1: $2"
  failed=1
}

auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
# Each object once, however many paths share it.
# Oids are hex and sizes digits, so lines are safe for them.
for i in "${!oids[@]}"; do printf '%s %s\n' "${oids[i]}" "${sizes[i]}"; done \
  | jq -Rn '{operation: "download", transfers: ["basic"],
      objects: [inputs | split(" ") | {oid: .[0], size: (.[1] | tonumber)}] | unique_by(.oid)}' \
  > "$tmp/request.json"
if ! curl -fsS --retry 2 --retry-delay 5 --retry-connrefused \
    -H "Authorization: basic $auth" \
    -H 'Accept: application/vnd.git-lfs+json' \
    -H 'Content-Type: application/vnd.git-lfs+json' \
    --data-binary "@$tmp/request.json" \
    -o "$tmp/response.json" "$repo/info/lfs/objects/batch" 2> "$tmp/curl.err"; then
  reason=$(head -c 300 "$tmp/curl.err")
  for path in "${paths[@]}"; do fail "$path" "batch request failed: $reason"; done
  exit 1
fi

for i in "${!paths[@]}"; do
  oid=${oids[i]} size=${sizes[i]} path=${paths[i]}
  object=$(jq -c --arg oid "$oid" '[.objects[]? | select(.oid == $oid)][0] // empty' "$tmp/response.json" 2> /dev/null || true)
  if [ -z "$object" ]; then
    fail "$path" "the batch response has no entry for it"
    continue
  fi
  if error=$(jq -er '.error | select(.) | "\(.message // "no message") (\(.code // "no code"))"' <<< "$object"); then
    fail "$path" "the server refused it: $error"
    continue
  fi
  # Built from the URL the checkout fetched from, never taken from the
  # response's href, so the token goes only to this server and object.
  url="$repo/info/lfs/objects/$oid"
  if [ ! -f "$tmp/$oid" ]; then
    if ! curl -fsS --retry 2 --retry-delay 5 --retry-connrefused -H "Authorization: basic $auth" \
        -o "$tmp/$oid.part" "$url" 2> "$tmp/curl.err"; then
      fail "$path" "download failed: $(head -c 300 "$tmp/curl.err")"
      continue
    fi
    if [ "$(sha256sum < "$tmp/$oid.part" | cut -c1-64)" != "$oid" ]; then
      fail "$path" "the download doesn't match the pointer's size and sha256"
      continue
    fi
    mv "$tmp/$oid.part" "$tmp/$oid"
  fi
  # Checked for every pointer: two may share an oid but not agree on its size.
  if [ "$(wc -c < "$tmp/$oid" | tr -d ' ')" != "$size" ]; then
    fail "$path" "the download doesn't match the pointer's size and sha256"
    continue
  fi
  # Copied into the file's directory first, so the pointer is replaced in one
  # rename. mktemp makes a new file, so a committed file or symlink can't be
  # the target, and its short name fits beside a name of any length. It runs
  # inside the if so that, under set -e, its failure is reported like the
  # others instead of skipping the index refresh.
  file=./$path
  if ! { part=$(mktemp "${file%/*}/.lfs-fetch.XXXXXX") && cp "$tmp/$oid" "$part" &&
    chmod --reference="$file" "$part" && mv "$part" "$file"; }; then
    [ -z "$part" ] || rm -f "$part"
    fail "$path" "couldn't write it in place of its pointer"
    continue
  fi
  fetched+=("$path")
  echo "Fetched $path"
done

# The index still has each pointer's size, and git takes a different size as
# a change without running the clean filter. Re-adding the same entries clears
# their stat data, so the refresh compares the cleaned content instead.
if [ "${#fetched[@]}" -gt 0 ]; then
  git --literal-pathspecs ls-files -s -z -- "${fetched[@]}" | git update-index -z --index-info
  git update-index -q --refresh > /dev/null || true
fi
exit "$failed"
