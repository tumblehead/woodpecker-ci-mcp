#!/usr/bin/env bash
# Upload a file as an asset of the Gitea release for $CI_COMMIT_TAG, creating
# the release if needed. Any failed Gitea request prints the HTTP status and
# response body and fails the step.
#
# Usage: upload-release-asset.sh <file>
# Env:   GITEA_TOKEN, CI_FORGE_URL, CI_REPO, CI_COMMIT_TAG
set -euo pipefail

file=$1
name=$(basename "$file")
api="$CI_FORGE_URL/api/v1/repos/$CI_REPO/releases"
body=$(mktemp)
trap 'rm -f "$body"' EXIT

# request METHOD URL [curl args...]: prints the HTTP status, saves the body to $body
request() {
    curl -sS -o "$body" -w '%{http_code}' -X "$1" "$2" \
        -H "Authorization: token $GITEA_TOKEN" "${@:3}"
}

fail() {
    echo "error: $1 failed with HTTP $2: $(cat "$body")" >&2
    exit 1
}

# The builds run in parallel, so another platform may have created it already (409)
status=$(request POST "$api" -H "Content-Type: application/json" \
    -d "{\"tag_name\":\"$CI_COMMIT_TAG\",\"name\":\"$CI_COMMIT_TAG\"}")
case $status in
    201) echo "Created release $CI_COMMIT_TAG" ;;
    409) echo "Release $CI_COMMIT_TAG already exists" ;;
    *) fail "creating release $CI_COMMIT_TAG" "$status" ;;
esac

status=$(request GET "$api/tags/$CI_COMMIT_TAG")
[ "$status" = 200 ] || fail "looking up release $CI_COMMIT_TAG" "$status"
# The release object's own id is the first "id" in the response
release_id=$(grep -oE '"id": *[0-9]+' "$body" | head -1 | grep -oE '[0-9]+$' || true)
[ -n "$release_id" ] || fail "reading the release id" "$status"

status=$(request POST "$api/$release_id/assets?name=$name" -F "attachment=@$file")
[ "$status" = 201 ] || fail "uploading $name" "$status"
echo "Uploaded $name to release $CI_COMMIT_TAG"
