use super::{accounting, adapters::Result, run};
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

fn command(repo: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

pub fn provenance(repo: &Path, output: &Path, cargo_config: Option<&str>) -> Result<Value> {
    let rustc = command(repo, "rustc", &["--version", "--verbose"])?;
    let host = rustc
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("rustc host missing")?;
    let mut metadata_args = vec![
        "metadata",
        "--format-version",
        "1",
        "--offline",
        "--filter-platform",
        host,
        "--locked",
    ];
    if let Some(config) = cargo_config {
        metadata_args.extend(["--config", config]);
    }
    let metadata: Value = serde_json::from_str(&command(repo, "cargo", &metadata_args)?)?;
    let lock = fs::read(repo.join("Cargo.lock"))?;
    fs::write(output.join("Cargo.lock"), &lock)?;
    let mut dependencies: Vec<_> = metadata["packages"]
        .as_array()
        .ok_or("dependency metadata missing")?
        .iter()
        .map(|p| json!({"name":p["name"],"version":p["version"],"source":p["source"]}))
        .collect();
    dependencies.sort_by_key(|p| format!("{}:{}:{}", p["name"], p["version"], p["source"]));
    let cadcodec = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "opencadcodec")
        .ok_or("cadcodec dependency missing")?;
    let local_override = cadcodec["source"].is_null();
    let mut dependency_source =
        json!({"local_override":local_override,"source":cadcodec["source"]});
    if local_override {
        let manifest = Path::new(
            cadcodec["manifest_path"]
                .as_str()
                .ok_or("cadcodec manifest missing")?,
        );
        let root = manifest.parent().ok_or("cadcodec source root missing")?;
        // Hash actual patched source, not just the unchanged package version.
        let mut files = artifact_manifest(&root.join("src"))?
            .as_array()
            .unwrap()
            .clone();
        files.push(serde_json::to_value(accounting::file(
            "Cargo.toml",
            "source",
            &fs::read(manifest)?,
        ))?);
        dependency_source["source_sha256"] =
            json!(accounting::digest(&serde_json::to_vec(&files)?));
        dependency_source["base_revision"] = json!(command(root, "git", &["rev-parse", "HEAD"])?);
        run::write_json(&output.join("cadcodec-source-manifest.json"), &json!(files))?;
    }
    // Hash actual source bytes, including uncommitted/untracked benchmark code.
    let files = command(
        repo,
        "git",
        &["ls-files", "--cached", "--others", "--exclude-standard"],
    )?;
    // Cached paths include files removed in the working tree. Record those
    // deletions separately; the manifest hashes only source bytes that exist.
    let deleted = command(repo, "git", &["ls-files", "--deleted"])?;
    let deleted_paths: std::collections::BTreeSet<_> = deleted.lines().collect();
    let mut sources = Vec::new();
    for path in files.lines().filter(|p| {
        p.ends_with(".rs")
            || p.ends_with("Cargo.toml")
            || p.starts_with("schemas/")
            || p.starts_with("conformance/next/")
    }) {
        if deleted_paths.contains(path) {
            continue;
        }
        sources.push(accounting::file(
            path,
            "source",
            &fs::read(repo.join(path))?,
        ));
    }
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    let source_json = serde_json::to_vec(&sources)?;
    fs::write(
        output.join("source-manifest.json"),
        serde_json::to_vec_pretty(&sources)?,
    )?;
    Ok(
        json!({"repository_revision":command(repo,"git",&["rev-parse","HEAD"])?,
        "repository_dirty":!command(repo,"git",&["status","--porcelain"])?.is_empty(),
        "source_manifest_sha256":accounting::digest(&source_json),
        "deleted_source_paths":deleted_paths,
        "rustc":rustc,
        "cargo_lock_sha256":accounting::digest(&lock), "dependencies":dependencies,
        "cadcodec":dependency_source,
        "writers":{"ocdraw":"current pretty JSON, uncompressed standalone file", "dxf":"cadcodec text AC1032", "dwg":"cadcodec normal AC1032, native compression"}}),
    )
}

pub fn artifact_manifest(root: &Path) -> Result<Value> {
    fn visit(
        root: &Path,
        directory: &Path,
        files: &mut Vec<accounting::FileMeasurement>,
    ) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                visit(root, &path, files)?;
            } else if entry.file_type()?.is_file() {
                let relative = path
                    .strip_prefix(root)?
                    .to_str()
                    .ok_or("non-UTF8 artifact path")?
                    .replace('\\', "/");
                files.push(accounting::file(&relative, "artifact", &fs::read(&path)?));
            } else {
                return Err("unexpected nonregular artifact".into());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(serde_json::to_value(files)?)
}

pub fn finish(
    output: &Path,
    provenance: Value,
    corpus_hash: String,
    selected: Option<&str>,
    first: Value,
    second: Value,
) -> Result<bool> {
    let repeatable = first == second;
    let passed = repeatable
        && first["cases"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty() && rows.iter().all(|r| r["status"] == "passed"));
    let result = json!({"status":if passed{"passed"}else{"failed"},"repeatability":if repeatable{"passed"}else{"failed"},"corpus_scope":selected.unwrap_or("primitive-v1"),"corpus_sha256":corpus_hash,"provenance":provenance,"measurements":first});
    run::write_json(&output.join("results.json"), &result)?;
    let mut report=format!("# OCDraw JSON / DXF / DWG primitive baseline\n\nOutcome: {}. Repeatability: {}.\n\nStandalone pretty JSON, text DXF, normally compressed DWG; actual writer outputs and semantic readback, two fresh passes. The existing v1 primitive recipes are reused; historical package/preservation fixtures and inline/external alternatives are outside this standalone experiment. No claim about typical CAD files or compression algorithms.\n\n| Case | OCDraw bytes | DXF bytes | DWG bytes |\n| --- | ---: | ---: | ---: |\n",result["status"],result["repeatability"]);
    for row in result["measurements"]["cases"].as_array().unwrap() {
        report.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            row["id"].as_str().unwrap(),
            row["outputs"]["ocdraw"]["bytes"],
            row["outputs"]["dxf"]["bytes"],
            row["outputs"]["dwg"]["bytes"]
        ));
    }
    fs::write(output.join("report.md"), report)?;
    Ok(passed)
}
