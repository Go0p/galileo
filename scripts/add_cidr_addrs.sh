#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: add_cidr_addrs.sh <interface> <network_cidr>

Example:
  add_cidr_addrs.sh eth0 217.217.223.0/24

Removes any existing assignments from the /24 on the interface and re-adds every usable
host address with a /24 mask. Skips .1 by default to avoid clashing with upstream
gateway; override by running SKIP_GATEWAY_HOST=0.
EOF
}

if (( $# != 2 )); then
  usage
  exit 1
fi

iface=$1
cidr=$2
skip_gateway_host=${SKIP_GATEWAY_HOST:-1}

if ! ip link show "$iface" >/dev/null 2>&1; then
  echo "Interface '$iface' not found" >&2
  exit 1
fi

if [[ $cidr != */* ]]; then
  echo "CIDR must be of the form <ip>/<prefix>" >&2
  exit 1
fi

prefix_ip=${cidr%/*}
prefix_len=${cidr#*/}

if [[ $prefix_len != "24" ]]; then
  echo "Only /24 prefixes are supported" >&2
  exit 1
fi

IFS=. read -r o1 o2 o3 o4 <<<"$prefix_ip" || {
  echo "Invalid IPv4 address '$prefix_ip'" >&2
  exit 1
}

if [[ $o4 != "0" ]]; then
  echo "For a /24 network the host portion must be .0 (e.g. 217.217.223.0/24)" >&2
  exit 1
fi

base="${o1}.${o2}.${o3}"

echo "Clearing existing ${base}.0/24 assignments from ${iface}"
for host in $(seq 1 254); do
  addr="${base}.${host}"
  ip addr del "${addr}/32" dev "$iface" 2>/dev/null || true
  ip addr del "${addr}/24" dev "$iface" 2>/dev/null || true
done

for host in $(seq 1 254); do
  if (( skip_gateway_host == 1 && host == 1 )); then
    echo "Skipping potential gateway ${base}.${host}/24"
    continue
  fi

  addr="${base}.${host}"
  echo "Adding ${addr}/24 to ${iface}"
  ip addr add "${addr}/24" dev "$iface"
done
