set -euo pipefail

MAX_CONCURRENT="${MAX_CONCURRENT:-8}"
MODPACKS_DIR="${MODPACKS_DIR:-modpacks}"

if ! command -v packwiz >/dev/null 2>&1; then
  echo "::error::packwiz not found in PATH" >&2
  exit 1
fi

if [ ! -d "$MODPACKS_DIR" ]; then
  echo "::error::modpacks directory not found: $MODPACKS_DIR" >&2
  exit 1
fi

TARGETS=()
SKIPPED=()

for pack in "$MODPACKS_DIR"/*/; do
  [ -d "$pack" ] || continue
  pack="${pack%/}"

  if [ -f "$pack/auto-update-ignore.json" ]; then
    SKIPPED+=("$pack")
    continue
  fi

  for sub in "$pack"/*-mr "$pack"/*-cf; do
    [ -d "$sub" ] || continue
    TARGETS+=("$sub")
  done
done

if [ "${#SKIPPED[@]}" -gt 0 ]; then
  echo "skipping ${#SKIPPED[@]} pack(s) with auto-update-ignore.json:"
  for s in "${SKIPPED[@]}"; do
    echo "  - $s"
  done
fi

if [ "${#TARGETS[@]}" -eq 0 ]; then
  echo "no pack subdirs to update."
  exit 0
fi

echo "queued ${#TARGETS[@]} subdir(s), running up to $MAX_CONCURRENT in parallel"

FAILURES=0

printf '%s\n' "${TARGETS[@]}" | MODPACKS_DIR="$MODPACKS_DIR" xargs -P "$MAX_CONCURRENT" -I{} bash -c '
  dir="$1"
  label="${dir#"$MODPACKS_DIR"/}"
  echo "updating $label"
  if (cd "$dir" && packwiz update --all -y) >"/tmp/upd-$$.log" 2>&1; then
    echo "ok: $label"
    rm -f "/tmp/upd-$$.log"
  else
    echo "::error::FAIL $label" >&2
    sed "s/^/  /" "/tmp/upd-$$.log" >&2
    rm -f "/tmp/upd-$$.log"
    exit 1
  fi
' _ {} || FAILURES=$?

if [ "$FAILURES" -ne 0 ]; then
  echo "::error::one or more updates failed" >&2
  exit 1
fi

echo "all updates finished successfully."