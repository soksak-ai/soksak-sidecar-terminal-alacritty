#!/bin/sh
set -eu
[ "$#" -eq 2 ] && [ -n "$1" ] && [ -n "$2" ] || { echo 'usage: stage-built.sh <out> <target>' >&2; exit 2; }
out=$1
target=$2
repository=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
# An absolute candidate output is allowed only outside the source repository.
case "$out" in ''|/|.|*..*|"$repository"|"$repository"/*) echo 'stage output is unsafe or inside the source repository' >&2; exit 2 ;; esac
name=soksak-sidecar-terminal-alacritty
case "$target" in *windows*) ext=.exe ;; *) ext= ;; esac
source=target/$target/release/$name$ext
[ -f "$source" ] || { echo "release binary is missing: $source" >&2; exit 1; }
staged_dir=$out/dist
mkdir -p "$staged_dir"
[ ! -L "$out" ] || { echo 'stage output must not be a symbolic link' >&2; exit 2; }
staged=$staged_dir/$name$ext
if [ -e "$staged" ]; then
  cmp -s "$source" "$staged" || { echo "staged binary conflicts with current build" >&2; exit 1; }
else
  next=$staged_dir/.$name$ext.next.$$
  cp "$source" "$next"
  chmod +x "$next"
  mv "$next" "$staged"
fi
generated=$out/.sidecar.json.next.$$
sed "s#\"process\": \"dist/$name\"#\"process\": \"dist/$name$ext\"#" sidecar.json > "$generated"
if [ -e "$out/sidecar.json" ]; then
  cmp -s "$generated" "$out/sidecar.json" || { echo "staged manifest conflicts with source" >&2; exit 1; }
  find "$generated" -delete
else
  mv "$generated" "$out/sidecar.json"
fi
staged_manifest=$staged_dir/sidecar.json
if [ -e "$staged_manifest" ]; then
  cmp -s "$out/sidecar.json" "$staged_manifest" || { echo "process manifest conflicts with staged manifest" >&2; exit 1; }
else
  cp "$out/sidecar.json" "$staged_dir/.sidecar.json.next.$$"
  mv "$staged_dir/.sidecar.json.next.$$" "$staged_manifest"
fi
echo "SIDECAR_STAGED target=$target output=$staged"
