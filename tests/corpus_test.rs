use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use gdcli::scene_parser::{parse_scene_text, write_scene};

fn corpus_dir() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

fn collect_scene_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, out);
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy();
            if ext == "tscn" || ext == "tres" {
                out.push(path);
            }
        }
    }
}

fn first_diff_line(a: &str, b: &str) -> String {
    for (i, (la, lb)) in a.lines().zip(b.lines()).enumerate() {
        if la != lb {
            return format!(
                "line {}: first write has {:?}, second write has {:?}",
                i + 1,
                la,
                lb
            );
        }
    }
    let count_a = a.lines().count();
    let count_b = b.lines().count();
    if count_a != count_b {
        return format!(
            "line count differs: first write has {}, second write has {}",
            count_a, count_b
        );
    }
    String::from("(no difference found)")
}

macro_rules! skip_without_corpus {
    () => {
        match corpus_dir() {
            Some(dir) => dir,
            None => {
                eprintln!(
                    "SKIPPED: corpus not found. Run `bash tests/fetch_corpus.sh` to fetch it."
                );
                return;
            }
        }
    };
}

struct CorpusResults {
    passed: usize,
    parse_warnings: Vec<String>,
    skipped_tres: usize,
    failures: Vec<String>,
}

impl CorpusResults {
    fn new() -> Self {
        Self {
            passed: 0,
            parse_warnings: Vec::new(),
            skipped_tres: 0,
            failures: Vec::new(),
        }
    }

    fn print_summary(&self) {
        eprintln!("\n=== Corpus Round-Trip Summary ===");
        eprintln!("  Passed (idempotent):  {}", self.passed);
        eprintln!("  Parse warnings:       {}", self.parse_warnings.len());
        eprintln!("  Skipped (.tres):      {}", self.skipped_tres);
        eprintln!("  FAILURES:             {}", self.failures.len());

        if !self.parse_warnings.is_empty() {
            eprintln!("\n-- Parse warnings (not fatal) --");
            for w in &self.parse_warnings {
                eprintln!("  WARN: {}", w);
            }
        }

        if !self.failures.is_empty() {
            eprintln!("\n-- HARD FAILURES --");
            for f in &self.failures {
                eprintln!("  FAIL: {}", f);
            }
        }
        eprintln!("=================================\n");
    }
}

#[test]
fn corpus_round_trip() {
    let dir = skip_without_corpus!();
    let files = collect_scene_files(&dir);

    assert!(
        !files.is_empty(),
        "Corpus directory exists but contains no .tscn/.tres files"
    );

    eprintln!("Found {} scene files in corpus", files.len());

    let mut results = CorpusResults::new();

    for path in &files {
        let rel = path.strip_prefix(&dir).unwrap_or(path);
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip .tres files (known limitation)
        if ext == "tres" {
            results.skipped_tres += 1;
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                results
                    .parse_warnings
                    .push(format!("{}: read error: {}", rel.display(), e));
                continue;
            }
        };

        // First parse
        let scene_a = match parse_scene_text(&content) {
            Ok(s) => s,
            Err(e) => {
                results
                    .parse_warnings
                    .push(format!("{}: {}", rel.display(), e));
                continue;
            }
        };

        // First write
        let text1 = write_scene(&scene_a);

        // Re-parse our own output (hard fail if this errors)
        let scene_b = match parse_scene_text(&text1) {
            Ok(s) => s,
            Err(e) => {
                let mut msg = String::new();
                let _ = write!(
                    msg,
                    "{}: re-parse of our own output failed: {}",
                    rel.display(),
                    e
                );
                results.failures.push(msg);
                continue;
            }
        };

        // Second write
        let text2 = write_scene(&scene_b);

        // Idempotency check (hard fail if different)
        if text1 != text2 {
            let diff = first_diff_line(&text1, &text2);
            let mut msg = String::new();
            let _ = write!(
                msg,
                "{}: non-idempotent serialization — {}",
                rel.display(),
                diff
            );
            results.failures.push(msg);
            continue;
        }

        results.passed += 1;
    }

    results.print_summary();

    if !results.failures.is_empty() {
        panic!(
            "{} corpus file(s) had hard failures (see above)",
            results.failures.len()
        );
    }
}
