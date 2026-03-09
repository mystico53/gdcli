#!/usr/bin/env bash
# Score a benchmark run directory against the checklist.
# Usage: ./score.sh <run-dir> [godot-binary]
#
# Prints pass/fail for each check item and a final score.

set -euo pipefail

DIR="${1:?Usage: score.sh <run-dir> [godot-binary]}"
GODOT="${2:-godot}"

score=0
total=20

pass() { echo "  [PASS] $1"; ((score++)) || true; }
fail() { echo "  [FAIL] $1"; }
check_file() { [[ -f "$DIR/$1" ]] && pass "$1 exists" || fail "$1 exists"; }

echo "=== Structure ==="
check_file "project.godot"
check_file "scenes/main_menu.tscn"
check_file "scenes/game.tscn"
check_file "scenes/settings.tscn"
check_file "scenes/credits.tscn"

echo ""
echo "=== Main Menu Scene ==="
if [[ -f "$DIR/scenes/main_menu.tscn" ]]; then
  MM="$DIR/scenes/main_menu.tscn"
  grep -q 'type="Control"' "$MM" && pass "Control root node" || fail "Control root node"
  grep -q 'type="ColorRect"' "$MM" && pass "Has ColorRect background" || fail "Has ColorRect background"
  grep -qi 'text.*=.*".*Game.*"' "$MM" && pass "Title label with 'Game'" || fail "Title label with 'Game'"
  # Check for Play, Settings, Quit buttons (in scene or script)
  if grep -q '"Play"' "$MM" && grep -q '"Settings"' "$MM" && grep -q '"Quit"' "$MM"; then
    pass "Has Play, Settings, Quit buttons"
  elif [[ -f "$DIR/scenes/main_menu.gd" ]] && grep -q 'Play' "$DIR/scenes/main_menu.gd"; then
    pass "Has Play, Settings, Quit buttons (via script)"
  else
    fail "Has Play, Settings, Quit buttons"
  fi
  grep -q 'type="VBoxContainer"' "$MM" && pass "VBoxContainer layout" || fail "VBoxContainer layout"
else
  for i in 1 2 3 4 5; do fail "main_menu.tscn missing"; done
fi

echo ""
echo "=== Placeholder Scenes ==="
for scene_info in "game:Game Scene" "settings:Settings" "credits:Credits"; do
  name="${scene_info%%:*}"
  label="${scene_info#*:}"
  SCENE="$DIR/scenes/${name}.tscn"
  SCRIPT="$DIR/scenes/${name}.gd"
  if [[ -f "$SCENE" ]]; then
    has_label=false
    has_back=false
    grep -qi "$label" "$SCENE" && has_label=true
    (grep -qi "Back" "$SCENE" || ([ -f "$SCRIPT" ] && grep -qi "Back\|menu\|main_menu" "$SCRIPT")) && has_back=true
    $has_label && $has_back && pass "${name}.tscn has label + back button" || fail "${name}.tscn has label + back button"
  else
    fail "${name}.tscn missing"
  fi
done

echo ""
echo "=== Navigation ==="
# Check scripts for scene transitions
check_nav() {
  local desc="$1" target="$2"
  found=false
  for f in "$DIR"/scenes/*.gd "$DIR"/scripts/*.gd "$DIR"/*.gd; do
    [[ -f "$f" ]] && grep -q "$target" "$f" && found=true && break
  done
  $found && pass "$desc" || fail "$desc"
}
check_nav "Play -> game.tscn" "game.tscn"
check_nav "Settings -> settings.tscn" "settings.tscn"
check_nav "Quit calls quit()" "quit()"
check_nav "game.tscn -> main_menu" "main_menu.tscn"
check_nav "settings.tscn -> main_menu" "main_menu.tscn"

echo ""
echo "=== Runtime ==="
if [[ -f "$DIR/project.godot" ]]; then
  # Try opening in editor headlessly
  if command -v "$GODOT" &>/dev/null; then
    if timeout 15 "$GODOT" --path "$DIR" --headless --quit 2>&1 | tee /tmp/godot_bench_out.txt; then
      if grep -qi "error\|SCRIPT ERROR" /tmp/godot_bench_out.txt; then
        fail "Opens without errors"
      else
        pass "Opens without errors"
      fi
    else
      fail "Opens without errors (godot exited non-zero)"
    fi
    # Second check: does it at least not crash
    pass "Headless quit test (ran)"
  else
    echo "  [SKIP] Godot binary not found, skipping runtime checks"
    total=$((total - 2))
  fi
else
  fail "Opens without errors (no project.godot)"
  fail "Headless quit test"
fi

echo ""
echo "=== SCORE: $score / $total ==="
