# Size and exchange experiment data

The [complete experiment report](../../docs/benchmarks/size-baseline-v1.md)
contains the purpose, corpus, method, results, explanatory compression check,
limitations and reproduction instructions.

- [corpus-v1.json](corpus-v1.json): frozen recipes and fixture inventory.
- [results-v1.json](results-v1.json): the successful full-corpus run, both
  generations, per-file accounting/hashes, stage results and provenance.

The retained result uses the pinned unmodified cadcodec dependency and has
`baseline_accepted: true`. Earlier runs with
[local fixes](../../patches/cadcodec-upstream/README.md) remain historical.
Generated files, lockfiles and diagnostic development runs remain below
`target/`.

Standalone OCDraw reuses only the primitive recipes and has its own
[experiment](../../docs/benchmarks/ocdraw-size-exchange-v1.md). The historical
package report/results above remain accepted evidence for their old configuration.

- [ocdraw-results-v1.json](ocdraw-results-v1.json): accepted standalone primitive
  run after the polyline coordinate and paper-layout fixes, with ordered
  ownership and no stream directory, including both
  generations and matching source/dependency provenance.
