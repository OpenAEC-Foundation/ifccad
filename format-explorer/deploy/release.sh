#!/usr/bin/env bash
set -Eeuo pipefail

# Only this application is managed; Compose, nginx, TLS and isolation stay intact.
root=${IFCCAD_DEPLOY_ROOT:-/opt/ifccad-explorer}
id=${1:?release ID required}
checksum=${2:?archive checksum required}
[[ "$id" =~ ^[a-f0-9]{40}-[0-9]+-[0-9]+$ && "$checksum" =~ ^[a-f0-9]{64}$ ]]
[[ "$root" == /* && "$root" != / && -f "$root/compose.yml" ]]
root=$(realpath "$root")
exec 9>"$root/.deployment.lock"
flock -w 180 9
archive="$root/incoming/$id.tar.gz"
printf '%s  %s\n' "$checksum" "$archive" | sha256sum --check --status
revision=${id:0:40}
release="$root/releases/$id"
[[ ! -e "$release" ]]
mkdir -p "$root/releases"
mkdir "$release"
tar --extract --gzip --file "$archive" --directory "$release" --no-same-owner --no-same-permissions
python3 -c 'import json,sys; assert json.load(open(sys.argv[1]))["revision"] == sys.argv[2]' "$release/dist/version.json" "$revision"
[[ -f "$release/container.mjs" && -f "$release/ifccad-viewer" ]]
chmod -R a+rX "$release"
chmod 755 "$release/ifccad-viewer"

previous=''
managed=false
if [[ -e "$root/current" || -L "$root/current" ]]; then
  [[ -L "$root/current" ]]
  previous=$(readlink -e "$root/current")
  [[ "$previous" == "$root/releases/"* ]]
fi
if [[ -f "$root/deployment.yml" ]]; then
  grep -qx '# Managed by IFCCAD deployment' "$root/deployment.yml"
  [[ -n "$previous" ]]
  managed=true
  cp "$root/deployment.yml" "$release/previous-deployment.yml"
elif [[ -n "$previous" ]]; then
  echo 'Existing current link has no managed configuration' >&2
  exit 1
fi

compose=(docker compose --project-name ifccad-explorer --project-directory "$root" -f "$root/compose.yml")
rollback() {
  trap - ERR INT TERM
  echo 'Deployment failed; restoring previous explorer release' >&2
  if [[ -n "$previous" ]]; then
    ln -s "$previous" "$root/current.restore"
    mv -Tf "$root/current.restore" "$root/current"
  else
    rm -f "$root/current"
  fi
  if $managed; then
    cp "$release/previous-deployment.yml" "$root/deployment.yml"
    "${compose[@]}" -f "$root/deployment.yml" up -d --no-deps --force-recreate explorer || true
  else
    rm -f "$root/deployment.yml"
    "${compose[@]}" up -d --no-deps --force-recreate explorer || true
  fi
  echo 'Verify the restored service if rollback reported an error.' >&2
  exit 1
}

cat > "$root/deployment.yml.next" <<EOF
# Managed by IFCCAD deployment
services:
  explorer:
    working_dir: /app
    command: ["node", "container.mjs"]
    volumes:
      - "$root/current:/app:ro"
EOF
ln -s "$release" "$root/current.next"
trap rollback ERR INT TERM
mv -Tf "$root/deployment.yml.next" "$root/deployment.yml"
mv -Tf "$root/current.next" "$root/current"
"${compose[@]}" -f "$root/deployment.yml" up -d --no-deps --force-recreate explorer

ready=false
for ((attempt=0; attempt<${IFCCAD_HEALTH_ATTEMPTS:-15}; attempt++)); do
  if docker exec ifccad-explorer-explorer-1 node /app/deploy/smoke.mjs http://127.0.0.1:4183 "$revision" basic; then
    ready=true
    break
  fi
  sleep 2
done
$ready
docker exec ifccad-explorer-explorer-1 node /app/deploy/smoke.mjs http://127.0.0.1:4183 "$revision" full
curl --fail --silent --show-error --max-time 20 https://ifccad-explorer.open-aec.com/version.json |
  python3 -c 'import json,sys; assert json.load(sys.stdin)["revision"] == sys.argv[1]' "$revision"
trap - ERR INT TERM
rm -f "$archive"
printf 'Deployed %s; previous release: %s\n' "$id" "${previous:-original Compose mount}"
