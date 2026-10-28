# opencadcodec dependency audit — 2026-10-16

Current base: `325fa98d2b881f48e106a830d5cf161624f63131`, replacing
`ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`. Upstream still declares package
version 0.6.0; the exact Git revision identifies the update. Both converters
and all eight codec-dependent companion crates select the same revision.

## Repairs and patch selection

Viewport off-state #103, paper grid behavior #106, DXF ShadePlot #107, APPID
handle collision #111, null table handles #112 and spatial-filter matrices #113
are merged upstream. Gradient state repair #109 is included. The former local
viewport patches remain historical evidence and must not be reapplied.

The only selected local patch is
[`0001-text-position-flag.patch`](../../legacy/ifccad-convert/reference/patches/opencadcodec-dimension/0001-text-position-flag.patch).
It applies without changes to this exact base and fixes the inverse polarity
between DWG dimension flags bit 0 and DXF 70 bit 128. It remains local repair
evidence, tracked upstream in [issue #117](https://github.com/HakanSeven12/opencadcodec/issues/117).
The shared composite CI action applies that patch explicitly. Core OCDraw and
IFCCAD continue to have no codec dependency.

## Public-model review

| Changed surface | Converter treatment |
| --- | --- |
| Boxed MText, Dimension, Hatch, Viewport, Ole2Frame, Light variants | Constructors and matches follow the new API; no new native semantics. |
| `EntityCommon.r12_metadata`, including explicit zero and child viewport slots | Ordinary import diagnoses unsupported context. Spline payload v3 retains the complete optional carrier. Typed restoration refuses this context rather than guessing R12 viewport identities. |
| `Viewport.dxf_status` and `stacking` | Existing source residual inspection diagnoses nondefault unrepresented values. No current-paper or display-order choice is inferred. |
| Gradient endpoint `color_name` and `book_name` | Named identity remains unqualified. Active unsupported gradient is rejected/skipped under the existing loss policy; dormant state produces explicit loss. It is not silently reduced to RGB. |
| `TextStyle.true_type_font_flags` | Default packed value 34 remains qualified. Other values are explicitly unsupported; this update does not infer family/charset/pitch/bold/italic semantics from packed flags. |
| `LineTypeComplexData.source_flags` | Mode bits 0x01/0x02/0x04 use existing typed fields; unknown bits explicitly trigger the existing fallback/loss path. Upright mode #108 remains unqualified. |
| `TabStop::DecimalWithCharacter` | Native text parsing still rejects decimal-tab syntax explicitly; it does not use the codec parser to infer decimal markers. |
| Header legacy state, interference color, real-world scale, lighting synchronization/glyph display, polysolid dimensions, DWF precision and additional dimension defaults | Existing full typed/serialized header residual comparisons report nondefault unsupported source state. No blanket ignore list is extended. |
| Public object XDATA | Existing inventory/accessor diagnostics remain effective; contents are not silently discarded. |
| Constraint group next-node ID, dynamic-block action IDs/tail state and new dynamic-block APIs | Existing unsupported object/entity classifications apply. No dynamic constraint semantics are enabled. |
| Plot-style parsing and audit/recovery APIs | No implicit recovery or external CTB/STB loading is enabled. Existing missing table-content losses remain applicable. |
| Transparency writer method correction | Retain semantic opacity tests and physical readback; serialized method bytes are not the drawing contract. |
| Layer packed zero transparency | The reader now returns explicit opaque for packed layer zero. Both converters recognize only the exact standard single-value XDATA duplicate matching that typed opaque state; mismatched/extra/unknown metadata remains loss. |
| DWG anonymous block names | Bare anonymous names are reconstructed from live BLOCK_CONTROL order instead of potentially stale marker suffixes. Converters retain the decoded name and references; physical DWG exchange does not guarantee an authored anonymous numeric suffix. Tests compare the decoded identity, metadata and instance bindings. Memory/DXF naming remains exact. |

Source inventories, schemas, fixtures and both directions of both converter
coverage contracts remain authoritative. R12 file support upstream does not
qualify a new standalone native profile. Unmerged palette #115 and ACIS #116
changes are not part of this pin.

## Preservation compatibility

New captures use `openaec.opencadcodec.spline` payload version 3. They require
`common.r12Metadata`, including explicit null. Its object requires `viewport`
(including null) and ordered `childViewports`, preserving absent slots and zero.
Version 2 remains accepted for audited revisions `ab2eecd` and `063c106`; version
1 remains accepted for `fe69506`. Older shapes must omit the new carrier.
All revision, payload-version, source identity and native restoration conditions
continue to be checked. Unknown revisions and inconsistent shapes are rejected.
Historical literal payload fixtures remain unchanged.

## Qualification

Focused regressions cover dimension raw-bit polarity, physical DXF/DWG paper
grid and ShadePlot readback, new source-field losses and preservation version
compatibility. The final integration candidate must pass formatting, all-target
Clippy and the complete workspace test suite. Local verification records retain
the exact source/configuration, external patch, tools and command results.

No size/exchange measurement is performed or updated by this task. Existing
reports and their dependency pins remain historical accepted evidence.
