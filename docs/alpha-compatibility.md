# IFCCAD alpha compatibility policy

Agreed on 26 October 2026.

IFCCAD remains an alpha drawing profile. Breaking changes are permitted while
its semantics, APIs and encodings develop. Integration into Open CAD Studio,
the Explorer or another publicly available application does not establish a
backward-compatibility guarantee for IFCCAD files.

A later release may be unable to open an earlier alpha file, including one
saved only a few weeks before. There is no commitment to maintain readers for
all earlier alpha versions or to provide automatic or manual migration tools.
This applies to the IFCCAD JSON/IFCX profile and its standalone CBOR and
CBOR + Zstd representations; it does not define compatibility for IFCX generally.

Users should keep their original drawings when evaluating alpha conversion.
An alpha IFCCAD file should not be the only retained copy of a drawing that
must remain accessible over time. Public documentation and release notes must
state the alpha status and absence of a file-compatibility guarantee clearly.

## Releases and development drafts

Published release artifacts and tags remain fixed, so their behavior and format
contracts can be identified and reproduced. Release notes must identify known
breaking changes and the profile/encoding versions supported by that release.
Compatibility may be provided for a particular version without creating a
general promise to support all historical files.

The active profile and CBOR mapping remain mutable, unreleased drafts until
their first publication. The [release and file-version policy](releases.md)
defines identification and subsequent increments. Preparing a candidate does
not itself freeze or publish these drafts. A software version is not a
statement of file compatibility.

Reassess this policy as coverage and the format contract mature. A stronger
compatibility or migration commitment requires an explicit new decision and
documented scope; it is not implied by adoption or elapsed time.
