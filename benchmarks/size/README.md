# Size and exchange experiment data

The [complete experiment report](../../docs/benchmarks/size-baseline-v1.md)
contains the purpose, corpus, method, results, explanatory compression check,
limitations and reproduction instructions.

- [corpus-v1.json](corpus-v1.json): frozen recipes and fixture inventory.
- [results-v1.json](results-v1.json): the successful full-corpus run, both
  generations, per-file accounting/hashes, stage results and provenance.

The reported DWG writer includes the two
[local fixes](../../patches/cadcodec-upstream/README.md). The result retains
`baseline_accepted: false`, preserving the automatic gate for local overrides.
The documented patched run is accepted as milestone 2's scoped initial
measurement reference. A passing unpatched upstream baseline remains separate
codec maintenance. Generated files, lockfiles and diagnostic development runs
remain below `target/`.
