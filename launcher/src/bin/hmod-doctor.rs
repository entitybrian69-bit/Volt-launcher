//! Inspect Horizon .hmod packages without launching the game.
//!
//! Usage:
//!   cargo run --manifest-path launcher/Cargo.toml --bin hmod-doctor -- --dir /path/to/horizon-mods
//!   cargo run --manifest-path launcher/Cargo.toml --bin hmod-doctor -- --dir ./my_mod.hmod --json
use horizon_mods::{Capability, Content, Package};
use serde_json::json;
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};

struct Report {
    file: String,
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    api: Option<String>,
    side: Option<String>,
    kind: Option<String>,
    capabilities: Vec<String>,
    valid: bool,
    problem: Option<String>,
}

fn inspect(path: &Path) -> Report {
    let file = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let package = match Package::read(path) {
        Ok(package) => package,
        Err(error) => {
            return Report {
                file,
                id: None,
                name: None,
                version: None,
                api: None,
                side: None,
                kind: None,
                capabilities: Vec::new(),
                valid: false,
                problem: Some(error),
            };
        }
    };

    let manifest = &package.manifest;
    let has_code = package.wasm.is_some();
    let has_content = manifest.capabilities.contains(&Capability::Registries);
    let content_check = if has_content {
        Content::read(path, manifest).map(|content| content.files.len())
    } else {
        Ok(0)
    };

    let (valid, problem, file_count) = match content_check {
        Ok(count) => (true, None, count),
        Err(error) => (false, Some(error), 0),
    };
    let kind = match (has_code, has_content) {
        (true, true) => format!("hybrid code + content ({file_count} content files)"),
        (true, false) => "WASM code mod".to_owned(),
        (false, true) => format!("content-only ({file_count} content files)"),
        (false, false) => "invalid: no code or registries".to_owned(),
    };

    Report {
        file,
        id: Some(manifest.id.clone()),
        name: Some(manifest.name.clone()),
        version: Some(manifest.version.clone()),
        api: Some(manifest.api.clone()),
        side: Some(format!("{:?}", manifest.side).to_lowercase()),
        kind: Some(kind),
        capabilities: manifest.capabilities.iter().map(ToString::to_string).collect(),
        valid,
        problem,
    }
}

fn print_help() {
    println!(
        "Volt HMOD Doctor\n\
         Inspect Horizon .hmod files without starting Minecraft.\n\n\
         Usage: hmod-doctor [--dir PATH] [--json]\n\n\
         --dir PATH   An .hmod file, a mod folder, or a folder containing .hmod files\n\
         --json       Print machine-readable JSON\n\
         -h, --help   Show this help"
    );
}

fn main() {
    let mut args = env::args().skip(1);
    let mut target = PathBuf::from("horizon-mods");
    let mut as_json = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dir" => match args.next() {
                Some(path) => target = PathBuf::from(path),
                None => {
                    eprintln!("--dir needs a path");
                    std::process::exit(2);
                }
            },
            "--json" => as_json = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                print_help();
                std::process::exit(2);
            }
        }
    }

    let paths: Vec<PathBuf> = if target.is_file() || target.join("horizon-mod.toml").is_file() {
        vec![target.clone()]
    } else {
        match std::fs::read_dir(&target) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "hmod"))
                .collect(),
            Err(error) => {
                eprintln!("{}: {error}", target.display());
                std::process::exit(2);
            }
        }
    };

    let mut reports: Vec<Report> = paths.iter().map(|path| inspect(path)).collect();
    reports.sort_by(|a, b| a.file.cmp(&b.file));

    let mut ids = BTreeMap::<String, String>::new();
    let mut duplicates = Vec::new();
    for report in &reports {
        if let Some(id) = &report.id {
            if let Some(first) = ids.insert(id.clone(), report.file.clone()) {
                duplicates.push(format!("{id}: {} and {}", first, report.file));
            }
        }
    }

    let valid = reports.iter().filter(|r| r.valid).count();
    let invalid = reports.len() - valid;
    let overall_ok = invalid == 0 && duplicates.is_empty();

    if as_json {
        let mods: Vec<_> = reports
            .iter()
            .map(|r| json!({
                "file": r.file,
                "id": r.id,
                "name": r.name,
                "version": r.version,
                "api": r.api,
                "side": r.side,
                "kind": r.kind,
                "capabilities": r.capabilities,
                "valid": r.valid,
                "problem": r.problem,
            }))
            .collect();
        println!("{}", json!({
            "directory": target,
            "total": reports.len(),
            "valid": valid,
            "invalid": invalid,
            "duplicates": duplicates,
            "ok": overall_ok,
            "mods": mods,
        }));
    } else {
        println!("HMOD Doctor — {}", target.display());
        if reports.is_empty() {
            println!("No .hmod packages found.");
        }
        for report in &reports {
            let mark = if report.valid { "OK" } else { "FAIL" };
            println!("[{mark}] {}", report.file);
            if let Some(name) = &report.name {
                println!(
                    "  {} {}  · API {} · {}",
                    name,
                    report.version.as_deref().unwrap_or("?"),
                    report.api.as_deref().unwrap_or("?"),
                    report.kind.as_deref().unwrap_or("?")
                );
                println!(
                    "  id={}  side={}  capabilities={}",
                    report.id.as_deref().unwrap_or("?"),
                    report.side.as_deref().unwrap_or("?"),
                    if report.capabilities.is_empty() {
                        "none".to_owned()
                    } else {
                        report.capabilities.join(", ")
                    }
                );
            }
            if let Some(problem) = &report.problem {
                println!("  Problem: {problem}");
            }
        }
        for duplicate in &duplicates {
            println!("[DUPLICATE] {duplicate}");
        }
        println!(
            "\n{} package(s): {} valid, {} invalid{}",
            reports.len(),
            valid,
            invalid,
            if duplicates.is_empty() {
                String::new()
            } else {
                format!(", {} duplicate ID(s)", duplicates.len())
            }
        );
    }

    if !overall_ok {
        std::process::exit(1);
    }
}
