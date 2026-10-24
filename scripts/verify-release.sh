#!/usr/bin/env bash
# §9 / U22: verify the actual staged artifact and updater without a service or network.
set -euo pipefail
stage=$(realpath "${1:?usage: verify-release.sh DIST VERSION}")
version=${2:?usage: verify-release.sh DIST VERSION}
name="pilotq-$version-linux-x86_64"
(cd "$stage" && sha256sum -c SHA256SUMS)
python3 - "$stage/latest.json" "$version" "$name" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    manifest = json.load(f)
assert manifest['version'] == sys.argv[2]
assert manifest['files'] == {'linux-x86_64': sys.argv[3]}
PY
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/home" "$work/site"
cp "$stage/"* "$work/site/"
cp "$stage/$name" "$work/candidate"
chmod +x "$work/candidate"
export HOME="$work/home" PILOTQ_INSTALL="$work/install" PILOTQ_SELFTEST_OFFLINE=1
export RELEASE_SITE="$work/site"
# coreq v0.14.6 download arguments: -fsSL --max-time 300 -o DEST URL.
cat > "$work/bin/curl" <<'CURL'
#!/bin/sh
set -eu
[ "$#" = 6 ] && [ "$4" = '-o' ]
cp "$RELEASE_SITE/${6##*/}" "$5"
CURL
chmod +x "$work/bin/curl"
export PATH="$work/bin:$PATH"
"$work/candidate" selftest
# An empty install running the latest binary sees the manifest and needs no update.
"$work/candidate" update --check | grep -Fx 'update: up to date'
test ! -e "$PILOTQ_INSTALL/current"
# A bootstrap selection permits exercising a newer release with the exact same binary.
printf '0.0.0' > "$PILOTQ_INSTALL/current"
"$work/candidate" update --check | grep -Fx "update: $version available"
test "$(cat "$PILOTQ_INSTALL/current")" = '0.0.0'
# A corrupt payload must fail checksum validation and leave the selection untouched.
printf 'corrupt fixture' >> "$RELEASE_SITE/$name"
if "$work/candidate" update > "$work/rejected" 2>&1; then
    echo 'corrupt artifact was accepted' >&2
    exit 1
fi
grep -F 'not in SHA256SUMS' "$work/rejected"
test "$(cat "$PILOTQ_INSTALL/current")" = '0.0.0'
test ! -e "$PILOTQ_INSTALL/pilotq"
cp "$stage/$name" "$RELEASE_SITE/$name"
"$work/candidate" update | grep -Fx "update: $version"
test "$(cat "$PILOTQ_INSTALL/current")" = "$version"
test "$(cat "$PILOTQ_INSTALL/previous")" = '0.0.0'
cmp "$stage/$name" "$PILOTQ_INSTALL/releases/$version/pilotq"
"$PILOTQ_INSTALL/pilotq" selftest
"$PILOTQ_INSTALL/pilotq" update --check | grep -Fx 'update: up to date'
echo 'release artifact and isolated updater: ok'
