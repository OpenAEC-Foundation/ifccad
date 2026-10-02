# Codec regression fixture

`anonymous-names.dwg` is copied unchanged from opencadcodec revision
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`, under that project's MPL-2.0 license.
[Upstream fixture provenance](https://github.com/HakanSeven12/opencadcodec/blob/d96e3fa2fe5acbeac966f1db4c01142618bf9c79/tests/anonymous-names.md)
describes its generation in AutoCAD 2025 from synthetic content, without
customer data. It alternates named and anonymous block definitions and gives
each one an attributed insertion. The expected names are `NamedBefore`, `*U2`,
`NamedBetween`, `*U4`, and `*U5`.

The test checks production DWG decoding, record/marker agreement, INSERT targets
and strict OCDraw readback without the former anonymous-name repair. Attribute
semantics remain unsupported and diagnosed; this fixture does not establish
lossless attribute conversion.
