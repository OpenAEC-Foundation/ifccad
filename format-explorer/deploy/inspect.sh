#!/usr/bin/env bash
set -euo pipefail
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
ss -ltnp '( sport = :4183 )' || true
for config in /etc/nginx/sites-enabled/*ifccad* /etc/nginx/conf.d/*ifccad*; do
  [[ -f "$config" ]] || continue
  printf 'Proxy: %s\n' "$config"
  grep -E 'server_name|proxy_pass|root |client_max_body_size|limit_req|proxy_.*timeout' "$config" || true
done
