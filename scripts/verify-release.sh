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
echo 'latest.json: ok'
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
if [ "$1" = --version ]; then echo "curl 7.68.0"; exit 0; fi
[ "$#" = 6 ] && [ "$4" = '-o' ]
cp "$RELEASE_SITE/${6##*/}" "$5"
CURL
chmod +x "$work/bin/curl"
# Invented oldest-supported host tools keep updater verification independent of
# runner provisioning. These fixtures never start a service or call a real tool.
python3 - "$work" <<'PYFIXTURE'
import hashlib, json, pathlib, shlex, sys
work = pathlib.Path(sys.argv[1])
bin = work / 'bin'
for name, args, reply in [
    ('codex', ['--version'], 'codex-cli 0.162.0'),
    ('taskq', ['service', 'status', '--json'], '{"service":{"version":"4.6.40"}}'),
    ('gh', ['--version'], 'gh version 2.45.0 (invented)'),
    ('ffmpeg', ['-version'], 'ffmpeg version 6.1.0'),
    ('recognizer', ['--version'], '2.0.1'),
    ('systemctl', ['--version'], 'systemd 245'),
    ('kill', ['--version'], 'kill from procps-ng 3.3.17'),
    ('hostname', ['--version'], 'hostname 3.23'),
]:
    tool = bin / name
    tool.write_text('#!/bin/sh\n[ "$*" = ' + shlex.quote(' '.join(args)) + ' ] || exit 1\nprintf \'%s\\n\' ' + shlex.quote(reply) + '\n')
    if name == 'hostname':
        tool.write_text(tool.read_text().replace('#!/bin/sh\n', '#!/bin/sh\n[ \"$#\" != 0 ] || { echo invented-host; exit 0; }\n'))
    tool.chmod(0o700)
    if name == 'taskq':
        tool.with_name(tool.name + '.pilotq-version.json').write_text(json.dumps({
            'version': '4.6.44', 'sha256': hashlib.sha256(tool.read_bytes()).hexdigest()}))
install = work / 'install'
install.mkdir()
config = install / 'config.json'
config.write_text(json.dumps({
    'bot_token': '123:invented', 'owner_id': 7,
    'codex': str(bin / 'codex'), 'gh': str(bin / 'gh'),
    'taskq': {'bin': str(bin / 'taskq'), 'root': str(work)},
    'voice': {'ffmpeg': str(bin / 'ffmpeg'), 'recognizer': str(bin / 'recognizer'), 'model': str(work)},
}))
config.chmod(0o600)
PYFIXTURE
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
