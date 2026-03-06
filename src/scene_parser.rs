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
    /// Raw text content — preserved for faithful round-tripping.
    #[serde(skip)]
    #[allow(dead_code)]
    pub raw: Option<String>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<NodeProperty>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneNode {
    pub name: String,
    pub node_type: String,
    pub parent: Option<String>,
    /// Instance reference for nodes that are instanced scenes (e.g. `ExtResource("2_abc")`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Properties like `script = ExtResource("1_abc")`, `position = Vector2(0, 0)`, etc.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<NodeProperty>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeProperty {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub signal: String,
    pub from: String,
    pub to: String,
    pub method: String,
}

/// Verify that `path` has a `.tscn` or `.tres` extension.
/// Prevents accidental corruption of non-scene files (e.g. project.godot).
pub fn require_scene_file(path: &Path) -> anyhow::Result<()> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("tscn" | "tres") => Ok(()),
        _ => anyhow::bail!(
            "Expected a .tscn or .tres file, got: {}\n\
             Scene/node operations only work on Godot scene and resource files.",
            path.display()
        ),
    }
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

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Scene header: [gd_scene load_steps=2 format=3 uid="uid://xxx"]
        if trimmed.starts_with("[gd_scene ") {
            uid = extract_quoted_attr(trimmed, "uid");
            if let Some(fmt) = extract_attr(trimmed, "format") {
                format = fmt.parse().unwrap_or(3);
            }
            i += 1;
            continue;
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
            i += 1;
            continue;
        }

        // Sub-resource: [sub_resource type="Animation" id="Animation_abc"]
        if trimmed.starts_with("[sub_resource ") {
            let resource_type = extract_quoted_attr(trimmed, "type").unwrap_or_default();
            let id = extract_quoted_attr(trimmed, "id").unwrap_or_default();

            // Collect properties (lines after [sub_resource ...] until next section or blank line)
            let mut properties = Vec::new();
            i += 1;
            while i < lines.len() {
                let prop_line = lines[i].trim();
                if prop_line.is_empty() || prop_line.starts_with('[') {
                    break;
                }
                if let Some(eq_pos) = prop_line.find(" = ") {
                    let key = prop_line[..eq_pos].to_string();
                    let value = prop_line[eq_pos + 3..].to_string();
                    properties.push(NodeProperty { key, value });
                }
                i += 1;
            }

            sub_resources.push(SubResource {
                id,
                resource_type,
                properties,
            });
            continue;
        }

        // Node: [node name="Name" type="Type" parent="." instance=ExtResource("id")]
        if trimmed.starts_with("[node ") {
            let name = extract_quoted_attr(trimmed, "name").unwrap_or_default();
            let node_type = extract_quoted_attr(trimmed, "type").unwrap_or_default();
            let parent = extract_quoted_attr(trimmed, "parent");
            let instance = extract_instance_attr(trimmed);

            // Collect properties (lines after [node ...] until next section or blank line)
            let mut properties = Vec::new();
            i += 1;
            while i < lines.len() {
                let prop_line = lines[i].trim();
                if prop_line.is_empty() || prop_line.starts_with('[') {
                    break;
                }
                if let Some(eq_pos) = prop_line.find(" = ") {
                    let key = prop_line[..eq_pos].to_string();
                    let value = prop_line[eq_pos + 3..].to_string();
                    properties.push(NodeProperty { key, value });
                }
                i += 1;
            }

            nodes.push(SceneNode {
                name,
                node_type,
                parent,
                instance,
                properties,
            });
            continue;
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

        i += 1;
    }

    Ok(ParsedScene {
        uid,
        format,
        ext_resources,
        sub_resources,
        nodes,
        connections,
        raw: Some(content.to_string()),
    })
}

/// Derive a PascalCase node name from a filename.
/// e.g. "enemy.tscn" → "Enemy", "game_over.tscn" → "GameOver", "my-level.tscn" → "MyLevel"
pub fn filename_to_node_name(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    stem.split(['_', '-'])
        .filter(|s| !s.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Generate a minimal .tscn scene with just a root node.
/// `root_name` is used for the node's name, `root_type` for its type.
/// If `script` is provided, adds an ext_resource for it and attaches it to the root node.
pub fn generate_minimal_scene(
    root_type: &str,
    root_name: &str,
    uid: &str,
    script: Option<&str>,
) -> String {
    if let Some(script_path) = script {
        let ext_id = generate_ext_resource_id(0);
        format!(
            "[gd_scene load_steps=2 format=3 uid=\"{}\"]\n\n\
             [ext_resource type=\"Script\" path=\"{}\" id=\"{}\"]\n\n\
             [node name=\"{}\" type=\"{}\"]\n\
             script = ExtResource(\"{}\")\n",
            uid, script_path, ext_id, root_name, root_type, ext_id
        )
    } else {
        format!(
            "[gd_scene format=3 uid=\"{}\"]\n\n[node name=\"{}\" type=\"{}\"]\n",
            uid, root_name, root_type
        )
    }
}

/// Generate a Godot-style UID like "uid://abc123xyz".
/// Uses random bytes encoded in a base62-like alphabet matching Godot's format.
pub fn generate_uid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    // Simple LCG PRNG seeded from current time — good enough for UIDs
    let mut state = seed;
    let charset = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut id = String::with_capacity(13);

    for _ in 0..13 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = ((state >> 33) as usize) % charset.len();
        id.push(charset[idx] as char);
    }

    format!("uid://{}", id)
}

/// Serialize a ParsedScene back to .tscn text.
#[allow(dead_code)]
pub fn write_scene(scene: &ParsedScene) -> String {
    let mut out = String::new();

    // Header
    let load_steps = scene.ext_resources.len() + scene.sub_resources.len();
    if load_steps > 0 {
        out.push_str(&format!(
            "[gd_scene load_steps={} format={}",
            load_steps + 1,
            scene.format
        ));
    } else {
        out.push_str(&format!("[gd_scene format={}", scene.format));
    }
    if let Some(ref uid) = scene.uid {
        out.push_str(&format!(" uid=\"{}\"", uid));
    }
    out.push_str("]\n");

    // Ext resources
    for ext in &scene.ext_resources {
        out.push('\n');
        out.push_str(&format!("[ext_resource type=\"{}\"", ext.resource_type));
        if let Some(ref uid) = ext.uid {
            out.push_str(&format!(" uid=\"{}\"", uid));
        }
        out.push_str(&format!(" path=\"{}\" id=\"{}\"]", ext.path, ext.id));
        out.push('\n');
    }

    // Sub resources
    for sub in &scene.sub_resources {
        out.push('\n');
        out.push_str(&format!(
            "[sub_resource type=\"{}\" id=\"{}\"]\n",
            sub.resource_type, sub.id
        ));
        for prop in &sub.properties {
            out.push_str(&format!("{} = {}\n", prop.key, prop.value));
        }
    }

    // Nodes
    for node in &scene.nodes {
        out.push('\n');
        out.push_str(&format!("[node name=\"{}\"", node.name));
        if !node.node_type.is_empty() {
            out.push_str(&format!(" type=\"{}\"", node.node_type));
        }
        if let Some(ref parent) = node.parent {
            out.push_str(&format!(" parent=\"{}\"", parent));
        }
        if let Some(ref inst) = node.instance {
            out.push_str(&format!(" instance={}", inst));
        }
        out.push_str("]\n");

        for prop in &node.properties {
            out.push_str(&format!("{} = {}\n", prop.key, prop.value));
        }
    }

    // Connections
    for conn in &scene.connections {
        out.push('\n');
        out.push_str(&format!(
            "[connection signal=\"{}\" from=\"{}\" to=\"{}\" method=\"{}\"]\n",
            conn.signal, conn.from, conn.to, conn.method
        ));
    }

    out
}

/// Compute the node path for a given node within the scene tree.
/// Root node (no parent) returns ".", children of root return their name,
/// deeper nodes return "ParentName/ChildName".
#[allow(dead_code)]
pub fn node_path(scene: &ParsedScene, node_name: &str) -> Option<String> {
    // Find the node
    let node = scene.nodes.iter().find(|n| n.name == node_name)?;
    if let Some(parent) = &node.parent {
        Some(parent.clone() + "/" + &node.name)
    } else {
        // Root node
        Some(".".to_string())
    }
}

/// Get the tree path to use as a parent reference for children of the given node.
/// For the root node, returns "."; for others, builds the full path.
pub fn parent_path_for(scene: &ParsedScene, node_name: &str) -> Option<String> {
    let node = scene.nodes.iter().find(|n| n.name == node_name)?;
    if let Some(parent) = &node.parent {
        if parent == "." {
            Some(node.name.clone())
        } else {
            Some(format!("{}/{}", parent, node.name))
        }
    } else {
        // This is the root — children use "." as parent
        Some(".".to_string())
    }
}

/// Generate a short random ext_resource ID like "1_abc5x".
fn generate_ext_resource_id(index: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let charset = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut state = seed.wrapping_add(index as u128);
    let mut suffix = String::with_capacity(5);

    for _ in 0..5 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = ((state >> 33) as usize) % charset.len();
        suffix.push(charset[idx] as char);
    }

    format!("{}_{}", index + 1, suffix)
}

/// Generate a short random sub_resource ID like "TypeName_abc5x".
fn generate_sub_resource_id(resource_type: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let charset = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut state = seed;
    let mut suffix = String::with_capacity(5);

    for _ in 0..5 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = ((state >> 33) as usize) % charset.len();
        suffix.push(charset[idx] as char);
    }

    format!("{}_{}", resource_type, suffix)
}

/// Find insertion position for new sub_resources.
/// After last [sub_resource] block, or after last [ext_resource], or after header.
fn find_sub_resource_insert_pos(content: &str) -> usize {
    let mut last_sub_end = None;
    let mut last_ext_end = None;
    let mut header_end = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("[gd_scene ") {
            header_end = Some(i);
        }
        if trimmed.starts_with("[ext_resource ") {
            last_ext_end = Some(i);
        }
        if trimmed.starts_with("[sub_resource ") {
            // Skip past properties to find end of this sub_resource block
            i += 1;
            while i < lines.len() {
                let prop_line = lines[i].trim();
                if prop_line.is_empty() || prop_line.starts_with('[') {
                    break;
                }
                i += 1;
            }
            last_sub_end = Some(i - 1);
            // Check if we stopped on a blank line — include it in the position
            if i < lines.len() && lines[i].trim().is_empty() {
                last_sub_end = Some(i);
            }
            continue;
        }
        i += 1;
    }

    let target_line = last_sub_end.or(last_ext_end).or(header_end).unwrap_or(0);

    // Find byte position at end of target line
    let mut byte_pos = 0;
    for (idx, line) in content.lines().enumerate() {
        byte_pos += line.len() + 1; // +1 for newline
        if idx == target_line {
            return byte_pos;
        }
    }
    content.len()
}

/// Add a sub_resource to a .tscn file.
/// If `wire_node` and `wire_property` are both provided, sets the node's property
/// to `SubResource("id")` referencing the newly created sub_resource.
/// Returns the generated sub_resource ID.
pub fn add_sub_resource_to_file(
    path: &Path,
    resource_type: &str,
    props: &[(String, String)],
    wire_node: Option<&str>,
    wire_property: Option<&str>,
) -> anyhow::Result<String> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let sub_id = generate_sub_resource_id(resource_type);

    // Build the sub_resource section
    let mut section = format!(
        "\n[sub_resource type=\"{}\" id=\"{}\"]\n",
        resource_type, sub_id
    );
    for (key, value) in props {
        section.push_str(&format!("{} = {}\n", key, value));
    }

    let insert_pos = find_sub_resource_insert_pos(&content);
    let mut new_content = content;
    new_content.insert_str(insert_pos, &section);
    new_content = update_load_steps(&new_content);

    atomic_write(path, &new_content)?;

    // Wire to a node if requested
    if let (Some(node), Some(prop)) = (wire_node, wire_property) {
        let wire_value = format!("SubResource(\"{}\")", sub_id);
        edit_node_property(path, node, prop, &wire_value)?;
    }

    Ok(sub_id)
}

/// Edit a property on an existing sub_resource by ID.
pub fn edit_sub_resource_property(
    path: &Path,
    sub_id: &str,
    property: &str,
    value: &str,
) -> anyhow::Result<()> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    // Verify sub_resource exists
    let scene = parse_scene_text(&content)?;
    if !scene.sub_resources.iter().any(|s| s.id == sub_id) {
        anyhow::bail!("SubResource '{}' not found in scene", sub_id);
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    let mut found_and_set = false;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("[sub_resource ") {
            let id = extract_quoted_attr(trimmed, "id").unwrap_or_default();
            result.push(lines[i].to_string());
            i += 1;

            if id == sub_id {
                // Inside target sub_resource — look for existing property
                let mut property_set = false;
                while i < lines.len() {
                    let prop_trimmed = lines[i].trim();
                    if prop_trimmed.is_empty() || prop_trimmed.starts_with('[') {
                        break;
                    }
                    if prop_trimmed.starts_with(&format!("{} = ", property)) {
                        result.push(format!("{} = {}", property, value));
                        property_set = true;
                        found_and_set = true;
                    } else {
                        result.push(lines[i].to_string());
                    }
                    i += 1;
                }
                if !property_set {
                    // Add new property
                    result.push(format!("{} = {}", property, value));
                    found_and_set = true;
                }
                continue;
            }
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    if !found_and_set {
        anyhow::bail!(
            "Failed to set property '{}' on sub_resource '{}'",
            property,
            sub_id
        );
    }

    let new_content = result.join("\n");
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        new_content + "\n"
    };

    atomic_write(path, &new_content)
}

/// Add a node to a parsed scene. Returns the modified scene.
/// Operates on the raw text for faithful round-tripping.
///
/// Either `node_type` or `instance_path` must be provided:
/// - `node_type`: creates a typed node (e.g. `Sprite2D`)
/// - `instance_path`: creates an instanced scene node (e.g. `res://scenes/enemy.tscn`)
pub fn add_node_to_file(
    path: &Path,
    node_type: Option<&str>,
    node_name: &str,
    parent: Option<&str>,
    script: Option<&str>,
    props: &[(String, String)],
    instance_path: Option<&str>,
) -> anyhow::Result<()> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let scene = parse_scene_text(&content)?;

    // Check for duplicate node name
    if scene.nodes.iter().any(|n| n.name == node_name) {
        anyhow::bail!("Node '{}' already exists in scene", node_name);
    }

    // Determine parent path
    let parent_ref = if let Some(p) = parent {
        // Find the parent node and compute its path for child references
        if p == "." || scene.nodes.first().map(|n| n.name.as_str()) == Some(p) {
            ".".to_string()
        } else {
            parent_path_for(&scene, p)
                .ok_or_else(|| anyhow::anyhow!("Parent node '{}' not found in scene", p))?
        }
    } else {
        // Default to root
        ".".to_string()
    };

    let mut new_content = content.clone();
    let mut ext_resource_count = scene.ext_resources.len();

    // If instance_path is specified, add a PackedScene ext_resource
    let instance_ext_id = if let Some(inst_path) = instance_path {
        let ext_id = generate_ext_resource_id(ext_resource_count);
        ext_resource_count += 1;
        let ext_line = format!(
            "\n[ext_resource type=\"PackedScene\" path=\"{}\" id=\"{}\"]\n",
            inst_path, ext_id
        );

        let insert_pos = find_ext_resource_insert_pos(&new_content);
        new_content.insert_str(insert_pos, &ext_line);
        new_content = update_load_steps(&new_content);

        Some(ext_id)
    } else {
        None
    };

    // If script is specified (only for typed nodes, not instanced), add an ext_resource for it
    let script_ext_id = if let Some(script_path) = script {
        if instance_path.is_some() {
            // Instanced nodes inherit their script — skip
            None
        } else {
            let ext_id = generate_ext_resource_id(ext_resource_count);
            let ext_line = format!(
                "\n[ext_resource type=\"Script\" path=\"{}\" id=\"{}\"]\n",
                script_path, ext_id
            );

            let insert_pos = find_ext_resource_insert_pos(&new_content);
            new_content.insert_str(insert_pos, &ext_line);
            new_content = update_load_steps(&new_content);

            Some(ext_id)
        }
    } else {
        None
    };

    // Build the node section
    let mut node_section = format!("\n[node name=\"{}\"", node_name);

    if let Some(nt) = node_type {
        node_section.push_str(&format!(" type=\"{}\"", nt));
    }

    node_section.push_str(&format!(" parent=\"{}\"", parent_ref));

    if let Some(ref ext_id) = instance_ext_id {
        node_section.push_str(&format!(" instance=ExtResource(\"{}\")]\n", ext_id));
    } else {
        node_section.push_str("]\n");
    }

    if let Some(ext_id) = &script_ext_id {
        node_section.push_str(&format!("script = ExtResource(\"{}\")\n", ext_id));
    }

    for (key, value) in props {
        node_section.push_str(&format!("{} = {}\n", key, value));
    }

    // Insert before connections section, or at end
    let insert_pos = find_node_insert_pos(&new_content);
    new_content.insert_str(insert_pos, &node_section);

    atomic_write(path, &new_content)
}

/// Remove a node and its children from a .tscn file.
pub fn remove_node_from_file(path: &Path, node_name: &str) -> anyhow::Result<Vec<String>> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let scene = parse_scene_text(&content)?;

    // Verify node exists and is not the root
    let target = scene
        .nodes
        .iter()
        .find(|n| n.name == node_name)
        .ok_or_else(|| anyhow::anyhow!("Node '{}' not found in scene", node_name))?;

    if target.parent.is_none() {
        anyhow::bail!("Cannot remove root node '{}'", node_name);
    }

    // Find all nodes to remove (target + descendants)
    let target_path = parent_path_for(&scene, node_name).unwrap_or_else(|| node_name.to_string());
    let mut names_to_remove = vec![node_name.to_string()];

    // Find children recursively by checking parent paths
    for node in &scene.nodes {
        if let Some(ref p) = node.parent {
            if p == &target_path || p.starts_with(&format!("{}/", target_path)) {
                names_to_remove.push(node.name.clone());
            }
        }
    }

    // Collect ext_resource IDs used only by removed nodes
    let mut ext_ids_used_by_removed: Vec<String> = Vec::new();
    let mut ext_ids_used_by_kept: Vec<String> = Vec::new();

    for node in &scene.nodes {
        let is_removed = names_to_remove.contains(&node.name);
        for prop in &node.properties {
            if let Some(ext_id) = extract_ext_resource_ref(&prop.value) {
                if is_removed {
                    ext_ids_used_by_removed.push(ext_id);
                } else {
                    ext_ids_used_by_kept.push(ext_id);
                }
            }
        }
    }

    let orphaned_ext_ids: Vec<&String> = ext_ids_used_by_removed
        .iter()
        .filter(|id| !ext_ids_used_by_kept.contains(id))
        .collect();

    // Remove sections from raw text
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<&str> = Vec::new();
    let mut skip = false;
    let mut skip_blank_after = false;

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Check if this is a node section to remove
        if trimmed.starts_with("[node ") {
            let name = extract_quoted_attr(trimmed, "name").unwrap_or_default();
            if names_to_remove.contains(&name) {
                skip = true;
                skip_blank_after = true;
                i += 1;
                continue;
            }
        }

        // Check if this is an ext_resource to remove
        if trimmed.starts_with("[ext_resource ") {
            let id = extract_quoted_attr(trimmed, "id").unwrap_or_default();
            if orphaned_ext_ids.iter().any(|eid| **eid == id) {
                skip = true;
                skip_blank_after = true;
                i += 1;
                continue;
            }
        }

        // New section starts — stop skipping
        if trimmed.starts_with('[') && skip {
            skip = false;
            skip_blank_after = false;
        }

        if skip {
            i += 1;
            continue;
        }

        // Skip blank lines that followed removed sections
        if skip_blank_after && trimmed.is_empty() {
            skip_blank_after = false;
            i += 1;
            continue;
        }
        skip_blank_after = false;

        result_lines.push(lines[i]);
        i += 1;
    }

    let new_content = result_lines.join("\n");
    // Ensure trailing newline
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        new_content + "\n"
    };

    let new_content = update_load_steps(&new_content);
    atomic_write(path, &new_content)?;

    Ok(names_to_remove)
}

/// Edit a node property in a .tscn file.
/// `node_name` is the node to edit, `property` is the key, `value` is the new value.
pub fn edit_node_property(
    path: &Path,
    node_name: &str,
    property: &str,
    value: &str,
) -> anyhow::Result<()> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let scene = parse_scene_text(&content)?;

    // Verify node exists
    if !scene.nodes.iter().any(|n| n.name == node_name) {
        anyhow::bail!("Node '{}' not found in scene", node_name);
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    let mut found_and_set = false;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("[node ") {
            let name = extract_quoted_attr(trimmed, "name").unwrap_or_default();
            result.push(lines[i].to_string());
            i += 1;

            if name == node_name {
                // We're inside the target node — look for existing property
                let mut property_set = false;
                while i < lines.len() {
                    let prop_trimmed = lines[i].trim();
                    if prop_trimmed.is_empty() || prop_trimmed.starts_with('[') {
                        break;
                    }
                    if prop_trimmed.starts_with(&format!("{} = ", property)) {
                        // Replace existing property
                        result.push(format!("{} = {}", property, value));
                        property_set = true;
                        found_and_set = true;
                    } else {
                        result.push(lines[i].to_string());
                    }
                    i += 1;
                }
                if !property_set {
                    // Add new property before the blank line / next section
                    result.push(format!("{} = {}", property, value));
                    found_and_set = true;
                }
                continue;
            }
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    if !found_and_set {
        anyhow::bail!(
            "Failed to set property '{}' on node '{}'",
            property,
            node_name
        );
    }

    let new_content = result.join("\n");
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        new_content + "\n"
    };

    atomic_write(path, &new_content)
}

/// Extract an ExtResource ID from a value like `ExtResource("1_abc")`.
fn extract_ext_resource_ref(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.starts_with("ExtResource(\"") && trimmed.ends_with("\")") {
        Some(trimmed[13..trimmed.len() - 2].to_string())
    } else {
        None
    }
}

/// Find insertion position for new ext_resources (after last ext_resource or header).
fn find_ext_resource_insert_pos(content: &str) -> usize {
    let mut last_ext_end = None;
    let mut header_end = None;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("[gd_scene ") {
            header_end = Some(i);
        }
        if trimmed.starts_with("[ext_resource ") {
            last_ext_end = Some(i);
        }
    }

    let target_line = last_ext_end.or(header_end).unwrap_or(0);

    // Find byte position at end of target line
    let mut byte_pos = 0;
    for (i, line) in content.lines().enumerate() {
        byte_pos += line.len() + 1; // +1 for newline
        if i == target_line {
            return byte_pos;
        }
    }
    content.len()
}

/// Find insertion position for new nodes (before connections or at end).
fn find_node_insert_pos(content: &str) -> usize {
    // Insert before first [connection] if present, otherwise at end
    if let Some(pos) = content.find("\n[connection ") {
        return pos;
    }
    content.len()
}

/// Update the load_steps count in the header based on actual ext_resource + sub_resource count.
fn update_load_steps(content: &str) -> String {
    let mut ext_count = 0;
    let mut sub_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[ext_resource ") {
            ext_count += 1;
        }
        if trimmed.starts_with("[sub_resource ") {
            sub_count += 1;
        }
    }

    let total = ext_count + sub_count;

    // Find and replace/add load_steps in header
    let mut result = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[gd_scene ") {
            // Rebuild header
            let format_val = extract_attr(trimmed, "format").unwrap_or_else(|| "3".into());
            let uid_val = extract_quoted_attr(trimmed, "uid");

            if total > 0 {
                result.push_str(&format!(
                    "[gd_scene load_steps={} format={}",
                    total + 1,
                    format_val
                ));
            } else {
                result.push_str(&format!("[gd_scene format={}", format_val));
            }
            if let Some(uid) = uid_val {
                result.push_str(&format!(" uid=\"{}\"", uid));
            }
            result.push_str("]\n");
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Remove potential trailing extra newline from the loop
    if result.ends_with("\n\n") && !content.ends_with("\n\n") {
        result.pop();
    }

    result
}

/// Write content to a file atomically (write to temp, then rename).
pub fn atomic_write(path: &Path, content: &str) -> anyhow::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let temp_path = dir.join(format!(".gdcli_tmp_{}", std::process::id()));

    fs::write(&temp_path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write temp file {}: {}", temp_path.display(), e))?;

    fs::rename(&temp_path, path).map_err(|e| {
        // Clean up temp file on rename failure
        let _ = fs::remove_file(&temp_path);
        anyhow::anyhow!(
            "Failed to rename {} to {}: {}",
            temp_path.display(),
            path.display(),
            e
        )
    })?;

    Ok(())
}

/// Parse simple property values for the --props flag.
/// Handles: booleans, integers, floats, strings, and Godot types passed through verbatim.
pub fn format_prop_value(value: &str) -> String {
    // Already quoted — return as-is to avoid double-quoting
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return value.to_string();
    }

    // Boolean
    if value == "true" || value == "false" {
        return value.to_string();
    }

    // Integer
    if value.parse::<i64>().is_ok() {
        return value.to_string();
    }

    // Float
    if value.parse::<f64>().is_ok() {
        return value.to_string();
    }

    // Godot types: Vector2(...), Vector3(...), Color(...), etc. — pass through
    if value.contains('(') && value.contains(')') {
        return value.to_string();
    }

    // Arrays: [...] — pass through unquoted
    if value.starts_with('[') && value.ends_with(']') {
        return value.to_string();
    }

    // Dictionaries: {...} — pass through unquoted
    if value.starts_with('{') && value.ends_with('}') {
        return value.to_string();
    }

    // res:// paths
    if value.starts_with("res://") {
        return format!("\"{}\"", value);
    }

    // Plain string — quote it
    format!("\"{}\"", value)
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

/// Extract the `instance=ExtResource("...")` attribute from a node line.
/// Returns `Some("ExtResource(\"id\")")` if present.
fn extract_instance_attr(line: &str) -> Option<String> {
    let marker = "instance=ExtResource(\"";
    let start = line.find(marker)?;
    let val_start = start + "instance=".len();
    let rest = &line[val_start..];
    // Find the closing `)` after `ExtResource("...")`
    let end = rest.find(')')?;
    Some(rest[..=end].to_string())
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

/// Infer the Godot resource type from a file extension.
/// e.g. `.gd` → `"Script"`, `.tscn` → `"PackedScene"`, `.tres` → `"Resource"`.
pub fn infer_resource_type(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "gd" => "Script",
        "tscn" => "PackedScene",
        "tres" => "Resource",
        "png" | "jpg" | "jpeg" | "webp" | "svg" | "bmp" => "Texture2D",
        "ogg" | "wav" | "mp3" => "AudioStream",
        "ttf" | "otf" | "woff" | "woff2" => "FontFile",
        "gdshader" | "shader" => "Shader",
        _ => "Resource",
    }
}

/// Add an ext_resource to a scene file if not already present.
/// Returns the ext_resource ID (existing or newly created).
pub fn add_ext_resource_to_file(
    path: &Path,
    res_path: &str,
    res_type: &str,
) -> anyhow::Result<String> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let scene = parse_scene_text(&content)?;

    // Check if this resource is already referenced
    for ext in &scene.ext_resources {
        if ext.path == res_path {
            return Ok(ext.id.clone());
        }
    }

    // Add a new ext_resource
    let ext_id = generate_ext_resource_id(scene.ext_resources.len());
    let ext_line = format!(
        "\n[ext_resource type=\"{}\" path=\"{}\" id=\"{}\"]\n",
        res_type, res_path, ext_id
    );

    let mut new_content = content;
    let insert_pos = find_ext_resource_insert_pos(&new_content);
    new_content.insert_str(insert_pos, &ext_line);
    new_content = update_load_steps(&new_content);

    atomic_write(path, &new_content)?;

    Ok(ext_id)
}

/// Add a signal connection to a .tscn file.
/// Validates that `from` and `to` nodes exist (or are "."), and checks for duplicates.
/// Appends `[connection signal="..." from="..." to="..." method="..."]` at end of file.
pub fn add_connection_to_file(
    path: &Path,
    signal: &str,
    from: &str,
    to: &str,
    method: &str,
) -> anyhow::Result<()> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let scene = parse_scene_text(&content)?;

    // Validate from node exists
    if from != "." {
        let root_name = scene.nodes.first().map(|n| n.name.as_str()).unwrap_or("");
        if !scene.nodes.iter().any(|n| n.name == from) && from != root_name {
            anyhow::bail!("Node '{}' (from) not found in scene", from);
        }
    }

    // Validate to node exists
    if to != "." {
        let root_name = scene.nodes.first().map(|n| n.name.as_str()).unwrap_or("");
        if !scene.nodes.iter().any(|n| n.name == to) && to != root_name {
            anyhow::bail!("Node '{}' (to) not found in scene", to);
        }
    }

    // Check for duplicate connection
    for conn in &scene.connections {
        if conn.signal == signal && conn.from == from && conn.to == to && conn.method == method {
            anyhow::bail!(
                "Duplicate connection: {}.{} -> {}.{} already exists",
                from,
                signal,
                to,
                method
            );
        }
    }

    // Append connection at end of file
    let connection_line = format!(
        "\n[connection signal=\"{}\" from=\"{}\" to=\"{}\" method=\"{}\"]\n",
        signal, from, to, method
    );

    let mut new_content = content;
    // Ensure file ends with newline before appending
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&connection_line);

    atomic_write(path, &new_content)
}

/// Remove a signal connection from a .tscn file.
/// Finds the matching `[connection ...]` line and removes it (plus trailing blank line).
pub fn remove_connection_from_file(
    path: &Path,
    signal: &str,
    from: &str,
    to: &str,
    method: &str,
) -> anyhow::Result<()> {
    require_scene_file(path)?;
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let scene = parse_scene_text(&content)?;

    // Verify connection exists
    let found = scene
        .connections
        .iter()
        .any(|c| c.signal == signal && c.from == from && c.to == to && c.method == method);
    if !found {
        anyhow::bail!(
            "Connection not found: {}.{} -> {}.{}",
            from,
            signal,
            to,
            method
        );
    }

    // Build target connection line for matching
    let target = format!(
        "[connection signal=\"{}\" from=\"{}\" to=\"{}\" method=\"{}\"]",
        signal, from, to, method
    );

    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == target {
            // Skip this line
            i += 1;
            // Skip trailing blank line if present
            if i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
            continue;
        }
        result_lines.push(lines[i]);
        i += 1;
    }

    let new_content = result_lines.join("\n");
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        new_content + "\n"
    };

    atomic_write(path, &new_content)
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
    fn test_parse_node_properties() {
        let content = r#"[gd_scene format=3]

[node name="Root" type="Node2D"]
position = Vector2(100, 200)
visible = false
"#;
        let scene = parse_scene_text(content).unwrap();
        assert_eq!(scene.nodes[0].properties.len(), 2);
        assert_eq!(scene.nodes[0].properties[0].key, "position");
        assert_eq!(scene.nodes[0].properties[0].value, "Vector2(100, 200)");
        assert_eq!(scene.nodes[0].properties[1].key, "visible");
        assert_eq!(scene.nodes[0].properties[1].value, "false");
    }

    #[test]
    fn test_extract_quoted_attr() {
        let line = r#"[node name="Main" type="Node2D" parent="."]"#;
        assert_eq!(extract_quoted_attr(line, "name").as_deref(), Some("Main"));
        assert_eq!(extract_quoted_attr(line, "type").as_deref(), Some("Node2D"));
        assert_eq!(extract_quoted_attr(line, "parent").as_deref(), Some("."));
        assert_eq!(extract_quoted_attr(line, "missing"), None);
    }

    #[test]
    fn test_generate_minimal_scene() {
        let scene = generate_minimal_scene("CharacterBody2D", "Enemy", "uid://test123", None);
        assert!(scene.contains("[gd_scene format=3 uid=\"uid://test123\"]"));
        assert!(scene.contains("[node name=\"Enemy\" type=\"CharacterBody2D\"]"));
    }

    #[test]
    fn test_generate_minimal_scene_with_script() {
        let scene = generate_minimal_scene(
            "Node2D",
            "Main",
            "uid://test456",
            Some("res://scripts/test.gd"),
        );
        assert!(scene.contains("[gd_scene load_steps=2 format=3 uid=\"uid://test456\"]"));
        assert!(scene.contains("[ext_resource type=\"Script\" path=\"res://scripts/test.gd\""));
        assert!(scene.contains("[node name=\"Main\" type=\"Node2D\"]"));
        assert!(scene.contains("script = ExtResource(\""));
    }

    #[test]
    fn test_filename_to_node_name() {
        assert_eq!(filename_to_node_name("enemy.tscn"), "Enemy");
        assert_eq!(filename_to_node_name("game_over.tscn"), "GameOver");
        assert_eq!(filename_to_node_name("my-level.tscn"), "MyLevel");
        assert_eq!(filename_to_node_name("scenes/main.tscn"), "Main");
        assert_eq!(filename_to_node_name("a_b_c.tscn"), "ABC");
        assert_eq!(filename_to_node_name("Player.tscn"), "Player");
    }

    #[test]
    fn test_write_scene_roundtrip() {
        let content = r#"[gd_scene format=3 uid="uid://abc"]

[node name="Root" type="Node2D"]

[node name="Child" type="Sprite2D" parent="."]
"#;
        let scene = parse_scene_text(content).unwrap();
        let written = write_scene(&scene);
        assert!(written.contains("[gd_scene format=3 uid=\"uid://abc\"]"));
        assert!(written.contains("[node name=\"Root\" type=\"Node2D\"]"));
        assert!(written.contains("[node name=\"Child\" type=\"Sprite2D\" parent=\".\"]"));
    }

    #[test]
    fn test_format_prop_value() {
        assert_eq!(format_prop_value("true"), "true");
        assert_eq!(format_prop_value("42"), "42");
        assert_eq!(format_prop_value("3.14"), "3.14");
        assert_eq!(format_prop_value("Vector2(1, 2)"), "Vector2(1, 2)");
        assert_eq!(format_prop_value("res://foo.gd"), "\"res://foo.gd\"");
        assert_eq!(format_prop_value("hello"), "\"hello\"");
        // Arrays pass through unquoted
        assert_eq!(format_prop_value("[\"enemies\"]"), "[\"enemies\"]");
        assert_eq!(format_prop_value("[1, 2, 3]"), "[1, 2, 3]");
        assert_eq!(format_prop_value("[]"), "[]");
        // Dictionaries pass through unquoted
        assert_eq!(
            format_prop_value("{\"key\": \"val\"}"),
            "{\"key\": \"val\"}"
        );
        assert_eq!(format_prop_value("{}"), "{}");
        // Already-quoted strings — should NOT double-quote
        assert_eq!(format_prop_value("\"Score: 0\""), "\"Score: 0\"");
        assert_eq!(format_prop_value("\"res://foo.gd\""), "\"res://foo.gd\"");
        assert_eq!(format_prop_value("\"hello world\""), "\"hello world\"");
        assert_eq!(format_prop_value("\"\""), "\"\"");
    }

    #[test]
    fn test_generate_uid() {
        let uid = generate_uid();
        assert!(uid.starts_with("uid://"));
        assert!(uid.len() > 10);
    }

    #[test]
    fn test_extract_ext_resource_ref() {
        assert_eq!(
            extract_ext_resource_ref("ExtResource(\"1_abc\")"),
            Some("1_abc".to_string())
        );
        assert_eq!(extract_ext_resource_ref("42"), None);
    }

    #[test]
    fn test_parse_instance_node() {
        let content = r#"[gd_scene load_steps=2 format=3 uid="uid://abc"]

[ext_resource type="PackedScene" path="res://coin.tscn" id="2_player"]

[node name="Main" type="Node2D"]

[node name="Coin1" parent="." instance=ExtResource("2_player")]
"#;
        let scene = parse_scene_text(content).unwrap();
        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.nodes[1].name, "Coin1");
        assert!(scene.nodes[1].node_type.is_empty());
        assert_eq!(
            scene.nodes[1].instance.as_deref(),
            Some("ExtResource(\"2_player\")")
        );
        // Root node should have no instance
        assert_eq!(scene.nodes[0].instance, None);
    }

    #[test]
    fn test_parent_path_for() {
        let content = r#"[gd_scene format=3]

[node name="Root" type="Node2D"]

[node name="Player" type="CharacterBody2D" parent="."]

[node name="Sprite" type="Sprite2D" parent="Player"]
"#;
        let scene = parse_scene_text(content).unwrap();
        assert_eq!(parent_path_for(&scene, "Root"), Some(".".to_string()));
        assert_eq!(
            parent_path_for(&scene, "Player"),
            Some("Player".to_string())
        );
        assert_eq!(
            parent_path_for(&scene, "Sprite"),
            Some("Player/Sprite".to_string())
        );
    }

    #[test]
    fn test_add_instance_node() {
        // Create a temp scene file and add an instanced node to it
        let dir = std::env::temp_dir().join("gdcli_test_instance");
        let _ = std::fs::create_dir_all(&dir);
        let scene_path = dir.join("test_instance.tscn");

        let content =
            "[gd_scene format=3 uid=\"uid://abc\"]\n\n[node name=\"Main\" type=\"Node2D\"]\n";
        std::fs::write(&scene_path, content).unwrap();

        add_node_to_file(
            &scene_path,
            None,
            "Enemy1",
            None,
            None,
            &[],
            Some("res://scenes/enemy.tscn"),
        )
        .unwrap();

        let result = std::fs::read_to_string(&scene_path).unwrap();
        let parsed = parse_scene_text(&result).unwrap();

        // Should have a PackedScene ext_resource
        assert_eq!(parsed.ext_resources.len(), 1);
        assert_eq!(parsed.ext_resources[0].resource_type, "PackedScene");
        assert_eq!(parsed.ext_resources[0].path, "res://scenes/enemy.tscn");

        // Should have the instanced node with no type
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.nodes[1].name, "Enemy1");
        assert!(parsed.nodes[1].node_type.is_empty());
        assert!(parsed.nodes[1].instance.is_some());

        // Clean up
        let _ = std::fs::remove_file(&scene_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_parse_sub_resource_properties() {
        let content = r#"[gd_scene load_steps=3 format=3 uid="uid://abc"]

[ext_resource type="Script" path="res://player.gd" id="1_abc"]

[sub_resource type="RectangleShape2D" id="RectangleShape2D_abc"]
size = Vector2(40, 40)

[sub_resource type="CircleShape2D" id="CircleShape2D_xyz"]
radius = 20.0

[node name="Player" type="CharacterBody2D"]
script = ExtResource("1_abc")

[node name="CollisionShape" type="CollisionShape2D" parent="."]
shape = SubResource("RectangleShape2D_abc")
"#;
        let scene = parse_scene_text(content).unwrap();
        assert_eq!(scene.sub_resources.len(), 2);

        assert_eq!(scene.sub_resources[0].resource_type, "RectangleShape2D");
        assert_eq!(scene.sub_resources[0].properties.len(), 1);
        assert_eq!(scene.sub_resources[0].properties[0].key, "size");
        assert_eq!(
            scene.sub_resources[0].properties[0].value,
            "Vector2(40, 40)"
        );

        assert_eq!(scene.sub_resources[1].resource_type, "CircleShape2D");
        assert_eq!(scene.sub_resources[1].properties.len(), 1);
        assert_eq!(scene.sub_resources[1].properties[0].key, "radius");
        assert_eq!(scene.sub_resources[1].properties[0].value, "20.0");
    }

    #[test]
    fn test_write_scene_with_sub_resources() {
        let content = r#"[gd_scene load_steps=2 format=3 uid="uid://abc"]

[sub_resource type="RectangleShape2D" id="Shape_abc"]
size = Vector2(30, 30)

[node name="Root" type="Node2D"]
"#;
        let scene = parse_scene_text(content).unwrap();
        let written = write_scene(&scene);
        assert!(written.contains("[sub_resource type=\"RectangleShape2D\" id=\"Shape_abc\"]"));
        assert!(written.contains("size = Vector2(30, 30)"));
    }

    #[test]
    fn test_add_connection_to_file() {
        let dir = std::env::temp_dir().join("gdcli_test_conn_add");
        let _ = std::fs::create_dir_all(&dir);
        let scene_path = dir.join("test_conn.tscn");

        let content = "[gd_scene format=3 uid=\"uid://abc\"]\n\n\
            [node name=\"Main\" type=\"Node2D\"]\n\n\
            [node name=\"Button1\" type=\"Button\" parent=\".\"]\n";
        std::fs::write(&scene_path, content).unwrap();

        add_connection_to_file(&scene_path, "pressed", "Button1", ".", "_on_pressed").unwrap();

        let result = std::fs::read_to_string(&scene_path).unwrap();
        assert!(result.contains(
            "[connection signal=\"pressed\" from=\"Button1\" to=\".\" method=\"_on_pressed\"]"
        ));

        // Duplicate should fail
        let dup = add_connection_to_file(&scene_path, "pressed", "Button1", ".", "_on_pressed");
        assert!(dup.is_err());
        assert!(dup.unwrap_err().to_string().contains("Duplicate"));

        // Non-existent node should fail
        let bad = add_connection_to_file(&scene_path, "pressed", "NoNode", ".", "_on_pressed");
        assert!(bad.is_err());
        assert!(bad.unwrap_err().to_string().contains("not found"));

        let _ = std::fs::remove_file(&scene_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_remove_connection_from_file() {
        let dir = std::env::temp_dir().join("gdcli_test_conn_rm");
        let _ = std::fs::create_dir_all(&dir);
        let scene_path = dir.join("test_conn_rm.tscn");

        let content = "[gd_scene format=3 uid=\"uid://abc\"]\n\n\
            [node name=\"Main\" type=\"Node2D\"]\n\n\
            [node name=\"Btn\" type=\"Button\" parent=\".\"]\n\n\
            [connection signal=\"pressed\" from=\"Btn\" to=\".\" method=\"_on_btn\"]\n";
        std::fs::write(&scene_path, content).unwrap();

        remove_connection_from_file(&scene_path, "pressed", "Btn", ".", "_on_btn").unwrap();

        let result = std::fs::read_to_string(&scene_path).unwrap();
        assert!(!result.contains("[connection"));

        // Removing again should fail
        let bad = remove_connection_from_file(&scene_path, "pressed", "Btn", ".", "_on_btn");
        assert!(bad.is_err());
        assert!(bad.unwrap_err().to_string().contains("not found"));

        let _ = std::fs::remove_file(&scene_path);
        let _ = std::fs::remove_dir(&dir);
    }
}
