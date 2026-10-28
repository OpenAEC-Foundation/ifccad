# cad-dimension

CAD-only preparation for ordinary linear and aligned dimensions. The helper uses
shared neutral dimension values and CAD handles; it does not allocate OCDraw or
IFCCAD identities. Each converter must bind its own styles, entities and owners.
The core model remains independent of opencadcodec.

Measurement comes from the stored definition, never the CAD measurement cache.
Geometric pairs include definition and explicit text anchors; they do not prove
glyphs, extension lines, markers or a complete displayed enclosure. Converters
must submit these pairs to the existing owner/occurrence tolerance engine.

DIMSTYLE and ACAD DSTYLE are qualified selectively. Sparse override presence is
retained, including false, zero, empty and equal-to-base values. Unknown dimvars
are unsupported; malformed/duplicate subsections and wrong typed values are
invalid. CAD coupled fields are accepted on output only when the exact sparse
factoring survives parsing. Per-entity arrow flips and line spacing are handled
separately with dependency and transport diagnostics.

Simple single-object DIMASSOC snaps cover line start/end, circle center and point
position. Endpoint identity must be unambiguous. Cross-owner, chained, subentity,
intersection, occurrence and trans-space relations are unsupported. Binding never
updates the stored dimension definition. CAD host regeneration/update execution
is outside the qualified codec boundary.

Pinned opencadcodec base: `325fa98d2b881f48e106a830d5cf161624f63131`.
The repository's explicit viewport-off patch is orthogonal to dimension tests;
results are for that local patch configuration, not a new dependency revision.
See the [field audit](../../docs/dimension/cad-field-audit-2026-10-09.md) and
[fixture provenance](../../tests/fixtures/dimension/PROVENANCE.md).

Current codec qualification: [2026-10-16 dependency audit](../../docs/geometry/opencadcodec-update-2026-10-16.md). Only the documented Dimension patch remains selected; viewport repairs are upstream.
