# Inline and external drawing and preservation resources

Date: 2026-09-09

Status: Approved, implemented and verified on `inline-resource-sources`.

## Purpose and scope

Complete milestone 2's common resource-source abstraction. Both external and
inline IFCDR JSON must reach the same decoded logical model and semantic
validation. IFCPR must accept both sources with its existing limited validation.
Resource identity and relationships must survive changing the storage form.

This follows issue #1 (explicit identity and inline IFCDR content). Explicit
ResourceId identity has already landed; this work does not reopen it. Issues
#3 and #6 concern later physical encoding and measurements. No binary payload,
container, automatic size threshold, new entity or preservation interpretation
is introduced. ROADMAP.md remains authoritative; milestone 2 remains Current.

## Agreed source contract

Drawing descriptors remain at `attributes.resource`; preservation descriptors
remain at `attributes.preservation`. Each descriptor has exactly one source:

- External: `uri` and `checksum` are required; `content` is forbidden.
- Inline: `content` is required and is the complete resource JSON object;
  `uri` and `checksum` are forbidden.

Neither source, both sources, null content and non-object content are invalid.
Inline content is an object, not escaped JSON text or base64. It retains the
resource header and all existing logical fields. Descriptor format, version,
resourceId and kind-specific metadata remain required in both forms.

For IFCDR the role remains `drawing`. For IFCPR, sourceDocumentId and
linkedDrawingResourceIds retain their current descriptor requirements. Header
resource identity must agree with descriptor identity in both source forms.

External checksums retain SHA-256 over the exact loaded file bytes. Inline
content has no checksum; it still receives all applicable content checks.
This change introduces no canonical-content hash or fingerprint contract.

## Schema versions and compatibility

Introduce IFCX overlay 0.9.0 and normative resource-source contract 0.9.0.
Retain drawing core 0.2.0, IFCDR logical registry and JSON mapping 0.7.0,
and IFCPR schema 0.2.0: resource bodies have not changed.

The new overlay retains the drawing relationship rules from the normative
drawing-resource contract 0.8.0. Its source-choice rules supersede that
document's selection of overlay 0.8.0. Reference both normative documents
explicitly so implementers need not infer cross-node rules from Rust.

Use mutually exclusive schema branches for the source forms. Validate inline
bodies through the same resource validators as external bodies; the overlay
only requires an object and validates descriptor fields. This preserves the
same resource-level diagnostics for either location.

Existing external 0.8.0 packages satisfy the new source contract without body
changes. Retired representation vocabulary remains explicitly unsupported;
unrelated IFCX extension points remain open. Update only active assets and
conformance/next (suite 1.1.0); historical schemas and numbered collections
remain unchanged. Keep the referenced 0.8.0 normative relationship document
in the candidate alongside the new 0.9.0 overlay and source contract.

## Reader architecture

Separate logical ResourceId, physical source, and diagnostic origin.
Discovery produces an explicit external-or-inline source value rather than
requiring an external URI. Inline origin identifies the containing IFCX file
and the JSON Pointer to the content object. Do not invent filenames or URIs
to make inline content fit an external-file structure.

Load external JSON through the existing bounded filesystem loader. Obtain
inline JSON from the parsed entrypoint. Both then enter the same resource-kind
validation path, including identity checks and package bindings. Avoid
serializing and reparsing inline content merely to imitate a file. Raw bytes
are needed for external checksum validation, not for semantic access.

Internal caches may use source identities. Public resource lookup and links
continue to use ResourceId. DrawingRepresentationRef.resource() remains
unchanged; external_uri() returns None for inline content. Logical entity
APIs and the shared IFCDR access trait expose no storage distinction.

The existing directory loader remains the public entry point. A package
containing only inline resources can consist solely of package.ifcx.json.
A separate arbitrary-filename file-opening API is outside this step.

## Identity and repeated declarations

Changing a resource's source form preserves its resourceId, entity IDs,
representation path and all logical links. An inline resource shared by
layouts is represented once; those layouts reference the representation node.

Preserve acceptance of repeated declarations of the same resource ID, kind
and external URI. Distinct inline content locations are distinct sources:
reusing one ID in two such locations is a duplicate identity error, even if
the JSON bodies happen to be equal. Reusing an ID across inline and external
sources or across resource kinds is also invalid. This avoids adding content
equivalence or precedence rules to identity validation.

Retain existing handling of different IDs declaring the same external file;
resource header identity checks remain decisive. Invalid source descriptors
must not cause invented missing-file errors or filesystem reads of a discarded
alternative source.

## IFCPR behavior

Run existing IFCPR schema, identity and drawing-resource-link checks on inline
and external bodies. External bodies additionally receive checksum checks.
Drawing targets resolve by ResourceId regardless of either resource's source.
Update diagnostic deduplication that currently matches descriptors by URI to
use their actual declaration/source identity.

No new blob, record, dependency or projection interpretation is added. The
existing deferred IFCPR conformance cases remain deferred. A strict package
result still proves only the implemented IFCPR checks. The converter neither
produces nor restores preservation content in this step.

## Diagnostics and loading limits

Keep resourceId available in diagnostics. For an external body error,
resource_uri names that external file and location is relative to its JSON.
For an inline body error, resource_uri names package.ifcx.json and location
is the content object's pointer followed by the resource-local error pointer.
An error on the entire inline body points to the content object itself.
Descriptor errors continue to point to descriptor fields in the entrypoint.

For example, a local error at /header/resourceId becomes
/data/3/attributes/resource/content/header/resourceId for that inline drawing.
Apply this translation once at the package boundary, including IFCPR errors;
validators remain independent of source form. Report ordering stays deterministic.

Keep existing entrypoint, per-file and total-byte limits. Inline bytes are
already part of the entrypoint and count once toward loading limits; do not
count them again via synthetic serialization. This means the entrypoint's
per-file limit bounds its combined inline contents. Document this distinction
from several independently bounded external files. No automatic externalizing
or new configurable logical-size limit is introduced.

## Writer

Add public DrawingResourceStorage with External and Inline variants; External
is its default. Expose a setter on DrawingBuilder named
set_resource_storage(storage: DrawingResourceStorage), preserving existing
DrawingOptions struct literals and the default output of existing callers.
Store the selection in package construction state, outside the logical IFCDR
model. The setting applies to that drawing resource; the writer's existing
single-drawing restriction is unchanged.

Both forms use the same validated resource and JSON mapping. External output
retains resources/drawing.ifcdr.json and its checksum descriptor. Inline output
embeds that JSON object as content and emits no separate IFCDR file or checksum.
PackageBuilder.finish() continues to validate the assembled output in memory
through the production reader before returning EncodedPackage.

EncodedPackage.files(), file() and write_directory() remain usable for both
forms. Inline output without other resources consists of the entrypoint file.
Repeated writing with the same options is deterministic within each mode;
byte equality across modes is not expected. IFCPR production is not added.

## Required evidence and documentation

- Schema cases cover both valid forms for both resource kinds; reject absent
  or simultaneous sources, invalid content type, missing external checksum
  and any inline checksum, including null.
- Production-reader fixtures cover external/external, inline/inline and both
  mixed IFCDR/IFCPR combinations with working drawing-resource links.
- External and inline versions of the same IFCDR resource expose equal units,
  bounds, IDs, scopes, ordered entities, geometry and appearance semantics.
- Existing drawing/layout representation consistency holds for inline sources.
- Both source forms exercise malformed bodies, identity mismatches, missing
  drawing targets and unsupported IFCDR content with accurate error origins.
- Duplicate source/identity cases follow the rules above; no dependent
  missing-source diagnostics are fabricated after invalid source syntax.
- Writer tests check exact file membership, per-mode determinism and semantic
  roundtrips through the production reader and existing converter.
- Loading-limit tests confirm inline entrypoint bytes are counted once.
- Update API documentation, README, converter guidance, compatibility matrix,
  candidate provenance and ROADMAP progress. Keep IFCPR limitations explicit.

Before reporting implementation complete, run cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace,
and focused cargo test --doc --workspace for public API documentation.

Implementation uses branch inline-resource-sources. Commit, merge and push require separate
user authorization; work remains in this task without subagents.
