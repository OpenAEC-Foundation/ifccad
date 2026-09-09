# Resource source contract 0.9.0

This document is normative alongside IFCX overlay 0.9.0. It retains the
Drawing/layout relationship rules in [drawing-resource-contract-0.8.0.md](drawing-resource-contract-0.8.0.md)
and supersedes that document's selection of overlay 0.8.0. Drawing core remains
0.2.0, IFCDR logical and JSON contracts remain 0.7.0, and IFCPR remains 0.2.0.

## Source alternatives

DrawingRepresentation uses `attributes.resource`; PreservationRepresentation
uses `attributes.preservation`. Each descriptor MUST select exactly one source:

- External: `uri` and `checksum` are required and `content` is forbidden.
- Inline: `content` is required and `uri` and `checksum` are forbidden.

Inline content MUST be the complete resource JSON object, including its header.
It is not a JSON string, array, null or base64 payload. Forbidden fields remain
forbidden when their value is null. Common and kind-specific descriptor metadata
remain required for both sources. Content receives the applicable IFCDR or
IFCPR validation independently of how it is stored.

External checksums are `sha256:` followed by the lowercase hexadecimal SHA-256
digest of the exact resource file bytes, including whitespace and line endings.
Inline content has no checksum. This contract defines no canonical-content hash.

## Identity and relationships

ResourceId is distinct from source location. Inlining or externalizing a
resource MUST preserve its ID, entity IDs and logical links. The descriptor ID
must match the resource header. IFCPR drawing links resolve by ResourceId,
independently of the source form of either resource.

Repeated declarations with the same resource ID, kind and external URI are
permitted. Distinct inline content locations are distinct sources, even when
their JSON bodies are equal. The same ID MUST NOT denote distinct inline
locations, both an inline and an external source, or different resource kinds.
Sharing inline drawing content is achieved by referring to its representation
node, not by copying its body into another descriptor with the same ID.

Different IDs declaring the same external file remain subject to header-ID
agreement. There is no content-equality deduplication or preferred source.
Invalid competing sources must not be resolved by choosing one alternative.

## Error origins

Report logical resource identity separately from physical origin. External
content diagnostics identify the external file and a resource-local JSON
Pointer. Inline content diagnostics identify the containing IFCX document and
the content pointer followed by the resource-local pointer. A root content
error identifies the content object. Descriptor errors identify descriptor
fields in the containing document.

For example, an inline drawing's `/header/version` error may appear at
`/data/3/attributes/resource/content/header/version` in `package.ifcx.json`.
An inline resource must not be assigned a fictitious external filename.

## Implementation support and limits

Inline storage does not imply that every IFCPR semantic feature is implemented.
Implementations must describe their validation and operation support separately;
accepting a source form does not establish full preservation validity.

The reference reader bounds physical files, including the IFCX entrypoint, and
total loaded bytes. Inline bytes count as part of that entrypoint exactly once.
Consequently, combined inline content shares the entrypoint's per-file limit.
An inline resource is not reserialized to impose a second byte-size limit.

A directory package without external resources can contain only
`package.ifcx.json`. Binary inline content and container packaging are outside
this contract. Historical schemas and frozen collections retain their contracts.
