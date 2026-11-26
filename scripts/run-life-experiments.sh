#!/usr/bin/env bash
# Experiments on living individuals (virtual clock, offline, reproducible).
#
#   scripts/run-life-experiments.sh <resources-dir> <schedule.ron> [individuals] [hours]
#
# In a temporary data directory:
#   population  individuals born from seeds 1..N live <hours> virtual hours
#               under the same scheduled stimuli;
#   decoupled   the same births with a sensitivity scale of zero: the latent
#               state then acts on no mechanism (channels keep their biases),
#               which isolates what the coupling itself changes;
#   inert       the same births with an impulse scale of zero: nothing that
#               happens moves the latent state (it only drifts);
#   twin        seed 1 born again and living the same: an identical
#               existence is expected (reproducibility);
#   secluded    a duplicate of newborn 1 living without any stimulus;
#   unswayed    seed 1 living with the conscious ignoring the influences;
#   dormant     a duplicate of the living seed 1 living another period
#               after a 30-day stop, against
#   continued   another duplicate living the same period without the stop.
# One CSV line per individual is written to <workspace>/results.csv and
# printed; complete summaries stay available with `inspect`.
set -euo pipefail

if [ "$#" -lt 2 ]; then
  sed -n '2,26p' "$0" >&2
  exit 1
fi

resources=$1
schedule=$2
count=${3:-6}
hours=${4:-24}
seconds=$((hours * 3600))

root=$(git rev-parse --show-toplevel)
cargo build --release --locked --manifest-path "$root/Cargo.toml" >&2
target=$(cargo metadata --format-version 1 --no-deps --manifest-path "$root/Cargo.toml" |
  python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
binary="$target/release/autonomous-ai"

workspace=$(mktemp -d /tmp/autonomous-ai-life.XXXXXX)
printf 'Workspace: %s\n' "$workspace" >&2

ai() {
  "$binary" "$@" --data "$workspace"
}

live() {
  local individual=$1
  shift
  ai live "$individual" --resources "$resources" --clock virtual --offline \
    --deterministic --for "$seconds" "$@" >/dev/null
}

record() {
  ai inspect "$1" --section summary --csv | sed -n '2p' >>"$workspace/results.csv"
}

header_written=0

for ((seed = 1; seed <= count; seed++)); do
  for variant in population decoupled inert; do
    name="$variant-$seed"
    ai birth "$name" --seed "$seed" >/dev/null

    case $variant in
      decoupled) ai intervene "$name" --scale sensitivity_scale=0 --reason "decoupled control" >/dev/null ;;
      inert) ai intervene "$name" --scale impulse_scale=0 --reason "inert control" >/dev/null ;;
    esac

    if [ "$seed" = 1 ] && [ "$variant" = population ]; then
      ai duplicate "$name" secluded-1 >/dev/null
    fi

    live "$name" --schedule "$schedule"

    if [ "$header_written" = 0 ]; then
      ai inspect "$name" --section summary --csv | sed -n '1p' >"$workspace/results.csv"
      header_written=1
    fi

    record "$name"
  done
done

ai birth twin-1 --seed 1 >/dev/null
live twin-1 --schedule "$schedule"
record twin-1

live secluded-1
record secluded-1

ai birth unswayed-1 --seed 1 >/dev/null
live unswayed-1 --schedule "$schedule" --without-influences
record unswayed-1

ai duplicate population-1 dormant-1 >/dev/null
ai duplicate population-1 continued-1 >/dev/null
live dormant-1 --schedule "$schedule" --dormancy 2592000
record dormant-1
live continued-1 --schedule "$schedule"
record continued-1

column -s, -t <"$workspace/results.csv"
