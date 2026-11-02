#!/usr/bin/env bash
# A client repo's review job: ask the reviewer service for a review of this
# PR's head, and wait for it (reviewer/README.md). Load it from the default
# branch only.
# Inputs: TOKEN (REVIEW_DISPATCH_TOKEN: Actions write on the reviewer repo,
#   read here), REVIEWER_API (<api>/repos/<owner>/reviewer), API (this repo's
#   API), REPOSITORY, PR, HEAD_SHA, REVIEW_BOT_LOGIN (the account reviews are
#   posted as), GITHUB_OUTPUT; optional REVIEW_TIMEOUT (seconds, default
#   1200) and POLL_INTERVAL (seconds, default 20).
# Outputs blocking/findings like pr-review.sh publish. Succeeds on a review of
# HEAD_SHA posted after the dispatch, fails on a review-failed marker for it
# or on the timeout. Only the bot's comments count: anyone can comment.
set -euo pipefail
: "${TOKEN:?}" "${REVIEWER_API:?}" "${API:?}" "${REPOSITORY:?}" "${PR:?}" "${HEAD_SHA:?}"
: "${REVIEW_BOT_LOGIN:?set the REVIEW_BOT_LOGIN variable}" "${GITHUB_OUTPUT:?}"
[[ "$PR" =~ ^[0-9]+$ ]] || { echo 'Invalid PR number' >&2; exit 1; }
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]] || { echo 'Invalid head SHA' >&2; exit 1; }
timeout=${REVIEW_TIMEOUT:-1200} interval=${POLL_INTERVAL:-20}
auth="Authorization: token $TOKEN"
comments() {
  curl -fsS --retry 2 --retry-delay 5 --retry-all-errors -H "$auth" "$API/issues/$PR/comments"
}
# Comment IDs only grow, so anything newer than this answers this request,
# not an earlier run's (a re-run of the same head must get a new review).
# Not paginated: Gitea returns every issue comment.
last=$(comments | jq -e '[.[].id] | max // 0')
ref=$(curl -fsS --retry 2 --retry-delay 5 --retry-all-errors -H "$auth" "$REVIEWER_API" | jq -er .default_branch)
jq -n --arg ref "$ref" --arg repo "$REPOSITORY" --arg pr "$PR" \
    '{ref: $ref, inputs: {repo: $repo, pr: $pr}}' \
  | curl -fsS -X POST -H "$auth" -H 'Content-Type: application/json' -d @- \
    "$REVIEWER_API/actions/workflows/review.yml/dispatches" > /dev/null
echo "Asked for a review of $HEAD_SHA; waiting up to $timeout seconds."
deadline=$(( $(date +%s) + timeout ))
review=$(mktemp)
trap 'rm -f "$review"' EXIT
while :; do
  sleep "$interval"
  # A failed poll is retried until the deadline, like a missing answer.
  if body=$(comments | jq -er --argjson last "$last" --arg bot "$REVIEW_BOT_LOGIN" --arg sha "$HEAD_SHA" '
      [.[] | select(.id > $last and .user.login == $bot) | .body
        | select((startswith("<!-- claude-review -->\n") and
              (split("\n")[1] | test("^### (Claude|Codex)( \\([^()]+\\))? review [0-9]+ of " + $sha + "$")))
            or startswith("<!-- review-failed {\"sha\":\"" + $sha + "\","))]
      | first // empty'); then
    printf '%s\n' "$body" > "$review"
    break
  fi
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "::error::No review of $HEAD_SHA after $timeout seconds; see the reviewer repo's runs."
    exit 1
  fi
done
# Model output must not be interpreted as runner annotation/masking commands.
command_token=$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')
echo "::stop-commands::$command_token"
cat "$review"
echo "::$command_token::"
if head -n 1 "$review" | grep -q '^<!-- review-failed '; then
  echo "::error::The reviewer service couldn't review $HEAD_SHA."
  exit 1
fi
# Same parsing as pr-review.sh publish: only digits, no line means unknown.
count() {
  grep -E "^[[:space:]]*\\**$1: *[0-9]+\\**[[:space:]]*\$" "$review" \
    | tail -n 1 | grep -oE '[0-9]+' || true
}
blocking=$(count blocking)
findings=$(count findings)
echo "blocking=$blocking" >> "$GITHUB_OUTPUT"
echo "findings=$findings" >> "$GITHUB_OUTPUT"
echo "Findings: ${findings:-unknown}, blocking: ${blocking:-unknown}"
