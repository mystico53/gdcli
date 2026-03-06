use crate::docs_parser::{self, ClassDoc};
use crate::godot_finder;
use crate::output;
use crate::runner;
use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;

// --- docs ---

#[derive(Serialize)]
pub struct DocsReport {
    pub class: ClassDoc,
    pub member: Option<MemberLookup>,
}

#[derive(Serialize)]
pub struct MemberLookup {
    pub name: String,
    pub kind: String,
    pub signature: Option<String>,
    pub description: String,
}

#[derive(Serialize)]
pub struct DocsBuildReport {
    pub docs_dir: String,
    pub class_count: usize,
}

#[derive(Serialize)]
pub struct DocsMembersReport {
    pub class_name: String,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub signals: Vec<String>,
}

pub fn run_docs(
    class_name: &str,
    member: Option<&str>,
    list_members: bool,
    json_mode: bool,
) -> Result<bool> {
    if !docs_parser::docs_exist() {
        bail!(
            "Docs not built yet. Run `gdcli docs --build` first.\n\
             This will invoke `godot --doctool` to generate XML class references."
        );
    }

    let xml_path = docs_parser::find_class_xml(class_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Class '{}' not found in docs.\n\
             Make sure the docs are up to date: `gdcli docs --build`",
            class_name
        )
    })?;

    let doc = docs_parser::parse_class_xml(&xml_path)?;

    if list_members {
        return run_list_members(&doc, json_mode);
    }

    if let Some(member_name) = member {
        return run_member_lookup(&doc, member_name, json_mode);
    }

    // Class overview
    if json_mode {
        let report = DocsReport {
            class: doc,
            member: None,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "docs".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        print_class_overview(&doc);
    }

    Ok(true)
}

fn run_member_lookup(doc: &ClassDoc, member_name: &str, json_mode: bool) -> Result<bool> {
    // Search methods
    if let Some(method) = doc.methods.iter().find(|m| m.name == member_name) {
        let sig = docs_parser::format_method_sig(method);
        if json_mode {
            let report = DocsReport {
                class: doc.clone(),
                member: Some(MemberLookup {
                    name: method.name.clone(),
                    kind: "method".into(),
                    signature: Some(sig),
                    description: method.description.clone(),
                }),
            };
            let envelope = output::JsonEnvelope {
                ok: true,
                command: "docs".into(),
                data: Some(report),
                error: None,
            };
            output::emit_json(&envelope);
        } else {
            println!("  {} (method)", doc.name);
            println!("  {}", docs_parser::format_method_sig(method));
            if !method.description.is_empty() {
                println!();
                println!("  {}", method.description);
            }
        }
        return Ok(true);
    }

    // Search properties
    if let Some(prop) = doc.properties.iter().find(|p| p.name == member_name) {
        if json_mode {
            let report = DocsReport {
                class: doc.clone(),
                member: Some(MemberLookup {
                    name: prop.name.clone(),
                    kind: "property".into(),
                    signature: Some(format!("{}: {}", prop.name, prop.property_type)),
                    description: prop.description.clone(),
                }),
            };
            let envelope = output::JsonEnvelope {
                ok: true,
                command: "docs".into(),
                data: Some(report),
                error: None,
            };
            output::emit_json(&envelope);
        } else {
            println!("  {} (property)", doc.name);
            println!("  {}: {}", prop.name, prop.property_type);
            if let Some(ref def) = prop.default {
                println!("  default: {}", def);
            }
            if !prop.description.is_empty() {
                println!();
                println!("  {}", prop.description);
            }
        }
        return Ok(true);
    }

    // Search signals
    if let Some(signal) = doc.signals.iter().find(|s| s.name == member_name) {
        if json_mode {
            let report = DocsReport {
                class: doc.clone(),
                member: Some(MemberLookup {
                    name: signal.name.clone(),
                    kind: "signal".into(),
                    signature: None,
                    description: signal.description.clone(),
                }),
            };
            let envelope = output::JsonEnvelope {
                ok: true,
                command: "docs".into(),
                data: Some(report),
                error: None,
            };
            output::emit_json(&envelope);
        } else {
            println!("  {} (signal)", doc.name);
            println!("  signal {}", signal.name);
            if !signal.description.is_empty() {
                println!();
                println!("  {}", signal.description);
            }
        }
        return Ok(true);
    }

    bail!(
        "Member '{}' not found in class '{}'.\n\
         Use `gdcli docs {} --members` to list all members.",
        member_name,
        doc.name,
        doc.name
    );
}

fn run_list_members(doc: &ClassDoc, json_mode: bool) -> Result<bool> {
    if json_mode {
        let report = DocsMembersReport {
            class_name: doc.name.clone(),
            methods: doc
                .methods
                .iter()
                .map(docs_parser::format_method_sig)
                .collect(),
            properties: doc
                .properties
                .iter()
                .map(|p| format!("{}: {}", p.name, p.property_type))
                .collect(),
            signals: doc.signals.iter().map(|s| s.name.clone()).collect(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "docs".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        output::print_header(&format!("{} members:", doc.name));

        if !doc.properties.is_empty() {
            println!("\n  Properties:");
            for prop in &doc.properties {
                println!("    {}: {}", prop.name, prop.property_type);
            }
        }

        if !doc.methods.is_empty() {
            println!("\n  Methods:");
            for method in &doc.methods {
                println!("    {}", docs_parser::format_method_sig(method));
            }
        }

        if !doc.signals.is_empty() {
            println!("\n  Signals:");
            for signal in &doc.signals {
                println!("    {}", signal.name);
            }
        }
    }

    Ok(true)
}

pub fn run_build(json_mode: bool) -> Result<bool> {
    let docs_dir = docs_parser::docs_dir();

    // Create docs directory
    fs::create_dir_all(&docs_dir)?;

    // Find Godot
    let godot_info = godot_finder::find_and_probe()?;

    let docs_dir_str = docs_dir.display().to_string();

    // Run godot --doctool <output_dir>
    let result = runner::run_raw(
        &godot_info.path,
        &["--headless", "--doctool", &docs_dir_str],
        120,
    )?;

    if result.exit_code != 0 && result.exit_code != -1 {
        bail!(
            "godot --doctool failed (exit code {}):\n{}",
            result.exit_code,
            result.stderr
        );
    }

    // Count generated XML files
    let class_count = fs::read_dir(&docs_dir)?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "xml"))
        .count();

    if class_count == 0 {
        bail!(
            "No XML files generated in {}.\n\
             Check that your Godot build supports --doctool.",
            docs_dir_str
        );
    }

    if json_mode {
        let report = DocsBuildReport {
            docs_dir: docs_dir_str,
            class_count,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "docs build".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Built docs: {} classes in {}",
            class_count, docs_dir_str
        );
    }

    Ok(true)
}

fn print_class_overview(doc: &ClassDoc) {
    output::print_header(&doc.name);
    if let Some(ref inherits) = doc.inherits {
        println!("  extends {}", inherits);
    }
    println!();

    if !doc.brief_description.is_empty() {
        println!("  {}", doc.brief_description);
        println!();
    }

    if !doc.description.is_empty() {
        // Truncate long descriptions for TTY display
        let desc = if doc.description.len() > 500 {
            format!("{}...", &doc.description[..500])
        } else {
            doc.description.clone()
        };
        println!("  {}", desc);
        println!();
    }

    println!(
        "  {} methods, {} properties, {} signals",
        doc.methods.len(),
        doc.properties.len(),
        doc.signals.len()
    );
    println!(
        "\n  Use `gdcli docs {} --members` to list all members.",
        doc.name
    );
}
