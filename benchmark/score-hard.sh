#!/usr/bin/env bash
# Score the hard benchmark run.
# Usage: ./score-hard.sh <run-dir> [phase] [godot-binary]
# phase: 1 (default), 2 (includes phase 2 checks)

set -uo pipefail

DIR="${1:?Usage: score-hard.sh <run-dir> [phase] [godot-binary]}"
PHASE="${2:-1}"
GODOT="${3:-godot}"

score=0
total=0

pass() { echo "  [PASS] $1"; ((score++)) || true; ((total++)) || true; }
fail() { echo "  [FAIL] $1"; ((total++)) || true; }
check_file() { [[ -f "$DIR/$1" ]] && pass "$1 exists" || fail "$1 exists"; }

# Search all .gd files
has_in_scripts() {
  local pattern="$1"
  for f in "$DIR"/scenes/*.gd "$DIR"/scripts/*.gd "$DIR"/*.gd; do
    [[ -f "$f" ]] && grep -q "$pattern" "$f" && return 0
  done
  return 1
}

echo "============================="
echo "  PHASE 1: Build from scratch"
echo "============================="
echo ""

echo "=== Structure (5+1 pts) ==="
check_file "project.godot"
if [[ -f "$DIR/project.godot" ]]; then
  grep -q "main_scene" "$DIR/project.godot" && pass "main_scene is set" || fail "main_scene is set"
fi
check_file "scenes/player.tscn"
check_file "scenes/enemy.tscn"
check_file "scenes/bullet.tscn"
check_file "scenes/hud.tscn"
check_file "scenes/main.tscn"

echo ""
echo "=== Sub-resources (6 pts) ==="
# player: RectangleShape2D
if [[ -f "$DIR/scenes/player.tscn" ]]; then
  grep -q "RectangleShape2D" "$DIR/scenes/player.tscn" && pass "player has RectangleShape2D sub_resource" || fail "player has RectangleShape2D sub_resource"
  grep -q 'SubResource(' "$DIR/scenes/player.tscn" && pass "player CollisionShape2D references sub_resource" || fail "player CollisionShape2D references sub_resource"
else
  fail "player has RectangleShape2D sub_resource"
  fail "player CollisionShape2D references sub_resource"
fi

# enemy: CircleShape2D
if [[ -f "$DIR/scenes/enemy.tscn" ]]; then
  grep -q "CircleShape2D" "$DIR/scenes/enemy.tscn" && pass "enemy has CircleShape2D sub_resource" || fail "enemy has CircleShape2D sub_resource"
else
  fail "enemy has CircleShape2D sub_resource"
fi

# bullet: CircleShape2D
if [[ -f "$DIR/scenes/bullet.tscn" ]]; then
  grep -q "CircleShape2D" "$DIR/scenes/bullet.tscn" && pass "bullet has CircleShape2D sub_resource" || fail "bullet has CircleShape2D sub_resource"
else
  fail "bullet has CircleShape2D sub_resource"
fi

# hud: StyleBoxFlat
if [[ -f "$DIR/scenes/hud.tscn" ]]; then
  grep -q "StyleBoxFlat" "$DIR/scenes/hud.tscn" && pass "hud has StyleBoxFlat sub_resource" || fail "hud has StyleBoxFlat sub_resource"
else
  fail "hud has StyleBoxFlat sub_resource"
fi

# All sub_resource IDs unique within their namespace per file
dup_count=0
for f in "$DIR"/scenes/*.tscn; do
  [[ -f "$f" ]] || continue
  # Check sub_resource IDs separately from ext_resource IDs
  sub_ids=$(grep '^\[sub_resource' "$f" 2>/dev/null | grep -oE 'id="[^"]+"' | sort)
  sub_dups=$(echo "$sub_ids" | uniq -d | grep -c . 2>/dev/null || true)
  ext_ids=$(grep '^\[ext_resource' "$f" 2>/dev/null | grep -oE 'id="[^"]+"' | sort)
  ext_dups=$(echo "$ext_ids" | uniq -d | grep -c . 2>/dev/null || true)
  dup_count=$((dup_count + sub_dups + ext_dups))
done
[[ $dup_count -eq 0 ]] && pass "All resource IDs unique in their namespace" || fail "All resource IDs unique (found $dup_count dups)"

echo ""
echo "=== Scene Instancing (3 pts) ==="
if [[ -f "$DIR/scenes/main.tscn" ]]; then
  grep -q "player.tscn" "$DIR/scenes/main.tscn" && pass "main has ext_resource for player.tscn" || fail "main has ext_resource for player.tscn"
  grep -q 'instance=' "$DIR/scenes/main.tscn" && pass "main instances scenes" || fail "main instances scenes"
  grep -q "hud.tscn" "$DIR/scenes/main.tscn" && pass "main instances HUD" || fail "main instances HUD"
else
  fail "main has ext_resource for player.tscn"
  fail "main instances scenes"
  fail "main instances HUD"
fi

echo ""
echo "=== Node Hierarchies (5 pts) ==="
if [[ -f "$DIR/scenes/player.tscn" ]]; then
  (grep -q 'type="CollisionShape2D"' "$DIR/scenes/player.tscn" && grep -q 'type="Sprite2D"' "$DIR/scenes/player.tscn") \
    && pass "player: CollisionShape2D + Sprite2D children" || fail "player: CollisionShape2D + Sprite2D children"
else
  fail "player: CollisionShape2D + Sprite2D children"
fi

if [[ -f "$DIR/scenes/enemy.tscn" ]]; then
  (grep -q 'type="CollisionShape2D"' "$DIR/scenes/enemy.tscn" && grep -q 'type="Sprite2D"' "$DIR/scenes/enemy.tscn") \
    && pass "enemy: CollisionShape2D + Sprite2D children" || fail "enemy: CollisionShape2D + Sprite2D children"
else
  fail "enemy: CollisionShape2D + Sprite2D children"
fi

if [[ -f "$DIR/scenes/hud.tscn" ]]; then
  (grep -q 'type="MarginContainer"' "$DIR/scenes/hud.tscn" && grep -q 'type="VBoxContainer"' "$DIR/scenes/hud.tscn" \
    && grep -q 'ScoreLabel' "$DIR/scenes/hud.tscn" && grep -q 'HealthBar\|ProgressBar' "$DIR/scenes/hud.tscn") \
    && pass "hud: MarginContainer > VBoxContainer > ScoreLabel + HealthBar" || fail "hud: MarginContainer > VBoxContainer > ScoreLabel + HealthBar"
else
  fail "hud: MarginContainer > VBoxContainer > ScoreLabel + HealthBar"
fi

if [[ -f "$DIR/scenes/main.tscn" ]]; then
  (grep -q 'name="Enemies"' "$DIR/scenes/main.tscn" && grep -q 'name="Bullets"' "$DIR/scenes/main.tscn" \
    && grep -q 'SpawnTimer\|Timer' "$DIR/scenes/main.tscn") \
    && pass "main: Enemies + Bullets containers + SpawnTimer" || fail "main: Enemies + Bullets containers + SpawnTimer"
else
  fail "main: Enemies + Bullets containers + SpawnTimer"
fi

# Check parent paths are present (at least some nodes have parent= attributes)
parent_count=0
for f in "$DIR"/scenes/*.tscn; do
  [[ -f "$f" ]] || continue
  c=$(grep -c 'parent=' "$f" 2>/dev/null || true)
  parent_count=$((parent_count + c))
done
[[ $parent_count -ge 10 ]] && pass "Correct parent paths in .tscn files ($parent_count found)" || fail "Correct parent paths in .tscn files (only $parent_count found, expected 10+)"

echo ""
echo "=== Signal Connections (3 pts) ==="
if [[ -f "$DIR/scenes/bullet.tscn" ]]; then
  grep -q 'body_entered' "$DIR/scenes/bullet.tscn" && pass "bullet: body_entered signal" || fail "bullet: body_entered signal"
else
  fail "bullet: body_entered signal"
fi

if [[ -f "$DIR/scenes/main.tscn" ]]; then
  grep -q 'timeout' "$DIR/scenes/main.tscn" && pass "main: SpawnTimer timeout signal" || fail "main: SpawnTimer timeout signal"
else
  fail "main: SpawnTimer timeout signal"
fi

# Check connection syntax is valid (has signal=, from=, to=, method=)
conn_valid=true
for f in "$DIR"/scenes/*.tscn; do
  [[ -f "$f" ]] || continue
  while IFS= read -r line; do
    if ! echo "$line" | grep -q 'signal=.*from=.*to=.*method='; then
      conn_valid=false
      break 2
    fi
  done < <(grep '^\[connection' "$f" 2>/dev/null || true)
done
$conn_valid && pass "All connections have valid syntax" || fail "All connections have valid syntax"

echo ""
echo "=== Scripts (5 pts) ==="
has_in_scripts "move_and_slide" && pass "player.gd has move_and_slide" || fail "player.gd has move_and_slide"
has_in_scripts "direction_to" && pass "enemy.gd chases target" || fail "enemy.gd chases target"
has_in_scripts "queue_free" && pass "bullet.gd handles body_entered" || fail "bullet.gd handles body_entered"
has_in_scripts "update_score\|update_health" && pass "hud.gd has update methods" || fail "hud.gd has update methods"
has_in_scripts "instantiate\|_on_spawn_timer" && pass "main.gd spawns enemies" || fail "main.gd spawns enemies"

echo ""
echo "=== Runtime (3 pts) ==="
if [[ -f "$DIR/project.godot" ]] && command -v "$GODOT" &>/dev/null; then
  output=$("$GODOT" --path "$DIR" --headless --quit 2>&1 || true)
  echo "$output" | grep -qi "SCRIPT ERROR" && fail "No SCRIPT ERROR" || pass "No SCRIPT ERROR"
  echo "$output" | grep -qi "Failed\|missing\|not found" && fail "No missing resource errors" || pass "No missing resource errors"
  pass "godot --headless --quit ran"
elif [[ -f "$DIR/project.godot" ]]; then
  echo "  [SKIP] Godot binary not found"
  # Still count total for fair comparison
else
  fail "No project.godot"
  fail "No SCRIPT ERROR"
  fail "godot --headless --quit"
fi

echo ""
echo "=== PHASE 1 SCORE: $score / $total ==="

# ---- Phase 2 ----
if [[ "$PHASE" == "2" ]]; then
  echo ""
  echo "============================="
  echo "  PHASE 2: Modify existing"
  echo "============================="
  echo ""

  echo "=== New Scene (4 pts) ==="
  check_file "scenes/pickup.tscn"
  if [[ -f "$DIR/scenes/pickup.tscn" ]]; then
    grep -q "CircleShape2D" "$DIR/scenes/pickup.tscn" && pass "pickup has CircleShape2D" || fail "pickup has CircleShape2D"
    grep -q "collision_layer.*=.*8\|collision_layer = 8" "$DIR/scenes/pickup.tscn" && pass "pickup collision_layer=8" || fail "pickup collision_layer=8"
  else
    fail "pickup has CircleShape2D"
    fail "pickup collision_layer=8"
  fi
  # pickup.gd
  found_pickup_gd=false
  for f in "$DIR"/scenes/pickup.gd "$DIR"/scripts/pickup.gd; do
    if [[ -f "$f" ]]; then
      (grep -q "collected" "$f" && grep -q "queue_free" "$f") && found_pickup_gd=true
    fi
  done
  $found_pickup_gd && pass "pickup.gd has collected signal + queue_free" || fail "pickup.gd has collected signal + queue_free"

  echo ""
  echo "=== main.tscn Modifications (4 pts) ==="
  if [[ -f "$DIR/scenes/main.tscn" ]]; then
    grep -q "PickupTimer" "$DIR/scenes/main.tscn" && pass "PickupTimer added" || fail "PickupTimer added"
    grep -q 'PickupTimer.*timeout\|signal.*timeout.*PickupTimer\|from="PickupTimer"' "$DIR/scenes/main.tscn" && pass "PickupTimer signal connected" || fail "PickupTimer signal connected"
    grep -q 'name="Pickups"' "$DIR/scenes/main.tscn" && pass "Pickups container added" || fail "Pickups container added"
    # Check existing stuff not broken
    (grep -q "Player" "$DIR/scenes/main.tscn" && grep -q "SpawnTimer" "$DIR/scenes/main.tscn") && pass "Existing nodes preserved" || fail "Existing nodes preserved"
  else
    fail "PickupTimer added"; fail "PickupTimer signal"; fail "Pickups container"; fail "Existing nodes preserved"
  fi

  echo ""
  echo "=== main.gd Modifications (3 pts) ==="
  main_gd=""
  for f in "$DIR"/scenes/main.gd "$DIR"/scripts/main.gd; do
    [[ -f "$f" ]] && main_gd="$f" && break
  done
  if [[ -n "$main_gd" ]]; then
    grep -q "pickup" "$main_gd" && pass "main.gd preloads pickup" || fail "main.gd preloads pickup"
    grep -q "_on_pickup_timer" "$main_gd" && pass "main.gd has pickup spawn method" || fail "main.gd has pickup spawn method"
    grep -q "Pickups\|pickups" "$main_gd" && pass "main.gd adds to Pickups container" || fail "main.gd adds to Pickups container"
  else
    fail "main.gd preloads pickup"; fail "main.gd pickup spawn"; fail "main.gd Pickups container"
  fi

  echo ""
  echo "=== hud.tscn Modifications (2 pts) ==="
  if [[ -f "$DIR/scenes/hud.tscn" ]]; then
    grep -q "PickupLabel" "$DIR/scenes/hud.tscn" && pass "PickupLabel added" || fail "PickupLabel added"
    (grep -q "ScoreLabel" "$DIR/scenes/hud.tscn" && grep -q "HealthBar" "$DIR/scenes/hud.tscn") && pass "Existing HUD nodes preserved" || fail "Existing HUD nodes preserved"
  else
    fail "PickupLabel added"; fail "Existing HUD nodes preserved"
  fi

  echo ""
  echo "=== hud.gd Modifications (2 pts) ==="
  hud_gd=""
  for f in "$DIR"/scenes/hud.gd "$DIR"/scripts/hud.gd; do
    [[ -f "$f" ]] && hud_gd="$f" && break
  done
  if [[ -n "$hud_gd" ]]; then
    grep -q "update_pickups" "$hud_gd" && pass "hud.gd has update_pickups" || fail "hud.gd has update_pickups"
    (grep -q "update_score" "$hud_gd" && grep -q "update_health" "$hud_gd") && pass "Existing hud methods preserved" || fail "Existing hud methods preserved"
  else
    fail "hud.gd update_pickups"; fail "Existing hud methods preserved"
  fi

  echo ""
  echo "=== TOTAL SCORE: $score / $total ==="
fi
