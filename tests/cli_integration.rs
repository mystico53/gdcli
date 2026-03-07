mod common;

use std::fs;

use common::{assert_ok_json, cleanup, run_gdcli, run_gdcli_in, setup_temp_project};

#[test]
fn test_scene_create_json() {
    let dir = setup_temp_project("scene_create");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    let output = run_gdcli(&["--json", "scene", "create", sp, "--root-type", "Node2D"]);
    let json = assert_ok_json(&output);
    assert!(json["data"]["path"].as_str().unwrap().contains("test.tscn"));
    assert!(json["data"]["uid"].as_str().unwrap().starts_with("uid://"));
    assert!(scene_path.exists());

    cleanup(&dir);
}

#[test]
fn test_node_add_json() {
    let dir = setup_temp_project("node_add");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);

    let output = run_gdcli(&["--json", "node", "add", sp, "--node-type", "Sprite2D", "--name", "MySprite"]);
    assert_ok_json(&output);

    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("MySprite"));
    assert!(content.contains("Sprite2D"));

    cleanup(&dir);
}

#[test]
fn test_node_add_duplicate_fails() {
    let dir = setup_temp_project("node_dup");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "Sprite2D", "--name", "Dupe"]);

    // Try adding the same node again
    let output = run_gdcli(&["--json", "node", "add", sp, "--node-type", "Sprite2D", "--name", "Dupe"]);
    assert!(!output.status.success(), "Expected failure when adding duplicate node");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("already exists"),
        "Expected 'already exists' error, got: {}",
        combined
    );

    cleanup(&dir);
}

#[test]
fn test_scene_inspect_json() {
    let dir = setup_temp_project("inspect");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "Label", "--name", "MyLabel"]);

    let output = run_gdcli(&["--json", "scene", "inspect", sp]);
    let json = assert_ok_json(&output);

    let nodes = json["data"]["nodes"].as_array().expect("nodes should be an array");
    assert!(nodes.len() >= 2, "Expected at least 2 nodes");

    cleanup(&dir);
}

#[test]
fn test_node_remove() {
    let dir = setup_temp_project("node_remove");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    // Create scene + add node
    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "Sprite2D", "--name", "ToRemove"]);

    // Verify node exists
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("ToRemove"));

    // Remove it
    let output = run_gdcli(&["--json", "node", "remove", sp, "ToRemove"]);
    let json = assert_ok_json(&output);
    assert!(json["data"]["removed_count"].as_u64().unwrap() >= 1);

    // Verify node is gone
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(!content.contains("ToRemove"));

    cleanup(&dir);
}

#[test]
fn test_scene_edit() {
    let dir = setup_temp_project("scene_edit");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "Label", "--name", "MyLabel"]);

    // Edit a property on the label
    let output = run_gdcli(&[
        "--json", "scene", "edit", sp,
        "--set", "MyLabel::text=Hello World",
    ]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["edits"][0]["node"], "MyLabel");
    assert_eq!(json["data"]["edits"][0]["property"], "text");

    // Verify in file
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("text = \"Hello World\""));

    cleanup(&dir);
}

#[test]
fn test_connection_add() {
    let dir = setup_temp_project("conn_add");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "Button", "--name", "MyButton"]);

    let output = run_gdcli(&[
        "--json", "connection", "add", sp,
        "pressed", "MyButton", ".", "_on_button_pressed",
    ]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["signal"], "pressed");
    assert_eq!(json["data"]["from"], "MyButton");
    assert_eq!(json["data"]["to"], ".");
    assert_eq!(json["data"]["method"], "_on_button_pressed");

    // Verify connection in file
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("[connection"));
    assert!(content.contains("signal=\"pressed\""));

    cleanup(&dir);
}

#[test]
fn test_connection_remove() {
    let dir = setup_temp_project("conn_remove");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "Button", "--name", "Btn"]);
    run_gdcli(&["connection", "add", sp, "pressed", "Btn", ".", "_on_pressed"]);

    // Verify connection exists
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("signal=\"pressed\""));

    // Remove it
    let output = run_gdcli(&[
        "--json", "connection", "remove", sp,
        "pressed", "Btn", ".", "_on_pressed",
    ]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["signal"], "pressed");

    // Verify connection is gone
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(!content.contains("[connection"));

    cleanup(&dir);
}

#[test]
fn test_sub_resource_add() {
    let dir = setup_temp_project("subres_add");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "CollisionShape2D", "--name", "Col"]);

    let output = run_gdcli(&[
        "--json", "sub-resource", "add", sp, "RectangleShape2D",
        "--props", "size=Vector2(32, 64)",
        "--wire-node", "Col",
        "--wire-property", "shape",
    ]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["resource_type"], "RectangleShape2D");
    assert!(json["data"]["sub_resource_id"].as_str().is_some());

    // Verify sub_resource in file
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("[sub_resource"));
    assert!(content.contains("RectangleShape2D"));
    assert!(content.contains("SubResource("));

    cleanup(&dir);
}

#[test]
fn test_sub_resource_edit() {
    let dir = setup_temp_project("subres_edit");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", sp, "--node-type", "CollisionShape2D", "--name", "Col"]);

    // Add a sub-resource first
    let add_output = run_gdcli(&[
        "--json", "sub-resource", "add", sp, "RectangleShape2D",
        "--wire-node", "Col",
        "--wire-property", "shape",
    ]);
    let add_json = assert_ok_json(&add_output);
    let sub_id = add_json["data"]["sub_resource_id"].as_str().unwrap();

    // Edit its property
    let output = run_gdcli(&[
        "--json", "sub-resource", "edit", sp, sub_id,
        "--set", "size=Vector2(100, 200)",
    ]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["sub_resource_id"], sub_id);

    // Verify in file
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("Vector2(100, 200)"));

    cleanup(&dir);
}

#[test]
fn test_scene_list() {
    let dir = setup_temp_project("scene_list");

    // Create multiple scenes (use absolute paths)
    let s1 = dir.join("a.tscn");
    let s2 = dir.join("b.tscn");
    run_gdcli(&["scene", "create", s1.to_str().unwrap(), "--root-type", "Node2D"]);
    run_gdcli(&["scene", "create", s2.to_str().unwrap(), "--root-type", "Node3D"]);

    // Run scene list from the project directory
    let output = run_gdcli_in(&dir, &["--json", "scene", "list"]);
    let json = assert_ok_json(&output);
    let total = json["data"]["total"].as_u64().unwrap();
    assert!(total >= 2, "Expected at least 2 scenes, got {}", total);

    let scenes = json["data"]["scenes"].as_array().unwrap();
    let paths: Vec<&str> = scenes.iter().map(|s| s["path"].as_str().unwrap()).collect();
    assert!(paths.iter().any(|p| p.contains("a.tscn")));
    assert!(paths.iter().any(|p| p.contains("b.tscn")));

    cleanup(&dir);
}

#[test]
fn test_scene_validate_clean() {
    let dir = setup_temp_project("scene_validate");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);

    let output = run_gdcli(&["--json", "scene", "validate", sp]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["issue_count"], 0);

    cleanup(&dir);
}

#[test]
fn test_script_create() {
    let dir = setup_temp_project("script_create");
    let script_path = dir.join("player.gd");
    let sp = script_path.to_str().unwrap();

    let output = run_gdcli(&[
        "--json", "script", "create", sp,
        "--extends", "CharacterBody2D",
        "--methods", "_ready,_process",
    ]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["extends"], "CharacterBody2D");

    // Verify file contents
    let content = fs::read_to_string(&script_path).unwrap();
    assert!(content.contains("extends CharacterBody2D"));
    assert!(content.contains("func _ready()"));
    assert!(content.contains("func _process(delta: float)"));

    cleanup(&dir);
}

#[test]
fn test_uid_fix_no_changes() {
    let dir = setup_temp_project("uid_fix");
    let scene_path = dir.join("test.tscn");
    run_gdcli(&["scene", "create", scene_path.to_str().unwrap(), "--root-type", "Node2D"]);

    // Run uid fix from project dir — no .uid files means no fixes
    let output = run_gdcli_in(&dir, &["--json", "uid", "fix", "--dry-run"]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["fix_count"], 0);
    assert_eq!(json["data"]["dry_run"], true);

    cleanup(&dir);
}

#[test]
fn test_no_project_godot_error() {
    let dir = std::env::temp_dir().join("gdcli_cli_test_no_project");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    // No project.godot created

    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    let output = run_gdcli(&["--json", "scene", "create", sp, "--root-type", "Node2D"]);
    assert!(!output.status.success(), "Expected failure without project.godot");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.to_lowercase().contains("project.godot")
            || combined.to_lowercase().contains("no godot project"),
        "Expected project.godot error, got: {}",
        combined
    );

    cleanup(&dir);
}

#[test]
fn test_load_sprite() {
    let dir = setup_temp_project("load_sprite");
    let scene_path = dir.join("test.tscn");
    let sp = scene_path.to_str().unwrap();

    // Create scene
    run_gdcli(&["scene", "create", sp, "--root-type", "Node2D"]);

    // Create a dummy texture file (content doesn't matter, only existence is checked)
    fs::write(dir.join("icon.svg"), b"<svg></svg>").unwrap();

    // Load sprite
    let output = run_gdcli(&["--json", "load-sprite", sp, "MySprite", "res://icon.svg"]);
    let json = assert_ok_json(&output);
    assert_eq!(json["data"]["sprite_type"], "Sprite2D");
    assert_eq!(json["data"]["node_name"], "MySprite");
    assert_eq!(json["data"]["texture_path"], "res://icon.svg");

    // Verify file contents
    let content = fs::read_to_string(&scene_path).unwrap();
    assert!(content.contains("[ext_resource"), "Missing ext_resource");
    assert!(content.contains("icon.svg"), "Missing icon.svg in ext_resource");
    assert!(content.contains("MySprite"), "Missing MySprite node");
    assert!(content.contains("Sprite2D"), "Missing Sprite2D type");
    assert!(content.contains("ExtResource("), "Missing ExtResource( in texture property");

    cleanup(&dir);
}
