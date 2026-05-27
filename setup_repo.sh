#!/usr/bin/env bash
set -euo pipefail

OLD_OWNER_NAME="isaac-cf-wong"
OLD_REPO_NAME="rust-package-template"
OLD_REPO_NAME_NORMALIZED="${OLD_REPO_NAME//-/_}"

url=$(git config --get remote.origin.url)

if [[ "$url" =~ [:/]([^/]+)/([^/]+)(\.git)?$ ]]; then
    NEW_OWNER_NAME="${BASH_REMATCH[1]}"
    NEW_REPO_NAME="${BASH_REMATCH[2]%.git}"
else
    echo "Unable to parse owner/repo from remote.origin.url: $url" >&2
    exit 1
fi

NEW_REPO_NAME_NORMALIZED="${NEW_REPO_NAME//-/_}"

[[ -f "Cargo.toml" ]] || {
    echo "Missing Cargo.toml" >&2
    exit 1
}
[[ -f "cliff.toml" ]] || {
    echo "Missing cliff.toml" >&2
    exit 1
}

if sed --version >/dev/null 2>&1; then
    SED_INPLACE=(-i)
else
    SED_INPLACE=(-i '')
fi

OLD_REPO_URL="https://github.com/$OLD_OWNER_NAME/$OLD_REPO_NAME"
NEW_REPO_URL="https://github.com/$NEW_OWNER_NAME/$NEW_REPO_NAME"

find . -type d \( -name ".git" -o -name "target" -o -name "node_modules" -o -name "site" \) -prune \
    -o -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" -o -name "*.yml" -o -name "*.yaml" -o -name "*.json" -o -name "LICENSE" -o -name "CITATION.cff" \) \
    -exec sed "${SED_INPLACE[@]}" \
        -e "s#${OLD_REPO_URL}#${NEW_REPO_URL}#g" \
        -e "s/${OLD_OWNER_NAME}/${NEW_OWNER_NAME}/g" \
        -e "s/${OLD_REPO_NAME}/${NEW_REPO_NAME}/g" \
        -e "s/${OLD_REPO_NAME_NORMALIZED}/${NEW_REPO_NAME_NORMALIZED}/g" {} +

cargo generate-lockfile

echo "Repository customized for $NEW_OWNER_NAME/$NEW_REPO_NAME"
