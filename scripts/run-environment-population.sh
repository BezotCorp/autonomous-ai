#!/usr/bin/env bash
# Coexistence of newborns, each with an isolated control.
#
#   scripts/run-environment-population.sh <data-dir> <tag> <pairs> <days> [first-seed]
#
# For each pair k, two individuals are born (consecutive seeds) and live
# together in <tag>-pair-k; an exact branch of each, taken at birth, lives
# alone in its own environment. All environments share seed, latency (60 s),
# perceptibility (1) and no external access. Nothing is imposed. Then one
# CSV line per individual (see `inspect --section summary --csv`), whose last
# columns give what it perceived, emitted and received.
set -euo pipefail

if [ "$#" -lt 4 ]; then
  sed -n '2,11p' "$0" >&2
  exit 1
fi

data=$1
tag=$2
pairs=$3
days=$4
seed=${5:-1}
seconds=$((days * 86400))
root=$(git rev-parse --show-toplevel)
cargo build --release --locked --manifest-path "$root/Cargo.toml" >&2
target=$(cargo metadata --format-version 1 --no-deps --manifest-path "$root/Cargo.toml" |
  python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
binary="$target/release/autonomous-ai"

ai() {
  "$binary" "$@" --data "$data"
}

names=()

for ((pair = 1; pair <= pairs; pair++)); do
  ai environment create "$tag-pair-$pair" --seed 1 --latency 60 >/dev/null

  for side in 1 2; do
    name="$tag-$seed"
    ai birth "$name" --seed "$seed" >/dev/null
    ai branch "$name" "$name-alone" >/dev/null
    ai environment create "$name-alone" --seed 1 --latency 60 >/dev/null
    ai environment admit "$name-alone" "$name-alone" >/dev/null
    ai environment admit "$tag-pair-$pair" "$name" >/dev/null
    names+=("$name" "$name-alone")
    seed=$((seed + 1))
  done

  ai environment live "$tag-pair-$pair" --deterministic --for "$seconds" >/dev/null
done

for name in "${names[@]}"; do
  case $name in *-alone) ai environment live "$name" --deterministic --for "$seconds" >/dev/null ;; esac
done

header_written=0

for name in "${names[@]}"; do
  if [ "$header_written" = 0 ]; then
    ai inspect "$name" --section summary --csv | sed -n '1p'
    header_written=1
  fi

  ai inspect "$name" --section summary --csv | sed -n '2p'
done

for ((pair = 1; pair <= pairs; pair++)); do
  ai environment inspect "$tag-pair-$pair"
done
