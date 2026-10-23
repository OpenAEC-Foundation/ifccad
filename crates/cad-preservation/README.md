# CAD source snapshots

This companion crate owns audited opencadcodec spline byte DTOs only. It has no drawing-model dependency. Capture preserves exact binary64 bits and interpreted/skipped source fields; raw_record is deliberately excluded. Payload v3 capture targets bd10efc; previous 325fa98 v3 snapshots remain decodable. The payload retains R12 common metadata; audited ab2eecd/063c106 payload-v2 and fe69506 payload-v1 decoding remain available. R12 viewport context is preserved but not qualified for typed restoration. Recorded literal fixtures retain their original provenance. Format-specific conditions, namespace binding and restoration remain in each converter.

Current codec qualification: [2026-10-23 dependency audit](../../docs/geometry/opencadcodec-update-2026-10-23.md). No local source patch is selected.
