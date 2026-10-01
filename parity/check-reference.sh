#!/usr/bin/env bash
# Sourced by the corpus and sweep gates; no checkout mutations.
: "${FLIP_RS_REFERENCE:?set FLIP_RS_REFERENCE to an NVlabs/flip clone at b475eb4}"
export FLIP_RS_REFERENCE
flip_reference_revision="$(git --no-optional-locks -C "$FLIP_RS_REFERENCE" rev-parse HEAD)"
if [[ "$flip_reference_revision" != b475eb4bf394ab877c42166c9eb0a84a02cc5b14 ]]; then
  echo 'The parity oracle requires NVIDIA FLIP v1.7 at b475eb4.' >&2
  exit 1
fi
# --no-optional-locks prevents refreshing/writing the index during the read.
flip_reference_status="$(git --no-optional-locks -C "$FLIP_RS_REFERENCE" status --porcelain --untracked-files=all)"
if [[ -n "$flip_reference_status" ]]; then
  echo 'The parity oracle requires a clean reference checkout (including untracked files).' >&2
  exit 1
fi
