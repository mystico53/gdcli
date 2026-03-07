use std::path::Path;

// Import the library crate's public API
use gdcli::scene_parser::{parse_scene, write_scene, parse_scene_text};

#[test]
fn test_parse_minimal_fixture() {
    let path = Path::new("tests/fixtures/minimal.tscn");
    let scene = parse_scene(path).expect("Failed to parse minimal.tscn");

    assert_eq!(scene.uid.as_deref(), Some("uid://minimal_test"));
    assert_eq!(scene.format, 3);
    assert_eq!(scene.ext_resources.len(), 0);
    assert_eq!(scene.sub_resources.len(), 0);
    assert_eq!(scene.nodes.len(), 1);
    assert_eq!(scene.nodes[0].name, "Root");
    assert_eq!(scene.nodes[0].node_type, "Node2D");
    assert!(scene.nodes[0].parent.is_none());
    assert_eq!(scene.connections.len(), 0);
}

#[test]
fn test_parse_complex_fixture() {
    let path = Path::new("tests/fixtures/complex.tscn");
    let scene = parse_scene(path).expect("Failed to parse complex.tscn");

    assert_eq!(scene.uid.as_deref(), Some("uid://complex_test"));
    assert_eq!(scene.format, 3);
    assert_eq!(scene.ext_resources.len(), 2);
    assert_eq!(scene.sub_resources.len(), 1);
    assert_eq!(scene.nodes.len(), 6);
    assert_eq!(scene.connections.len(), 2);

    // Verify ext_resources
    assert_eq!(scene.ext_resources[0].resource_type, "Script");
    assert_eq!(scene.ext_resources[0].path, "res://player.gd");
    assert_eq!(
        scene.ext_resources[0].uid.as_deref(),
        Some("uid://script_abc")
    );
    assert_eq!(scene.ext_resources[1].resource_type, "PackedScene");
    assert_eq!(scene.ext_resources[1].path, "res://scenes/enemy.tscn");

    // Verify sub_resources
    assert_eq!(scene.sub_resources[0].resource_type, "RectangleShape2D");
    assert_eq!(scene.sub_resources[0].properties.len(), 1);
    assert_eq!(scene.sub_resources[0].properties[0].key, "size");

    // Verify nodes
    assert_eq!(scene.nodes[0].name, "World");
    assert_eq!(scene.nodes[1].name, "Player");
    assert_eq!(scene.nodes[1].parent.as_deref(), Some("."));
    assert_eq!(scene.nodes[2].name, "CollisionShape");
    assert_eq!(scene.nodes[2].parent.as_deref(), Some("Player"));
    assert_eq!(scene.nodes[5].name, "Enemy1");
    assert!(scene.nodes[5].instance.is_some());

    // Verify connections
    assert_eq!(scene.connections[0].signal, "body_entered");
    assert_eq!(scene.connections[0].from, "Player");
    assert_eq!(scene.connections[1].signal, "tree_exited");
    assert_eq!(scene.connections[1].from, "Enemy1");
}

#[test]
fn test_parse_no_uid_fixture() {
    let path = Path::new("tests/fixtures/no_uid.tscn");
    let scene = parse_scene(path).expect("Failed to parse no_uid.tscn");

    assert_eq!(scene.uid, None);
    assert_eq!(scene.format, 3);
    assert_eq!(scene.nodes.len(), 2);
    assert_eq!(scene.nodes[0].name, "Legacy");
    assert_eq!(scene.nodes[1].name, "Child");
    assert_eq!(scene.nodes[1].properties.len(), 1);
    assert_eq!(scene.nodes[1].properties[0].key, "text");
}

#[test]
fn test_roundtrip_minimal_fixture() {
    let path = Path::new("tests/fixtures/minimal.tscn");
    let scene1 = parse_scene(path).unwrap();
    let written = write_scene(&scene1);
    let scene2 = parse_scene_text(&written).unwrap();

    assert_eq!(scene1.uid, scene2.uid);
    assert_eq!(scene1.nodes.len(), scene2.nodes.len());
    assert_eq!(scene1.nodes[0].name, scene2.nodes[0].name);
}

#[test]
fn test_roundtrip_complex_fixture() {
    let path = Path::new("tests/fixtures/complex.tscn");
    let scene1 = parse_scene(path).unwrap();
    let written = write_scene(&scene1);
    let scene2 = parse_scene_text(&written).unwrap();

    assert_eq!(scene1.uid, scene2.uid);
    assert_eq!(scene1.ext_resources.len(), scene2.ext_resources.len());
    assert_eq!(scene1.sub_resources.len(), scene2.sub_resources.len());
    assert_eq!(scene1.nodes.len(), scene2.nodes.len());
    assert_eq!(scene1.connections.len(), scene2.connections.len());

    for (a, b) in scene1.nodes.iter().zip(scene2.nodes.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.node_type, b.node_type);
        assert_eq!(a.parent, b.parent);
        assert_eq!(a.instance, b.instance);
    }
}

#[test]
fn test_roundtrip_no_uid_fixture() {
    let path = Path::new("tests/fixtures/no_uid.tscn");
    let scene1 = parse_scene(path).unwrap();
    let written = write_scene(&scene1);
    let scene2 = parse_scene_text(&written).unwrap();

    assert_eq!(scene1.uid, scene2.uid);
    assert_eq!(scene1.nodes.len(), scene2.nodes.len());
    for (a, b) in scene1.nodes.iter().zip(scene2.nodes.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.properties.len(), b.properties.len());
    }
}

#[test]
fn test_all_types_roundtrip() {
    let path = Path::new("tests/fixtures/all_types.tscn");
    let scene1 = parse_scene(path).expect("Failed to parse all_types.tscn");
    let written = write_scene(&scene1);
    let scene2 = parse_scene_text(&written).expect("Failed to re-parse written all_types");

    assert_eq!(scene1.uid, scene2.uid);
    assert_eq!(scene1.ext_resources.len(), scene2.ext_resources.len());
    assert_eq!(scene1.sub_resources.len(), scene2.sub_resources.len());
    assert_eq!(scene1.nodes.len(), scene2.nodes.len());
    assert_eq!(scene1.connections.len(), scene2.connections.len());

    for (a, b) in scene1.nodes.iter().zip(scene2.nodes.iter()) {
        assert_eq!(a.name, b.name, "node name mismatch");
        assert_eq!(a.node_type, b.node_type, "node type mismatch for {}", a.name);
        assert_eq!(a.parent, b.parent, "parent mismatch for {}", a.name);
        assert_eq!(a.instance, b.instance, "instance mismatch for {}", a.name);
        assert_eq!(
            a.properties.len(),
            b.properties.len(),
            "property count mismatch for node {}",
            a.name
        );
        for (pa, pb) in a.properties.iter().zip(b.properties.iter()) {
            assert_eq!(pa.key, pb.key, "property key mismatch in node {}", a.name);
            assert_eq!(
                pa.value, pb.value,
                "property value mismatch for {}.{}: {:?} vs {:?}",
                a.name, pa.key, pa.value, pb.value
            );
        }
    }

    for (a, b) in scene1.sub_resources.iter().zip(scene2.sub_resources.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.resource_type, b.resource_type);
        for (pa, pb) in a.properties.iter().zip(b.properties.iter()) {
            assert_eq!(pa.key, pb.key);
            assert_eq!(pa.value, pb.value, "sub_resource prop mismatch for {}.{}", a.id, pa.key);
        }
    }
}

#[test]
fn test_all_types_value_exact() {
    let path = Path::new("tests/fixtures/all_types.tscn");
    let scene = parse_scene(path).expect("Failed to parse all_types.tscn");

    let root = &scene.nodes[0];
    assert_eq!(root.name, "Root");

    let expected: &[(&str, &str)] = &[
        ("bool_val", "true"),
        ("bool_false", "false"),
        ("int_val", "42"),
        ("neg_int", "-7"),
        ("float_val", "3.14"),
        ("sci_float", "1.5e-4"),
        ("string_val", "\"hello world\""),
        ("empty_string", "\"\""),
        ("string_with_escapes", "\"line1\\nline2\""),
        ("string_name", "&\"my_name\""),
        ("node_path", "NodePath(\"Player/Sprite\")"),
        ("empty_path", "NodePath(\"\")"),
        ("vec2", "Vector2(1.5, -2.0)"),
        ("vec2i", "Vector2i(3, 4)"),
        ("vec3", "Vector3(1, 2, 3)"),
        ("vec3i", "Vector3i(-1, 0, 1)"),
        ("vec4", "Vector4(1, 2, 3, 4)"),
        ("vec4i", "Vector4i(0, 0, 0, 1)"),
        ("rect2", "Rect2(0, 0, 100, 50)"),
        ("rect2i", "Rect2i(10, 20, 30, 40)"),
        ("xform2d", "Transform2D(1, 0, 0, 1, 100, 200)"),
        ("xform3d", "Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 10, 20, 30)"),
        ("plane", "Plane(0, 1, 0, 0.5)"),
        ("quat", "Quaternion(0, 0, 0, 1)"),
        ("aabb", "AABB(0, 0, 0, 1, 1, 1)"),
        ("basis", "Basis(1, 0, 0, 0, 1, 0, 0, 0, 1)"),
        ("projection", "Projection(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)"),
        ("color", "Color(1, 0.5, 0, 1)"),
        ("color_transparent", "Color(0, 0, 0, 0)"),
        ("packed_byte", "PackedByteArray(1, 2, 255)"),
        ("packed_i32", "PackedInt32Array(-1, 0, 42)"),
        ("packed_i64", "PackedInt64Array(100, 200)"),
        ("packed_f32", "PackedFloat32Array(1.0, 2.5, 3.14)"),
        ("packed_f64", "PackedFloat64Array(1.0, 2.0)"),
        ("packed_str", "PackedStringArray(\"a\", \"b\", \"c\")"),
        ("packed_vec2", "PackedVector2Array(0, 0, 1, 1)"),
        ("packed_vec3", "PackedVector3Array(0, 0, 0, 1, 1, 1)"),
        ("packed_color", "PackedColorArray(1, 0, 0, 1, 0, 1, 0, 1)"),
        ("packed_vec4", "PackedVector4Array(1, 2, 3, 4)"),
        ("array", "[1, \"two\", Vector2(3, 4)]"),
        ("dict", "{\"key\": \"value\", \"num\": 42}"),
        ("empty_array", "[]"),
        ("empty_dict", "{}"),
        ("ext_ref", "ExtResource(\"1_abc\")"),
        ("sub_ref", "SubResource(\"Shape_xyz\")"),
        ("rid", "RID()"),
        ("script", "ExtResource(\"1_abc\")"),
    ];

    for (key, expected_val) in expected {
        let prop = root.properties.iter().find(|p| p.key == *key);
        assert!(prop.is_some(), "missing property: {}", key);
        assert_eq!(
            prop.unwrap().value, *expected_val,
            "value mismatch for property '{}'",
            key
        );
    }
}

#[test]
fn test_no_panics_on_fixtures() {
    let fixtures = &[
        "tests/fixtures/minimal.tscn",
        "tests/fixtures/complex.tscn",
        "tests/fixtures/no_uid.tscn",
        "tests/fixtures/all_types.tscn",
    ];
    for fixture in fixtures {
        let path = Path::new(fixture);
        let result = parse_scene(path);
        assert!(
            result.is_ok(),
            "Parsing {} panicked or errored: {:?}",
            fixture,
            result.err()
        );
    }
}
