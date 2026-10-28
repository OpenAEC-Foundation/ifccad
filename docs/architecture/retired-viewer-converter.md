# Retired viewer conversion route

Explorer and the viewer CLI use the shared `opencaddocument` mapping. The active
`viewer` and `browser` crates do not depend on `ifccad-convert` or its codec.
`python scripts/check_explorer_dependencies.py` checks the actual browser target's
normal/build dependency graph, including transitive dependencies.

The old viewer CAD conversion/export APIs, adjustable-tolerance and preservation
adapters, codec readback helpers, export preflights and physical-CAD selection
qualification have been removed. Tests that qualified those retired viewer
operations no longer assert guarantees for the new route. The converter-only
Hatch version matrix now lives with the standalone converter's own tests.

Native validation and source inspection remain active through the production
core: strict JSON/CBOR/Zstd reading, composed IFCX nodes, foreign nodes/fragments,
full-width identities, stored geometry and authored state, bounds assessment,
native source downloads and binary encoding. Their viewer regression tests are
retained. Shared-route tests cover explicit unavailable-option failures, real
CAD serialization/readback, native readback and typed import selection.

## Reference and recovery

The separate `crates/ifccad-convert` crate and its standalone tests remain in the
workspace pending a dedicated archival task. Its numerical, source-loss and
preservation proofs continue to describe that converter. They do not qualify
the shared mapping; see [current route coverage](explorer-document-route.md).

The complete former viewer implementation and tests are recoverable from
[`f80d7bd84389b9dff5c01befa3e4bccc87f10ee8`](https://github.com/OpenAEC-Foundation/ifccad/tree/f80d7bd84389b9dff5c01befa3e4bccc87f10ee8/crates/viewer).
Use an isolated checkout of that revision with its documented reference codec
patch preparation for reproduction. Restore the complete recorded slice rather
than reintroducing isolated old helpers into the active Explorer.

The reference workspace's CI preparation can still select its patched codec for
standalone converter tests and workspace dependency resolution. The browser
artifact excludes those packages; after their separate archival, that preparation
can be retired with them.
