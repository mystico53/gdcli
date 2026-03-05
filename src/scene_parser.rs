use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ParsedScene {
    pub uid: Option<String>,
    pub format: u32,
    pub ext_resources: Vec<ExtResource>,
    pub sub_resources: Vec<SubResource>,
    pub nodes: Vec<SceneNode>,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtResource {
    pub id: String,
    pub resource_type: String,
    pub path: String,
    pub uid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubResource {
    pub id: String,
    pub resource_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneNode {
    pub name: String,
    pub node_type: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub signal: String,
    pub from: String,
    pub to: String,
    pub method: String,
}

/// Parse a .tscn file and return its structure.
pub fn parse_scene(path: &Path) -> anyhow::Result<ParsedScene> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    parse_scene_text(&content)
}

/// Parse .tscn text content.
pub fn parse_scene_text(content: &str) -> anyhow::Result<ParsedScene> {
    let mut uid = None;
    let mut format = 3;
    let mut ext_resources = Vec::new();
    let mut sub_resources = Vec::new();
    let mut nodes = Vec::new();
    let mut connections = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Scene header: [gd_scene load_steps=2 format=3 uid="uid://xxx"]
        if trimmed.starts_with("[gd_scene ") {
            uid = extract_quoted_attr(trimmed, "uid");
            if let Some(fmt) = extract_attr(trimmed, "format") {
                format = fmt.parse().unwrap_or(3);
            }
        }

        // External resource: [ext_resource type="Script" path="res://..." id="1_abc"]
        if trimmed.starts_with("[ext_resource ") {
            let resource_type = extract_quoted_attr(trimmed, "type").unwrap_or_default();
            let path = extract_quoted_attr(trimmed, "path").unwrap_or_default();
            let id = extract_quoted_attr(trimmed, "id").unwrap_or_default();
            let res_uid = extract_quoted_attr(trimmed, "uid");

            ext_resources.push(ExtResource {
                id,
                resource_type,
                path,
                uid: res_uid,
            });
        }

        // Sub-resource: [sub_resource type="Animation" id="Animation_abc"]
        if trimmed.starts_with("[sub_resource ") {
            let resource_type = extract_quoted_attr(trimmed, "type").unwrap_or_default();
            let id = extract_quoted_attr(trimmed, "id").unwrap_or_default();

            sub_resources.push(SubResource { id, resource_type });
        }

        // Node: [node name="Name" type="Type" parent="."]
        if trimmed.starts_with("[node ") {
            let name = extract_quoted_attr(trimmed, "name").unwrap_or_default();
            let node_type = extract_quoted_attr(trimmed, "type").unwrap_or_default();
            let parent = extract_quoted_attr(trimmed, "parent");

            nodes.push(SceneNode {
                name,
                node_type,
                parent,
            });
        }

        // Connection: [connection signal="pressed" from="..." to="." method="..."]
        if trimmed.starts_with("[connection ") {
            let signal = extract_quoted_attr(trimmed, "signal").unwrap_or_default();
            let from = extract_quoted_attr(trimmed, "from").unwrap_or_default();
            let to = extract_quoted_attr(trimmed, "to").unwrap_or_default();
            let method = extract_quoted_attr(trimmed, "method").unwrap_or_default();

            connections.push(Connection {
                signal,
                from,
                to,
                method,
            });
        }
    }

    Ok(ParsedScene {
        uid,
        format,
        ext_resources,
        sub_resources,
        nodes,
        connections,
    })
}

/// Extract a quoted attribute value from a bracket line.
/// e.g. extract_quoted_attr(`[node name="Foo" type="Bar"]`, "name") -> Some("Foo")
fn extract_quoted_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = line.find(&pattern)? + pattern.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Extract an unquoted attribute value from a bracket line.
/// e.g. extract_attr(`[gd_scene format=3]`, "format") -> Some("3")
fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=", attr);
    let start = line.find(&pattern)? + pattern.len();
    let rest = &line[start..];
    // Value ends at space or ]
    let end = rest.find([' ', ']']).unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// Recursively find all .tscn files, skipping hidden dirs and .godot/.
pub fn find_scene_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut scenes = Vec::new();
    find_scenes_recursive(dir, &mut scenes);
    scenes.sort();
    scenes
}

fn find_scenes_recursive(dir: &Path, results: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            find_scenes_recursive(&path, results);
        } else if path.extension().is_some_and(|ext| ext == "tscn") {
            results.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_scene() {
        let content = r#"[gd_scene load_steps=2 format=3 uid="uid://cg3hylang5fxn"]

[ext_resource type="Script" uid="uid://bv6y7in6otgcm" path="res://main.gd" id="1_j0gfq"]

[node name="Main" type="Node2D"]
script = ExtResource("1_j0gfq")

[node name="Child" type="Button" parent="."]
text = "Click me"

[connection signal="pressed" from="Child" to="." method="_on_pressed"]
"#;
        let scene = parse_scene_text(content).unwrap();
        assert_eq!(scene.uid.as_deref(), Some("uid://cg3hylang5fxn"));
        assert_eq!(scene.format, 3);
        assert_eq!(scene.ext_resources.len(), 1);
        assert_eq!(scene.ext_resources[0].path, "res://main.gd");
        assert_eq!(
            scene.ext_resources[0].uid.as_deref(),
            Some("uid://bv6y7in6otgcm")
        );
        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.nodes[0].name, "Main");
        assert_eq!(scene.nodes[0].node_type, "Node2D");
        assert_eq!(scene.nodes[0].parent, None);
        assert_eq!(scene.nodes[1].name, "Child");
        assert_eq!(scene.nodes[1].parent.as_deref(), Some("."));
        assert_eq!(scene.connections.len(), 1);
        assert_eq!(scene.connections[0].signal, "pressed");
    }

    #[test]
    fn test_extract_quoted_attr() {
        let line = r#"[node name="Main" type="Node2D" parent="."]"#;
        assert_eq!(extract_quoted_attr(line, "name").as_deref(), Some("Main"));
        assert_eq!(extract_quoted_attr(line, "type").as_deref(), Some("Node2D"));
        assert_eq!(extract_quoted_attr(line, "parent").as_deref(), Some("."));
        assert_eq!(extract_quoted_attr(line, "missing"), None);
    }
}
