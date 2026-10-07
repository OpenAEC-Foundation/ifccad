# IFCCAD opaque spline pilot

The independent IFCCAD preservation slice stores complete interpreted spline
snapshots without native spline geometry. The format contract is described in
schemas/ifccad/experimental-contract-0.1.0.md; docs/preservation.md records the
architecture, predicates and target restrictions. Source snapshots remain
unchanged during ordinary native editing and after durable save/reopen.

Verification covers all supported owners, nested/unused definitions, optional
exact common state, source ownership and unknown-provider storage, exact uint64
IDs/counters, atomic composition, bounds incompleteness and clip restrictions.
Restoration tests recheck required source/context/reference conditions after
storage, including Paper5inch=127mm equivalence, meaning change/reversion and
forward typed XDATA in newly allocated CAD namespaces. Eight native mutation
cases pass actual DXF and AC1032 DWG readback against independent control-point
oracles. A create-new filesystem test closes/reopens and restores to fresh DWG.

The pinned fit-only parameterization1/2 is retained in typed storage/restoration
and DWG. Physical DXF readback still returns0 on063c106; tests require that known
loss and the inspector reports TARGET_CODEC_SPLINE_PARAMETERIZATION_LOSS. The
#99 codec repair, viewer codec adoption, native spline evaluator and #91 broad
private/shared storage are separate follow-ups. No controlled measurement ran.

Delivery verification: full Rust workspace772 passed,0 failed,1 existing ignored;
strict all-target Clippy, formatting and doc tests pass. Browser Node68 passed
with4 existing skips. The production release WASM and full smoke pass both
routes, text, geometry, clips, layout-output/Paper accuracy and the IFCCAD spline
capture/durable/exchange extension. A wrapper temp-directory denial was resolved
using the pre-existing matching wasm-bindgen0.2.129 against the compiled release
artifact; no codec pin/patch or source cache was changed.

Review was performed by the author in a separate pass; no reviewer agents were
used, following repository instructions. Complete source/envelope/condition
coverage and independent target oracles were reviewed. Implementation decisions:
- The execution remained local and uncommitted, with its ledger retained until
  separately authorized integration. Author review has less independence than
  review by another person.
- Inherited ByLayer/ByBlock source handles bind to the Drawing domain through
  qualified byLayer:<hex>/byBlock:<hex> keys, keeping ordinary pattern IDs honest.
  The namespace-risk cases are covered by positive and contradictory-handle tests.

This slice does not publish/freeze either provisional0.1.0 contract or authorize
IFCX source-graph writeback. Main and unrelated worktree changes remain separate.
