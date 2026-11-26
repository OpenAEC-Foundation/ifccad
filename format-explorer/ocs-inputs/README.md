# Official XREF integration inputs

Explorer embeds official Studio merge `0ca42af051bfdf808f8824e15132c8504dbab331`
(PR #3651), selecting official Document merge `1db213a7babde93b64a4887fd6ba441f0177b00d`
(PR #32), canonical acadrust `b0b768cee674c28bac63c75cbd5638b894f33f23` and staged
native contract `785590abb669c38afb700b339d36cc8cc0c18161`. Exact identities are
recorded in ../ocs-source.json. No local host patch or development override is
required. The frozen lock starts from the official Studio merge lock and applies
the explicit web timer correction described below. The Explorer workspace uses
its intentional local core patch to share one typed model with Document; its
contract matches the host.

Run scripts/prepare_explorer_ocs.py against the exact official checkout before
building. Record and verify the actual host/worker bundle; previous development
bundles are historical evidence and are not relabeled as this candidate.

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
