#!/usr/bin/env bash
# Comparative experiments in the laboratory (TODO section 19).
#
#   scripts/run-experiments.sh <resources-dir> <events-dir> [runs] [--offline]
#
# Every configuration is an individual born from the same seed
# (AUTONOMOUS_AI_BIRTH_SEED, default 1) in a temporary data directory, so
# that they share their constitution. A training individual first reasons
# once over every event. Then every event is submitted `runs` times, as
# laboratory episodes, to copies of it under four configurations:
#   A: no acquired knowledge, no learned routing (a newborn)
#   B: acquired knowledge, no learned routing
#   C: learned routing, no acquired knowledge
#   D: acquired knowledge and learned routing
# Laboratory episodes are neutral (without latent modulation), which keeps
# them comparable with earlier generations; AUTONOMOUS_AI_LATENT=1 lets the
# latent state of each individual modulate them instead.
#
# With AUTONOMOUS_AI_STUDY=activities, the configurations compare instead
# the cognitive activities, all with knowledge and routing:
#   M: action only
#   U: action and the unconscious (living without dream, reverie, simulation)
#   R: action and imagination (living without reflection, introspection)
#   F: the complete architecture
#   N: the complete architecture, the conscious ignoring the influences of
#      the unconscious (to measure what the influences themselves bring)
# After each round over the events, the individual lives
# AUTONOMOUS_AI_LIVE virtual seconds (default 3600) on its own.
# With AUTONOMOUS_AI_TEST_EVENTS=<dir>, events never seen during training
# are then submitted to a copy of each trained individual, to measure
# generalisation; their summaries are printed separately.
# Adding --without-cache to AUTONOMOUS_AI_OPTIONS disables the reuse of
# acquired data, to tell cognitive gains from technical ones.
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
birth_seed=${AUTONOMOUS_AI_BIRTH_SEED:-1}
neutral=(--neutral)

if [ "${AUTONOMOUS_AI_LATENT:-}" = 1 ]; then
  neutral=()
fi

birth() {
  "$binary" birth "$1" --seed "$birth_seed" --data "$workspace" >/dev/null
}

duplicate() {
  "$binary" duplicate "$1" "$2" --data "$workspace" >/dev/null
}

inspect() {
  "$binary" inspect "$1" --data "$workspace" --resources "$resources" --section "$2"
}

mapfile -t files < <(find "$events" -maxdepth 1 -name '*.ron' | sort)

if [ "${#files[@]}" -eq 0 ]; then
  printf 'No event (*.ron) in %s\n' "$events" >&2
  exit 1
fi

run() {
  local individual=$1 seed=$2 event=$3
  shift 3
  "$binary" episode "$individual" --data "$workspace" --resources "$resources" \
    --seed "$seed" "${neutral[@]}" "${extra[@]}" "$@" <"$event" >/dev/null
}

seed=1
birth training

for event in "${files[@]}"; do
  run training "$seed" "$event"
  seed=$((seed + 1))
done

live_options=(--clock virtual --deterministic --for "${AUTONOMOUS_AI_LIVE:-3600}")

if [ "${4:-}" = "--offline" ]; then
  live_options+=(--offline)
fi

live() {
  local individual=$1
  shift
  "$binary" live "$individual" --data "$workspace" --resources "$resources" \
    "${live_options[@]}" "$@" | sed -n '1,12p'
}

if [ "${AUTONOMOUS_AI_STUDY:-}" = activities ]; then
  configurations=(M U R F N)
  declare -A flags=([M]="" [U]="" [R]="" [F]="" [N]="--without-influences")
  declare -A activities=(
    [M]=""
    [U]="--without dream --without reverie --without simulation"
    [R]="--without reflection --without introspection"
    [F]="--"
    [N]="--without-influences"
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
  individual=$configuration

  if [ "$configuration" = A ]; then
    birth "$individual"
  else
    duplicate training "$individual"
  fi

  for ((iteration = 0; iteration < runs; iteration++)); do
    for event in "${files[@]}"; do
      # shellcheck disable=SC2086
      run "$individual" "$seed" "$event" ${flags[$configuration]}
      seed=$((seed + 1))
    done

    if [ -n "${activities[$configuration]}" ]; then
      printf '\n--- %s: activities after round %d\n' "$configuration" "$((iteration + 1))"
      # shellcheck disable=SC2086
      live "$individual" $(printf '%s' "${activities[$configuration]}" | sed 's/^--$//')
      seed=$((seed + 1))
    fi
  done

  printf '\n=== Configuration %s (%s)\n' "$configuration" "${flags[$configuration]:-${activities[$configuration]:-default}}"
  inspect "$individual" episodes
  inspect "$individual" statistics

  if [ -n "${AUTONOMOUS_AI_TEST_EVENTS:-}" ]; then
    duplicate "$individual" "$individual-test"
    before=$(inspect "$individual-test" episodes | wc -l)

    for event in $(find "$AUTONOMOUS_AI_TEST_EVENTS" -maxdepth 1 -name '*.ron' | sort); do
      # shellcheck disable=SC2086
      run "$individual-test" "$seed" "$event" ${flags[$configuration]}
      seed=$((seed + 1))
    done

    printf '\n=== Configuration %s on unseen events\n' "$configuration"
    inspect "$individual-test" episodes | sed -n "1p;$((before + 1)),\$p"
  fi
done
