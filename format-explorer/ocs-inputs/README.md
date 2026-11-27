# Local XREF placement integration inputs

This slice-A development pair uses Studio5eb21f03, Documentb5925f27 and
native413c29e6 plus explicit local patches identified in ../ocs-source.json.
Those base commits alone do not implement profile13. No public deployment or
release is authorized; slice B will share the same unpublished13/13 draft.
The frozen lock and bundle record must match this actual pair before local
browser qualification. Native staging and official downstream pins follow review.

## Frozen web timer dependency

The official Studio source is built with the frozen `wasmtimer` 0.4.3 registry
package and its `pin-utils` dependency. The upstream 0.4.5 lock selection stops
both short intervals in a minimal browser-only continuity regression, without
a drawing or renderer. The identical 0.4.3 probe continues. This integration
therefore retains the previously working timer version explicitly in the frozen
lockfile; no Cargo-cache edits, local source override or host source patch is used.
The official Document, native contract and codec revisions are unchanged. The
new host/worker assets must be rebuilt and qualified against this exact lock.

The bundle record includes `frozenLockSha256`. Assembly checks it against the
selected frozen lock as well as the main pins and asset hashes, so an old bundle
with unchanged Document/Studio pins cannot conceal a different timer dependency.
