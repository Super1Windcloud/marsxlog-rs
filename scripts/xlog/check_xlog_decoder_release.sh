#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Run release preflight checks for xlog-decoder.

Usage:
  scripts/xlog/check_xlog_decoder_release.sh [options]

Options:
  --out-dir <dir>           Artifact root (default: artifacts/release/<timestamp>-xlog-decoder)
  --allow-dirty             Allow cargo package/publish dry-run on a dirty worktree
  --skip-tests              Skip cargo test
  --skip-crates-io-check    Skip cargo publish --dry-run
  -h, --help                Show help
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

timestamp="$(date +"%Y%m%d-%H%M%S")"
out_dir="${repo_root}/artifacts/release/${timestamp}-xlog-decoder"
allow_dirty=0
skip_tests=0
skip_crates_io_check=0

while (($# > 0)); do
  case "$1" in
    --out-dir)
      out_dir="$2"
      shift 2
      ;;
    --allow-dirty)
      allow_dirty=1
      shift
      ;;
    --skip-tests)
      skip_tests=1
      shift
      ;;
    --skip-crates-io-check)
      skip_crates_io_check=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

mkdir -p "$out_dir/logs"
summary_file="$out_dir/summary.md"
status_file="$out_dir/status.txt"

pkg_name="${CLI_PKG_NAME:-xlog-decoder}"
package_list_cmd=(cargo package -p "$pkg_name" --locked --list)
publish_dry_run_cmd=(cargo publish --dry-run -p "$pkg_name" --locked)
if [[ "$allow_dirty" -eq 1 ]]; then
  package_list_cmd+=(--allow-dirty)
  publish_dry_run_cmd+=(--allow-dirty)
fi

failures=0
last_exit_code=0
failed_steps=()

run_step() {
  local name="$1"
  shift
  local logfile="$out_dir/logs/${name}.log"
  echo "[release-cli] RUN ${name}: $*" | tee -a "$summary_file"
  if "$@" >"$logfile" 2>&1; then
    echo "[release-cli] PASS ${name}" | tee -a "$summary_file"
    last_exit_code=0
  else
    last_exit_code=$?
    failures=$((failures + 1))
    failed_steps+=("${name}:${last_exit_code}")
    echo "[release-cli] FAIL ${name} (exit=${last_exit_code}, log=${logfile})" | tee -a "$summary_file"
  fi
}

: > "$summary_file"
echo "# $pkg_name Release Preflight" >> "$summary_file"
echo >> "$summary_file"
echo "- artifact_root: \`$out_dir\`" >> "$summary_file"
echo "- worktree: \`$(git -C "$repo_root" status --short | wc -l | tr -d ' ')\` changed paths" >> "$summary_file"
echo "- allow_dirty: \`$allow_dirty\`" >> "$summary_file"
echo >> "$summary_file"

run_step package_list "${package_list_cmd[@]}"
if [[ "$last_exit_code" -eq 0 ]]; then
  cp "$out_dir/logs/package_list.log" "$out_dir/package_list.txt"
fi

run_step check cargo check -p "$pkg_name" --locked
if [[ "$skip_tests" -eq 0 ]]; then
  run_step tests cargo test -p "$pkg_name" --locked
fi

if [[ "$skip_crates_io_check" -eq 1 ]]; then
  echo "[release-cli] SKIP publish_dry_run" | tee -a "$summary_file"
else
  run_step publish_dry_run "${publish_dry_run_cmd[@]}"
fi

echo >> "$summary_file"
if [[ "$failures" -eq 0 ]]; then
  echo "release preflight: success" | tee "$status_file" >> "$summary_file"
  exit 0
fi

echo "release preflight: failed (${failures})" | tee "$status_file" >> "$summary_file"
echo "- failed_steps: \`${failed_steps[*]}\`" >> "$summary_file"
exit 1
