# Leader family (development profile 0.7.0 / CBOR7)

The ordinary native kinds are **Leader**, **MultiLeader** and **Tolerance**
(CBOR kind codes 17–19). The released 0.6.0 contract remains fixed in its tag.
This development contract does not change the optional Zstd transport revision.
The core model, validation and file IO have no CAD runtime dependency.

A Leader retains its straight/spline interpolation intent, vertices, placement,
DIMSTYLE reference, typed overrides, arrow and hook state, offsets and optional
annotation dimensions. Its annotation is an optional ordinary, same-owner
MText, block instance or Tolerance entity. A shared target remains one entity.
A missing target is allowed; a supplied wrong-kind, foreign-owner or dangling
target is rejected. `DIMLDRBLK` is independent of the dimension arrow pair.
Explicit equal supported override values retain their presence.

A MultiLeader owns ordered groups and lines, content and compact block labels.
These are IFCX child nodes, not ordinary drawing entities. Positive local keys
and their allocation watermark are independent of optional CAD source indices.
Segment breaks refer to the effective path, including a separately stored group
endpoint when active. Inactive stored directions and independent matrices stay
stored. A label binds directly to an ATTDEF identity; tag/order guessing is not
a substitute for that relationship. Block labels do not become INSERT attributes.

MLEADERSTYLE resources and entity settings share common typed values. Override
presence, base values and per-scale presentation values are independent. Saved
base and scale presentations own their own plane, points, breaks and content.
Positions are local to the presentation frame; directions have no translation.
The Document adapter transforms them once at each boundary. Text uses the
existing structured MText contract. Its separately stored CAD rotation is an
optional `storedRotation`; it is never recomputed from the text direction.

Native IO preserves dormant text, block and tolerance payloads. Physical CAD
output preflights unsupported combinations: the selected DWG writer uses text
**else** block, so simultaneous text and block payloads are refused before
writing. Existing shared dimension-style and MText conversion limits continue
to apply, with diagnostics or explicit errors rather than silent omission.
A missing source MLeader linetype means the actual declared CONTINUOUS resource;
an absent text style uses the actual declared Standard resource. Neither is a
fabricated resource or a ByLayer guess. Custom non-null arrow blocks keep their
identity. A canonical empty `_NONE` block can carry an absent arrow; a custom
same-named block containing geometry remains a symbol.

Tolerance retains literal `CadFeatureControlTextV1` notation, its placement,
DIMSTYLE, typed overrides and independent scale poses. This is preservation of
feature-control-frame controls, not GD&T interpretation or a text-field engine.
Owned MultiLeader tolerance content is also a complete native value and can be
strictly read/written in IFCX and CBOR. Its **physical source payload remains
open**. A self-generated H1 experiment (type 3 plus a text-context FCF string)
qualifies only the selected codec's internal tag/string retention. It does not
establish an independent CAD payload layout, attachment semantics or DIMSTYLE
link. Production Document conversion refuses that unqualified interpretation.

Saved graphics are an optional ordered, typed command stream, independent of
definitions and scale membership. Layers and line patterns are native resource
references, rebound from actual CAD table storage order. The codec validates
record framing/counts, finite values, UTF-16 and bounded padding. Legacy text
requires a faithful declared codepage. Native capture is atomic: an unknown or
malformed record never produces a partially successful picture. The bounded
zero-word compatibility record 51 is preserved exactly; no renderer no-op claim
is made. Record 21 remains unqualified. Generated/source snapshot provenance is
separate; CAD reimport normalizes a generated picture to a source snapshot and
reports that normalization.

Storage, inspection, rendered appearance and editing are separate claims.
Native bounds for these three kinds are unavailable; IO does not synthesize
glyph bounds, spline geometry or refresh a saved picture. Explorer inspection
keeps parts under their owner, and selection of an owned part selects that
ordinary CAD owner. Creation and editing belong to Document and its host.

The active schemas, CBOR register and `conformance/next` are normative. Numbered
released collections are unchanged. Development verification and review-file
provenance are recorded separately; a stored picture is not evidence of a
qualified editable physical tolerance payload.
