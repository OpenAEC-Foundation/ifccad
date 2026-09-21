#!/usr/bin/env bash
set -euo pipefail
docker exec ifccad-explorer-explorer-1 node --version
docker exec ifccad-explorer-explorer-1 ldd --version | head -1
docker inspect --format '{{.Config.WorkingDir}}' ifccad-explorer-explorer-1
docker inspect --format '{{json .Config.Env}}' ifccad-explorer-explorer-1 | python3 -c 'import json,sys; print("\n".join(v for v in json.load(sys.stdin) if v.split("=",1)[0] in ("PORT", "PUBLIC_ORIGIN", "IFCCAD_VIEWER_BIN", "NODE_ENV")))'
uname -m
. /etc/os-release
printf 'OS: %s %s\n' "$ID" "$VERSION_ID"
command -v node || true
node --version || true
systemctl list-unit-files --no-pager --no-legend | awk '$1 ~ /ifccad|format-explorer/ {print $1}' | while read -r unit; do
  systemctl show "$unit" --property=Id,FragmentPath,WorkingDirectory,User,Group,ActiveState,SubState,MainPID
  systemctl cat "$unit" | grep -E '^(WorkingDirectory|ExecStart|User|Group|ProtectSystem|ProtectHome|PrivateTmp|MemoryMax|CPUQuota|ReadWritePaths|NoNewPrivileges|Environment=(PORT|PUBLIC_ORIGIN|IFCCAD_VIEWER_BIN|NODE_ENV))=' || true
  pid=$(systemctl show "$unit" --property=MainPID --value)
  if [[ "$pid" =~ ^[1-9][0-9]*$ ]]; then
    readlink "/proc/$pid/exe" || true
    readlink "/proc/$pid/cwd" || true
  fi
done
docker ps -a --filter name=ifccad --format '{{.ID}} {{.Names}} {{.Image}} {{.Status}}'
for container in $(docker ps -aq --filter name=ifccad); do
  docker inspect --format '{{json .Config.Labels}} {{json .Mounts}} {{json .NetworkSettings.Networks}}' "$container"
  docker inspect --format 'User={{.Config.User}} Image={{.Config.Image}} Entrypoint={{json .Config.Entrypoint}} Cmd={{json .Config.Cmd}} Readonly={{.HostConfig.ReadonlyRootfs}} Memory={{.HostConfig.Memory}} NanoCpus={{.HostConfig.NanoCpus}} Security={{json .HostConfig.SecurityOpt}} Caps={{json .HostConfig.CapDrop}} Tmpfs={{json .HostConfig.Tmpfs}} Restart={{json .HostConfig.RestartPolicy}} Pids={{.HostConfig.PidsLimit}}' "$container"
done
ss -ltnp '( sport = :4183 )' || true
for config in /etc/nginx/sites-enabled/*ifccad* /etc/nginx/conf.d/*ifccad*; do
  [[ -f "$config" ]] || continue
  printf 'Proxy: %s\n' "$config"
  grep -E 'server_name|proxy_pass|root |client_max_body_size|limit_req|proxy_.*timeout' "$config" || true
done
