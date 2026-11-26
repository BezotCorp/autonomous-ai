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
# A one-line summary per episode, then the statistics of each configuration,
# are printed at the end (complete reports stay available with `inspect`).
#
# With AUTONOMOUS_AI_STUDY=activities, the configurations compare instead
# the cognitive activities, all with knowledge and routing:
#   M: action only
#   U: action and the unconscious (idle --without-dreams)
#   R: action and dreams (idle --without-reflection)
#   F: the complete architecture
#   N: the complete architecture, the conscious ignoring the influences of
#      the unconscious (to measure what the influences themselves bring)
# After each round over the events, the intelligence may undertake up to
# AUTONOMOUS_AI_IDLE activities (default 10) on its own initiative.
# With AUTONOMOUS_AI_TEST_EVENTS=<dir>, events never seen during training
# are then submitted to a copy of each trained memory, to measure
# generalisation; their summaries are printed separately.
# Adding --without-cache to AUTONOMOUS_AI_OPTIONS disables the reuse of
# acquired data, to tell cognitive gains from technical ones. Statistics of
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

# Further decision parameters (e.g. "--cost-weight 0.05 --minimum-utility 0.2").
if [ -n "${AUTONOMOUS_AI_OPTIONS:-}" ]; then
  read -r -a options <<<"$AUTONOMOUS_AI_OPTIONS"
  extra+=("${options[@]}")
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

idle_options=()

if [ "${4:-}" = "--offline" ]; then
  idle_options+=(--offline)
fi

idle() {
  local memory=$1 seed=$2
  shift 2
  "$binary" idle --memory "$memory" --resources "$resources" --seed "$seed" \
    --max-activities "${AUTONOMOUS_AI_IDLE:-10}" "${idle_options[@]}" "$@"
}

if [ "${AUTONOMOUS_AI_STUDY:-}" = activities ]; then
  configurations=(M U R F N)
  declare -A flags=([M]="" [U]="" [R]="" [F]="" [N]="--without-influences")
  declare -A activities=(
    [M]=""
    [U]="--without-dreams"
    [R]="--without-reflection"
    [F]="--"
    [N]="--"
  )
else
  configurations=(A B C D)
  declare -A flags=(
    [A]="--without-knowledge --without-routing"
    [B]="--without-routing"
    [C]="--without-knowledge"
    [D]=""
  )
  declare -A activities=([A]="" [B]="" [C]="" [D]="")
fi

for configuration in "${configurations[@]}"; do
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

    if [ -n "${activities[$configuration]}" ]; then
      printf '\n--- %s: activities after round %d\n' "$configuration" "$((iteration + 1))"
      # shellcheck disable=SC2086
      idle "$memory" "$seed" $(printf '%s' "${activities[$configuration]}" | sed 's/^--$//')
      seed=$((seed + 1))
    fi
  done

  printf '\n=== Configuration %s (%s)\n' "$configuration" "${flags[$configuration]:-${activities[$configuration]:-default}}"
  "$binary" inspect --memory "$memory" --section summary
  "$binary" inspect --memory "$memory" --section statistics

  if [ -n "${AUTONOMOUS_AI_TEST_EVENTS:-}" ]; then
    test_memory="$workspace/$configuration.test.bin"
    cp "$memory" "$test_memory"
    before=$("$binary" inspect --memory "$test_memory" --section summary | wc -l)

    for event in $(find "$AUTONOMOUS_AI_TEST_EVENTS" -maxdepth 1 -name '*.ron' | sort); do
      # shellcheck disable=SC2086
      run "$test_memory" "$seed" "$event" ${flags[$configuration]}
      seed=$((seed + 1))
    done

    printf '\n=== Configuration %s on unseen events\n' "$configuration"
    "$binary" inspect --memory "$test_memory" --section summary | sed -n "1p;$((before + 1)),\$p"
  fi
done
