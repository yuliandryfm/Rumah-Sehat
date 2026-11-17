#!/usr/bin/env bash
# PreToolUse hook (Bash): mencegah Claude melewati aturan "hanya commit hijau".
#
# Diblokir:
# - --no-verify pada git commit / merge / cherry-pick / rebase / am
# - flag pendek -n pada git commit (setara --no-verify)
# - mengubah atau menimpa core.hooksPath
# - mengubah isi .githooks/ lewat shell
# - mengubah scripts/verify.sh lewat shell (pakai tool Edit, yang meminta persetujuan user)

INPUT="$(cat)"

if command -v python3 >/dev/null 2>&1; then
  CMD="$(printf '%s' "$INPUT" | python3 -c 'import json,sys
try:
    print(json.load(sys.stdin).get("tool_input", {}).get("command", ""))
except Exception:
    print("")')"
else
  CMD="$INPUT"
fi

[ -z "$CMD" ] && exit 0

block() {
  echo "DIBLOKIR oleh guard-git: $1" >&2
  echo "Aturan proyek: hanya kode hijau yang boleh di-commit, dan penjaganya tidak boleh dilewati atau diubah tanpa persetujuan user." >&2
  exit 2
}

if printf '%s' "$CMD" | grep -Eq 'hooksPath'; then
  block "mengubah atau menimpa core.hooksPath tidak diizinkan."
fi

if printf '%s' "$CMD" | grep -Eq 'git([[:space:]].*)?[[:space:]](commit|merge|cherry-pick|rebase|am)([[:space:]].*)?--no-verify'; then
  block "--no-verify tidak diizinkan."
fi

if printf '%s' "$CMD" | grep -Eq 'git([[:space:]].*)?[[:space:]]commit([[:space:]]+[^[:space:]]+)*[[:space:]]+-[a-zA-Z]*n[a-zA-Z]*([[:space:]]|$)'; then
  block "flag -n pada git commit (setara --no-verify) tidak diizinkan. Kalau -n muncul di dalam pesan commit, ubah kalimat pesannya."
fi

WRITE_OPS='(>|[[:space:]]rm[[:space:]]|[[:space:]]mv[[:space:]]|[[:space:]]cp[[:space:]]|sed[[:space:]]+-i|chmod|tee[[:space:]]|truncate|ln[[:space:]])'

if printf '%s' "$CMD" | grep -q '\.githooks' && printf ' %s' "$CMD" | grep -Eq "$WRITE_OPS"; then
  block "mengubah .githooks/ lewat shell tidak diizinkan."
fi

if printf '%s' "$CMD" | grep -q 'scripts/verify\.sh' && printf ' %s' "$CMD" | grep -Eq "$WRITE_OPS"; then
  block "mengubah scripts/verify.sh lewat shell tidak diizinkan. Gunakan tool Edit/Write, yang akan meminta persetujuan user."
fi

exit 0
