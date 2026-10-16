# CAD source snapshots

This companion crate owns audited opencadcodec spline byte DTOs only. It has no drawing-model dependency. Capture preserves exact binary64 bits and interpreted/skipped source fields; raw_record is deliberately excluded. Payload v3 capture targets 325fa98 and retains R12 common metadata; audited ab2eecd/063c106 payload-v2 and fe69506 payload-v1 decoding remain available. R12 viewport context is preserved but not qualified for typed restoration. Recorded literal fixtures retain their original provenance. Format-specific conditions, namespace binding and restoration remain in each converter.

Current codec qualification: [2026-10-16 dependency audit](../../docs/geometry/opencadcodec-update-2026-10-16.md). Only the documented Dimension patch remains selected; viewport repairs are upstream.
