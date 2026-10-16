# Dimension text position codec repair

Current base: opencadcodec `325fa98d2b881f48e106a830d5cf161624f63131` (0.6.0). The unchanged patch also applies to its original `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` base.
Apply `0001-text-position-flag.patch` as the only remaining local repair,
using the shared [codec configuration](../opencadcodec-viewports/README.md).
The composite CI action selects this repair for Rust checks and the browser
processor; both browser cache keys include this patch directory.

The pinned DWG reader and writer incorrectly treat flags bit 0 as equivalent to
DXF group 70 bit 128. [ODA Common Dimension Data, section 20.4.22](https://static.opendesign.com/files/guestdownloads/OpenDesign_Specification_for_.dwg_files.pdf)
defines the inverse polarity: clear means user-positioned text, set means default
placement. The patch corrects both directions and the public field comment,
preserving all other flag bits. It changes no public field or native format.

The foundation source exposes the issue independently of codec roundtrips:
two rotated dimensions have DWG flags 10 and user-positioned text in the
corresponding DXF. The old interpretation classifies them as automatic and the
converter correctly refuses unsupported rotated automatic placement. Correcting
the decoded flag qualifies their existing explicit native placement.

`cad-dimension/tests/codec_boundary.rs` checks the raw DWG bit, the decoded boolean
and unrelated bits for both placement states. A matching erroneous reader and
writer cannot pass this check merely by agreeing with one another. Existing
anchor tests check actual DXF/AC1032 transport separately.

This is local repair evidence, not evidence for the unmodified dependency.
Tracked upstream in [issue #117](https://github.com/HakanSeven12/opencadcodec/issues/117). The patch
does not automatically recognize or repair files emitted with the old polarity.
