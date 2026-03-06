use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ClassDoc {
    pub name: String,
    pub inherits: Option<String>,
    pub brief_description: String,
    pub description: String,
    pub methods: Vec<MethodDoc>,
    pub properties: Vec<PropertyDoc>,
    pub signals: Vec<SignalDoc>,
    pub constants: Vec<ConstantDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodDoc {
    pub name: String,
    pub return_type: String,
    pub description: String,
    pub params: Vec<ParamDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParamDoc {
    pub name: String,
    pub param_type: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PropertyDoc {
    pub name: String,
    pub property_type: String,
    pub default: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalDoc {
    pub name: String,
    pub description: String,
    pub params: Vec<ParamDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstantDoc {
    pub name: String,
    pub value: String,
    pub description: String,
}

/// Find the docs cache directory. Uses `.gdcli/docs/` in the project root if
/// `project.godot` exists, otherwise `~/.gdcli/docs/`.
pub fn docs_dir() -> PathBuf {
    if Path::new("project.godot").is_file() {
        PathBuf::from(".gdcli/docs")
    } else if let Some(home) = home_dir() {
        home.join(".gdcli").join("docs")
    } else {
        PathBuf::from(".gdcli/docs")
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Check if docs have been built (XML files exist in the docs dir).
pub fn docs_exist() -> bool {
    let dir = docs_dir();
    if !dir.is_dir() {
        return false;
    }
    // Check for at least one XML file
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "xml") {
                return true;
            }
        }
    }
    false
}

/// Parse a single class XML file from Godot's doctool output.
pub fn parse_class_xml(path: &Path) -> anyhow::Result<ClassDoc> {
    let content = fs::read_to_string(path)?;
    parse_class_xml_text(&content)
}

/// Parse class XML text content.
pub fn parse_class_xml_text(content: &str) -> anyhow::Result<ClassDoc> {
    let mut reader = Reader::from_str(content);

    let mut doc = ClassDoc {
        name: String::new(),
        inherits: None,
        brief_description: String::new(),
        description: String::new(),
        methods: Vec::new(),
        properties: Vec::new(),
        signals: Vec::new(),
        constants: Vec::new(),
    };

    let mut current_section = Section::None;
    let mut current_method: Option<MethodDoc> = None;
    let mut current_signal: Option<SignalDoc> = None;
    let mut text_target = TextTarget::None;
    let mut text_buf = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag.as_str() {
                    "class" => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            match key.as_str() {
                                "name" => doc.name = val,
                                "inherits" => doc.inherits = Some(val),
                                _ => {}
                            }
                        }
                    }
                    "brief_description" => {
                        text_target = TextTarget::BriefDescription;
                        text_buf.clear();
                    }
                    "description" if current_section == Section::None => {
                        text_target = TextTarget::ClassDescription;
                        text_buf.clear();
                    }
                    "description" => {
                        text_target = TextTarget::ItemDescription;
                        text_buf.clear();
                    }
                    "methods" => current_section = Section::Methods,
                    "members" => current_section = Section::Members,
                    "signals" => current_section = Section::Signals,
                    "constants" => current_section = Section::Constants,
                    "method" | "constructor" | "operator" => {
                        if current_section == Section::Methods
                            || tag == "constructor"
                            || tag == "operator"
                        {
                            let name = get_attr(e, "name").unwrap_or_default();
                            current_method = Some(MethodDoc {
                                name,
                                return_type: String::new(),
                                description: String::new(),
                                params: Vec::new(),
                            });
                        }
                    }
                    "return" => {
                        if let Some(ref mut method) = current_method {
                            method.return_type =
                                get_attr(e, "type").unwrap_or_else(|| "void".into());
                        }
                    }
                    "param" => {
                        let name = get_attr(e, "name").unwrap_or_default();
                        let param_type = get_attr(e, "type").unwrap_or_default();
                        let default = get_attr(e, "default");

                        let param = ParamDoc {
                            name,
                            param_type,
                            default,
                        };

                        if let Some(ref mut method) = current_method {
                            method.params.push(param);
                        } else if let Some(ref mut signal) = current_signal {
                            signal.params.push(param);
                        }
                    }
                    "member" if current_section == Section::Members => {
                        let name = get_attr(e, "name").unwrap_or_default();
                        let prop_type = get_attr(e, "type").unwrap_or_default();
                        let default = get_attr(e, "default");

                        doc.properties.push(PropertyDoc {
                            name,
                            property_type: prop_type,
                            default,
                            description: String::new(),
                        });
                        text_target = TextTarget::PropertyDescription;
                        text_buf.clear();
                    }
                    "signal" if current_section == Section::Signals => {
                        let name = get_attr(e, "name").unwrap_or_default();
                        current_signal = Some(SignalDoc {
                            name,
                            description: String::new(),
                            params: Vec::new(),
                        });
                    }
                    "constant" if current_section == Section::Constants => {
                        let name = get_attr(e, "name").unwrap_or_default();
                        let value = get_attr(e, "value").unwrap_or_default();
                        doc.constants.push(ConstantDoc {
                            name,
                            value,
                            description: String::new(),
                        });
                        text_target = TextTarget::ConstantDescription;
                        text_buf.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if text_target != TextTarget::None {
                    if let Ok(text) = e.unescape() {
                        text_buf.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "brief_description" => {
                        doc.brief_description = clean_description(&text_buf);
                        text_target = TextTarget::None;
                    }
                    "description" if text_target == TextTarget::ClassDescription => {
                        doc.description = clean_description(&text_buf);
                        text_target = TextTarget::None;
                    }
                    "description" if text_target == TextTarget::ItemDescription => {
                        if let Some(ref mut method) = current_method {
                            method.description = clean_description(&text_buf);
                        }
                        if let Some(ref mut signal) = current_signal {
                            signal.description = clean_description(&text_buf);
                        }
                        text_target = TextTarget::None;
                    }
                    "method" | "constructor" | "operator" => {
                        if let Some(method) = current_method.take() {
                            doc.methods.push(method);
                        }
                    }
                    "member" if text_target == TextTarget::PropertyDescription => {
                        if let Some(prop) = doc.properties.last_mut() {
                            prop.description = clean_description(&text_buf);
                        }
                        text_target = TextTarget::None;
                    }
                    "signal" => {
                        if let Some(signal) = current_signal.take() {
                            doc.signals.push(signal);
                        }
                    }
                    "constant" if text_target == TextTarget::ConstantDescription => {
                        if let Some(constant) = doc.constants.last_mut() {
                            constant.description = clean_description(&text_buf);
                        }
                        text_target = TextTarget::None;
                    }
                    "methods" | "members" | "signals" | "constants" => {
                        current_section = Section::None;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                anyhow::bail!("XML parse error: {}", e);
            }
            _ => {}
        }
    }

    Ok(doc)
}

/// Find the XML file for a given class name.
pub fn find_class_xml(class_name: &str) -> Option<PathBuf> {
    let dir = docs_dir();
    let path = dir.join(format!("{}.xml", class_name));
    if path.is_file() {
        Some(path)
    } else {
        // Try case-insensitive search
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.eq_ignore_ascii_case(&format!("{}.xml", class_name)) {
                    return Some(entry.path());
                }
            }
        }
        None
    }
}

/// Format a method signature for display.
pub fn format_method_sig(method: &MethodDoc) -> String {
    let params: Vec<String> = method
        .params
        .iter()
        .map(|p| {
            if let Some(ref def) = p.default {
                format!("{}: {} = {}", p.name, p.param_type, def)
            } else {
                format!("{}: {}", p.name, p.param_type)
            }
        })
        .collect();

    format!(
        "{}({}) -> {}",
        method.name,
        params.join(", "),
        if method.return_type.is_empty() {
            "void"
        } else {
            &method.return_type
        }
    )
}

fn get_attr(e: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == name.as_bytes() {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

/// Clean up Godot XML documentation text (strip BBCode-like tags).
fn clean_description(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Strip common BBCode-like tags: [b], [i], [code], [url], [member], [method], etc.
    let mut result = trimmed.to_string();
    let tags = [
        "[b]",
        "[/b]",
        "[i]",
        "[/i]",
        "[code]",
        "[/code]",
        "[codeblock]",
        "[/codeblock]",
        "[codeblocks]",
        "[/codeblocks]",
        "[gdscript]",
        "[/gdscript]",
        "[br]",
    ];
    for tag in &tags {
        result = result.replace(tag, "");
    }

    // Strip tags like [member name], [method name], etc. — keep the inner name
    let ref_tags = [
        "[member ",
        "[method ",
        "[signal ",
        "[enum ",
        "[constant ",
        "[param ",
    ];
    for pat in &ref_tags {
        while let Some(start) = result.find(pat) {
            if let Some(end) = result[start..].find(']') {
                let inner = &result[start + pat.len()..start + end];
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    inner,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }
    }

    // Strip [url=...]...[/url] — remove the tag markers but keep content between
    while let Some(start) = result.find("[url=") {
        if let Some(end) = result[start..].find(']') {
            result = format!("{}{}", &result[..start], &result[start + end + 1..]);
        } else {
            break;
        }
    }
    // Remove closing [/url] etc.
    result = result.replace("[/url]", "");

    result.trim().to_string()
}

#[derive(Debug, Clone, PartialEq)]
enum Section {
    None,
    Methods,
    Members,
    Signals,
    Constants,
}

#[derive(Debug, Clone, PartialEq)]
enum TextTarget {
    None,
    BriefDescription,
    ClassDescription,
    ItemDescription,
    PropertyDescription,
    ConstantDescription,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_class() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<class name="Node2D" inherits="CanvasItem" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <brief_description>
        A 2D game object.
    </brief_description>
    <description>
        A 2D game object, with position and rotation.
    </description>
    <methods>
        <method name="apply_scale">
            <return type="void" />
            <param index="0" name="ratio" type="Vector2" />
            <description>
                Multiplies the current scale by the ratio vector.
            </description>
        </method>
    </methods>
    <members>
        <member name="position" type="Vector2" default="Vector2(0, 0)">
            Position relative to the parent.
        </member>
    </members>
    <signals>
        <signal name="visibility_changed">
            <description>
                Emitted when visibility changes.
            </description>
        </signal>
    </signals>
    <constants>
    </constants>
</class>"#;
        let doc = parse_class_xml_text(xml).unwrap();
        assert_eq!(doc.name, "Node2D");
        assert_eq!(doc.inherits.as_deref(), Some("CanvasItem"));
        assert!(!doc.brief_description.is_empty());
        assert_eq!(doc.methods.len(), 1);
        assert_eq!(doc.methods[0].name, "apply_scale");
        assert_eq!(doc.methods[0].params.len(), 1);
        assert_eq!(doc.properties.len(), 1);
        assert_eq!(doc.properties[0].name, "position");
        assert_eq!(doc.signals.len(), 1);
    }

    #[test]
    fn test_format_method_sig() {
        let method = MethodDoc {
            name: "move_and_slide".into(),
            return_type: "bool".into(),
            description: String::new(),
            params: vec![],
        };
        assert_eq!(format_method_sig(&method), "move_and_slide() -> bool");

        let method2 = MethodDoc {
            name: "apply_scale".into(),
            return_type: "void".into(),
            description: String::new(),
            params: vec![ParamDoc {
                name: "ratio".into(),
                param_type: "Vector2".into(),
                default: None,
            }],
        };
        assert_eq!(
            format_method_sig(&method2),
            "apply_scale(ratio: Vector2) -> void"
        );
    }

    #[test]
    fn test_clean_description() {
        assert_eq!(
            clean_description("[b]Bold[/b] and [code]code[/code]"),
            "Bold and code"
        );
        assert_eq!(
            clean_description("See [member position] for details."),
            "See position for details."
        );
    }
}
