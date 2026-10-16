# Dimension text position codec repair

Base: opencadcodec `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` (0.6.0).
Apply `0001-text-position-flag.patch` alongside the existing viewport-off patch,
using the shared [codec configuration](../opencadcodec-viewports/README.md).
The composite CI action selects both repairs for Rust checks and the browser
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
No upstream issue or pull request was created as part of this task. The patch
does not automatically recognize or repair files emitted with the old polarity.
