# CAD source snapshots

This companion crate owns audited opencadcodec spline byte DTOs only. It has no drawing-model dependency. Capture preserves exact binary64 bits and interpreted/skipped source fields; raw_record is deliberately excluded. Payload v2 targets 063c106; v1 decoding remains available for fe69506. Format-specific conditions, namespace binding and restoration remain in each converter.
