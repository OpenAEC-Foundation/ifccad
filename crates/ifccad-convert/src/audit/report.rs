use super::*;
use std::fmt::Write;

impl CadToIfccadAuditReport {
    /// Grouped human-readable inventory; the JSON retains individual evidence.
    pub fn markdown(&self) -> String {
        let mut out = format!("# CAD conversion diagnosis\n\nLogical conversion ready: **{}**. Eligible scope traversal complete: **{}**.\n\n", self.conversion_ready, self.traversal_complete);
        out.push_str("This is a report-only pass. No entities were pre-excluded or removed from the source. No encoded drawing, file readback, benchmark or losslessness certification is produced. Each failed preparation reports its first hard failure; dependent and later checks may be blocked.\n\n");
        let mut statuses = BTreeMap::<String, usize>::new();
        for e in &self.entities {
            *statuses.entry(format!("{:?}", e.status)).or_default() += 1;
        }
        out.push_str(
            "## Stored source coverage\n\n| Preparation status | Entities |\n| --- | ---: |\n",
        );
        for (status, count) in statuses {
            let _ = writeln!(out, "| {status} | {count} |");
        }
        out.push_str("\n## Hard failures\n\n| Phase | Kind | Category | Count | Examples |\n| --- | --- | --- | ---: | --- |\n");
        let mut groups = BTreeMap::<(&str, &str, &str), Vec<&CadAuditEvent>>::new();
        for e in self.failures() {
            groups
                .entry((
                    &e.phase,
                    e.source.as_ref().map_or("drawing", |s| s.kind.as_str()),
                    &e.code,
                ))
                .or_default()
                .push(e);
        }
        for ((phase, kind, code), events) in groups {
            let examples = events
                .iter()
                .take(3)
                .filter_map(|e| e.source.as_ref().map(|s| s.handle.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(
                out,
                "| {phase} | {kind} | {code} | {} | {examples} |",
                events.len()
            );
        }
        out.push_str("\n## Semantic diagnostics and recoveries\n\n| Action | Code | Count |\n| --- | --- | ---: |\n");
        let mut warnings = BTreeMap::<(String, &str), usize>::new();
        for d in &self.diagnostics {
            *warnings
                .entry((format!("{:?}", d.action), d.code))
                .or_default() += 1;
        }
        for ((action, code), count) in warnings {
            let _ = writeln!(out, "| {action} | {code} | {count} |");
        }
        out.push_str("\nDiagnostics are records, not counts of lost entities. Passing preparation does not imply complete presentation, fill, font, Dimension display or source fidelity.\n\n## Blocked checks\n\n");
        let mut blocked = BTreeMap::<(&str, &str), usize>::new();
        for e in &self.events {
            if e.status == CadAuditStatus::Blocked {
                *blocked.entry((&e.phase, &e.message)).or_default() += 1;
            }
        }
        for ((phase, message), count) in blocked {
            let _ = writeln!(out, "- {phase}: {message} ({count})");
        }
        out
    }
}
