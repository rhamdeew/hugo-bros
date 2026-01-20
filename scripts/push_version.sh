#!/usr/bin/env bash
set -euo pipefail

version="$(
  python3 - <<'PY'
import json
with open("package.json", "r", encoding="utf-8") as f:
    data = json.load(f)
print(data.get("version", ""))
PY
)"

if [[ -z "${version}" ]]; then
  echo "Could not determine version from package.json" >&2
  exit 1
fi

read -r -p "Create and push tag v${version}? [y/N]: " confirm
confirm="${confirm:-N}"

if [[ "${confirm}" != "y" && "${confirm}" != "Y" ]]; then
  echo "Canceled."
  exit 0
fi

git tag "v${version}"
git push origin "v${version}"
