use super::{accounting, adapters::Result, projection, run};
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
        .find(|p| p["name"] == "acadrust")
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
    let mut sources = Vec::new();
    for path in files.lines().filter(|p| {
        p.ends_with(".rs")
            || p.ends_with("Cargo.toml")
            || p.starts_with("schemas/")
            || p.starts_with("conformance/next/")
    }) {
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
        "rustc":rustc,
        "cargo_lock_sha256":accounting::digest(&lock), "dependencies":dependencies,
        "cadcodec":dependency_source,
        "writers":{"ifccad":"current pretty JSON, uncompressed directory package", "dxf":"cadcodec text AC1032", "dwg":"cadcodec normal AC1032, native compression"}}),
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

pub fn repeatability(
    first: &Value,
    second: &Value,
    first_files: &Value,
    second_files: &Value,
) -> Value {
    let fields = projection::first_difference(first, second, "");
    let artifacts = projection::first_difference(first_files, second_files, "/artifacts");
    let mut direct_outputs = Vec::new();
    if let (Some(a), Some(b)) = (first["cases"].as_array(), second["cases"].as_array()) {
        for (a, b) in a.iter().zip(b) {
            for mode in ["ifccad-external", "ifccad-inline", "dxf", "dwg"] {
                let left = &a["outputs"][mode];
                let right = &b["outputs"][mode];
                direct_outputs.push(json!({"case":a["id"], "mode":mode,
                    "sizes_equal":left["bytes"].is_object() && left["bytes"] == right["bytes"],
                    "files_identical":left["files"].is_array() && left["files"] == right["files"],
                    "first_bytes":left["bytes"]["total"], "second_bytes":right["bytes"]["total"]}));
            }
        }
    }
    json!({"status":if fields.is_none() && artifacts.is_none() {"passed"} else {"failed"},
        "measurement_difference":fields,"artifact_difference":artifacts,"direct_outputs":direct_outputs})
}

fn cell(value: &Value) -> String {
    if value.is_null() {
        "—".into()
    } else if let Some(value) = value.as_f64() {
        if value.fract() == 0. {
            format!("{value:.0}")
        } else {
            format!("{value:.3}")
        }
    } else {
        value.as_str().unwrap_or("?").into()
    }
}
fn failure(route: &Value) -> String {
    if let Some(stages) = route["stages"].as_array() {
        if let Some(stage) = stages.iter().find(|s| s["status"] == "failed") {
            return format!("{}: {}", cell(&stage["stage"]), cell(&stage["error"]));
        }
    }
    cell(&route["validation"]["error"])
}
pub fn markdown(result: &Value) -> String {
    let mut out = format!("# IFCCAD JSON / DXF / DWG size baseline v1\n\n**Outcome: {}.** Repeatability: {}. Corpus: {}.\n\n",
        cell(&result["status"]), cell(&result["repeatability"]["status"]),cell(&result["corpus_scope"]));
    out.push_str("Controlled fixtures and generated XY lines/straight polylines; millimetres, AC1032. IFCCAD is uncompressed pretty JSON, DXF is text, DWG uses its normal native compression. These ratios compare complete writer outputs, not compression algorithms or representative CAD practice.\n\n");
    if result["provenance"]["cadcodec"]["local_override"] == true {
        out.push_str("This run uses a **local cadcodec patch candidate**. Its exact source hash is recorded in the provenance. Passing checks do not update the repository's pinned dependency or automatically accept a release baseline.\n\n");
    }
    if result["status"] != "passed" {
        out.push_str("This is an **incomplete diagnostic run**, not an accepted baseline or a completed milestone. Observed sizes remain useful, but failed semantic comparisons have no ratio. A failed conversion chain prevents a full exchange conclusion even where direct readback passes.\n\n");
    }
    out.push_str("## Observed complete-file bytes\n\n| Case | Lines | Polylines | Vertices | IFCCAD external | IFCCAD inline | DXF | DWG | External / DXF | External / DWG | Inline / DXF | Inline / DWG |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    if let Some(cases) = result["measurements"]["cases"].as_array() {
        for case in cases {
            let outputs = &case["outputs"];
            let values = [
                &case["id"],
                &case["counts"]["lines"],
                &case["counts"]["polylines"],
                &case["counts"]["polyline_vertices"],
                &outputs["ifccad-external"]["bytes"]["total"],
                &outputs["ifccad-inline"]["bytes"]["total"],
                &outputs["dxf"]["bytes"]["total"],
                &outputs["dwg"]["bytes"]["total"],
                &outputs["ifccad-external"]["ratio_to_dxf"],
                &outputs["ifccad-external"]["ratio_to_dwg"],
                &outputs["ifccad-inline"]["ratio_to_dxf"],
                &outputs["ifccad-inline"]["ratio_to_dwg"],
            ];
            out.push_str(&format!(
                "| {} |\n",
                values
                    .iter()
                    .map(|v| cell(v))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
    }
    out.push_str("\n## Component accounting (bytes)\n\nInline bodies count only inside IFCX. Every physical blob is counted once. Preservation-inclusive fixture totals are not a native-package estimate.\n\n| Case / mode | IFCX | External IFCDR | External IFCPR | Blobs | Total |\n| --- | ---: | ---: | ---: | ---: | ---: |\n");
    let mut component = |name: &str, row: &Value| {
        out.push_str(&format!(
            "| {name} | {} |\n",
            ["ifcx", "ifcdr", "ifcpr", "blob", "total"]
                .iter()
                .map(|k| cell(&row["bytes"][k]))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    };
    if let Some(cases) = result["measurements"]["cases"].as_array() {
        for case in cases {
            for mode in ["ifccad-external", "ifccad-inline"] {
                component(
                    &format!("{} / {mode}", cell(&case["id"])),
                    &case["outputs"][mode],
                );
            }
        }
    }
    if let Some(fixtures) = result["measurements"]["fixtures"].as_array() {
        for row in fixtures {
            component(
                &format!("fixture {} ({})", cell(&row["id"]), cell(&row["kind"])),
                row,
            );
        }
    }
    out.push_str("\n## Executed checks\n\nEvery passing comparison checks each entity in order, exact XY geometry, closure, layer, visibility and all four appearance modes/values, plus unit and layers including unused ones. IFCCAD storage variants additionally retain the same ordered entity IDs. CAD handles and technical default tables are outside this projection.\n\n| Case | Route | Result | First failure |\n| --- | --- | --- | --- |\n");
    if let Some(cases) = result["measurements"]["cases"].as_array() {
        for case in cases {
            for mode in ["ifccad-external", "ifccad-inline", "dxf", "dwg"] {
                let row = &case["outputs"][mode];
                out.push_str(&format!(
                    "| {} | Direct {mode} readback | {} | {} |\n",
                    cell(&case["id"]),
                    cell(&row["status"]),
                    failure(row).replace('|', "/")
                ));
            }
            for format in ["dxf", "dwg"] {
                let row = &case["chains"][format];
                out.push_str(&format!(
                    "| {} | IFCCAD → {format} → IFCCAD (Reject) | {} | {} |\n",
                    cell(&case["id"]),
                    cell(&row["status"]),
                    failure(row).replace('|', "/")
                ));
            }
        }
    }
    out.push_str("\nThe JSON retains executed stages, import/export diagnostics and scoped assessments. Import remains NotFullyAssessed for the general converter contract even when this recipe's explicit properties match. The source-archive fixture retains preservationSemanticsNotAssessed; its accounting does not prove IFCPR fidelity. Failed stages stop dependent stages, which are not counted as executed.\n\n## Repeatability and reproduction\n\n");
    out.push_str(&format!("Repository revision: {} (dirty: {}). Two fresh runs compare measurements and hashes of every produced artifact, including chain outputs.\n\n",cell(&result["provenance"]["repository_revision"]),result["provenance"]["repository_dirty"]));
    out.push_str(
        "| Case | Output | Same byte count | Identical files |\n| --- | --- | --- | --- |\n",
    );
    if let Some(rows) = result["repeatability"]["direct_outputs"].as_array() {
        for row in rows {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                cell(&row["case"]),
                cell(&row["mode"]),
                row["sizes_equal"],
                row["files_identical"]
            ));
        }
    }
    out.push_str(&format!(
        "\nFirst measurement difference: {}. First artifact difference: {}.\n\n",
        cell(&result["repeatability"]["measurement_difference"]),
        cell(&result["repeatability"]["artifact_difference"])
    ));
    if result["provenance"]["cadcodec"]["local_override"] == true {
        out.push_str("To reproduce a local candidate, prepare the matching base revision and patches documented under `patches/`. Pass its Cargo config to both Cargo (`--config`) and this example (`--cargo-config`), and choose a **new** run name. Check the base revision and source hash against this run's provenance.\n\n");
    } else {
        out.push_str("Run from the repository root with a **new** run name:\n\n```text\ncargo run -p ifccad-convert --example size_baseline -- --run baseline-v1-review\n```\n\n");
    }
    out.push_str("Add `--case mixed-1000` for a labelled partial diagnostic run. The command returns failure for failed checks or nondeterminism, retaining its report. It never replaces an accepted baseline or overwrites an existing run.\n\n");
    out.push_str("The local run directory contains results.json, report.md, provenance.json, source-manifest.json, Cargo.lock, first/ and second/. Each case contains ifccad-external/, ifccad-inline/, drawing.dxf, drawing.dwg and separately labelled chain artifacts. Header snapshots support investigation of rejected CAD metadata. Exact dependency versions and source/lock/corpus hashes are in the JSON. Generated outputs and environment details remain below target/ and are not committed.\n\n");
    out.push_str("## Interpretation limits\n\nThe empty case measures fixed overhead; the two line/polyline scales expose growth, and long polylines isolate vertex-heavy storage. Inline placement changes JSON nesting and package metadata as well as the number of files. Fractional coordinates test numeric text at exact binary fractions. No result generalizes to arbitrary decimal precision, unsupported entities, external CAD applications, real-world files, runtime or memory. No size threshold or new physical encoding follows from this experiment.\n");
    out
}

pub fn finish(
    output: &Path,
    provenance: Value,
    corpus_hash: String,
    selected: Option<&str>,
    first: Value,
    second: Value,
) -> Result<bool> {
    let first_files = artifact_manifest(&output.join("first"))?;
    let second_files = artifact_manifest(&output.join("second"))?;
    let repeated = repeatability(&first, &second, &first_files, &second_files);
    let success =
        run::successful(&first) && run::successful(&second) && repeated["status"] == "passed";
    let fixture_files: Vec<_> = first["fixtures"]
        .as_array()
        .ok_or("fixture measurements missing")?
        .iter()
        .map(|f| json!({"id":f["id"],"files":f["files"]}))
        .collect();
    let corpus_fingerprint = accounting::digest(&serde_json::to_vec(
        &json!({"inventory_sha256":corpus_hash,"fixtures":fixture_files}),
    )?);
    let result = json!({"status":if success {"passed"} else {"incomplete"}, "corpus_scope":selected.unwrap_or("full-v1"),
        "baseline_accepted":success && selected.is_none() && provenance["cadcodec"]["local_override"] != true,
        "corpus_inventory_sha256":corpus_hash,"corpus_sha256":corpus_fingerprint, "provenance":provenance,"repeatability":repeated,
        "measurements":first,"second_measurements":second,"first_artifacts":first_files,"second_artifacts":second_files});
    run::write_json(&output.join("results.json"), &result)?;
    fs::write(output.join("report.md"), markdown(&result))?;
    let mut index = String::from("# Local generated artifacts\n\nThese files are diagnostic outputs, not an accepted exchange baseline.\n\n");
    for pass in ["first", "second"] {
        index.push_str(&format!("## {pass} generation\n\n"));
        for file in result[format!("{pass}_artifacts")]
            .as_array()
            .ok_or("artifact manifest missing")?
        {
            let path = file["path"].as_str().ok_or("artifact path missing")?;
            index.push_str(&format!("- [{path}]({pass}/{path})\n"));
        }
        index.push('\n');
    }
    fs::write(output.join("artifacts.md"), index)?;
    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_changed_hash_or_measurement_fails_repeatability() {
        let row = json!({"bytes":1});
        assert_eq!(
            repeatability(&row, &row, &json!([]), &json!([]))["status"],
            "passed"
        );
        assert_eq!(
            repeatability(&row, &json!({"bytes":2}), &json!([]), &json!([]))["status"],
            "failed"
        );
        assert_eq!(
            repeatability(&row, &row, &json!(["hash-a"]), &json!(["hash-b"]))["status"],
            "failed"
        );
    }
}
