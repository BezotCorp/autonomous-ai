#!/usr/bin/env bash
# Coexistence experiment: isolated against shared environments.
#
#   scripts/run-environment-experiment.sh <data-dir> <individual-a> <individual-b> <tag> <days> [--external]
#
# Exact branches (same identity, same history, same random state) of two
# existing individuals are made, never touching the originals:
#   <tag>-isolated-a, <tag>-isolated-b   each alone in its own environment
#   <tag>-shared-a,   <tag>-shared-b     together in one environment
# The three environments have the same seed, latency (60 s), perceptibility
# (1) and external access (none, unless --external). Nothing is imposed:
# no stimulus, no initial exchange. Each environment then lives <days>
# virtual days (deterministic mode). Reports cover the experiment period
# only; complete data stay available with `inspect` and `environment inspect`.
set -euo pipefail

if [ "$#" -lt 5 ]; then
  sed -n '2,15p' "$0" >&2
  exit 1
fi

data=$1
a=$2
b=$3
tag=$4
days=$5
external=()

if [ "${6:-}" = "--external" ]; then
  external=(--external)
fi

seconds=$((days * 86400))
root=$(git rev-parse --show-toplevel)
cargo build --release --locked --manifest-path "$root/Cargo.toml" >&2
target=$(cargo metadata --format-version 1 --no-deps --manifest-path "$root/Cargo.toml" |
  python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
binary="$target/release/autonomous-ai"

ai() {
  "$binary" "$@" --data "$data"
}

lived() {
  ai inspect "$1" --section summary --csv | sed -n '2p' | cut -d, -f3
}

for side in a b; do
  source_name=$a
  [ "$side" = b ] && source_name=$b
  ai branch "$source_name" "$tag-isolated-$side" >&2
  ai branch "$source_name" "$tag-shared-$side" >&2
  ai verify "$tag-isolated-$side" >&2
done

declare -A start
for name in "$tag-isolated-a" "$tag-isolated-b" "$tag-shared-a" "$tag-shared-b"; do
  start[$name]=$(lived "$name")
done

for environment in "$tag-isolated-a" "$tag-isolated-b" "$tag-shared"; do
  ai environment create "$environment" --seed 1 --latency 60 --perceptibility 1 "${external[@]}" >/dev/null
done

ai environment admit "$tag-isolated-a" "$tag-isolated-a"
ai environment admit "$tag-isolated-b" "$tag-isolated-b"
ai environment admit "$tag-shared" "$tag-shared-a"
ai environment admit "$tag-shared" "$tag-shared-b"

for environment in "$tag-isolated-a" "$tag-isolated-b" "$tag-shared"; do
  printf '\n=== living %s\n' "$environment"
  ( time ai environment live "$environment" --clock virtual --deterministic --for "$seconds" ) 2>&1
done

for name in "$tag-isolated-a" "$tag-shared-a" "$tag-isolated-b" "$tag-shared-b"; do
  printf '\n=== %s (from lived %s s)\n' "$name" "${start[$name]}"
  ai inspect "$name" --section report --from "${start[$name]}"
done
