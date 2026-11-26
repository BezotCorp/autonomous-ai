#!/usr/bin/env bash
# Comparative experiments (TODO section 19).
#
#   scripts/run-experiments.sh <resources-dir> <events-dir> [runs] [--offline]
#
# A training memory is first built by reasoning once over every event.
# Then, in isolated temporary copies under /tmp, every event is replayed
# `runs` times under four configurations:
#   A: no acquired knowledge, no learned routing (empty memory)
#   B: acquired knowledge, no learned routing
#   C: learned routing, no acquired knowledge
#   D: acquired knowledge and learned routing
# The statistics of each configuration are printed at the end. Statistics of
# B, C and D also include the training episodes, which ran under the full
# configuration (label "knowledge=true routing=true").
set -euo pipefail

if [ "$#" -lt 2 ]; then
  sed -n '2,15p' "$0" >&2
  exit 1
fi

resources=$1
events=$2
runs=${3:-5}
extra=()

if [ "${4:-}" = "--offline" ]; then
  extra+=(--offline)
fi

root=$(git rev-parse --show-toplevel)
cargo build --release --locked --manifest-path "$root/Cargo.toml" >&2
target=$(cargo metadata --format-version 1 --no-deps --manifest-path "$root/Cargo.toml" |
  python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
binary="$target/release/autonomous-ai"

workspace=$(mktemp -d /tmp/autonomous-ai-experiments.XXXXXX)
printf 'Workspace: %s\n' "$workspace" >&2

mapfile -t files < <(find "$events" -maxdepth 1 -name '*.ron' | sort)

if [ "${#files[@]}" -eq 0 ]; then
  printf 'No event (*.ron) in %s\n' "$events" >&2
  exit 1
fi

run() {
  local memory=$1 seed=$2 event=$3
  shift 3
  "$binary" run --memory "$memory" --resources "$resources" --seed "$seed" \
    "${extra[@]}" "$@" <"$event" >/dev/null
}

seed=1

for event in "${files[@]}"; do
  run "$workspace/training.bin" "$seed" "$event"
  seed=$((seed + 1))
done

declare -A flags=(
  [A]="--without-knowledge --without-routing"
  [B]="--without-routing"
  [C]="--without-knowledge"
  [D]=""
)

for configuration in A B C D; do
  memory="$workspace/$configuration.bin"

  if [ "$configuration" != A ]; then
    cp "$workspace/training.bin" "$memory"
  fi

  for ((iteration = 0; iteration < runs; iteration++)); do
    for event in "${files[@]}"; do
      # shellcheck disable=SC2086
      run "$memory" "$seed" "$event" ${flags[$configuration]}
      seed=$((seed + 1))
    done
  done

  printf '\n=== Configuration %s (%s)\n' "$configuration" "${flags[$configuration]:-full}"
  "$binary" inspect --memory "$memory" --section statistics
done
