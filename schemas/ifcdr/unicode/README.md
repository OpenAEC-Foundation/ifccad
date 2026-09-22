# Pinned name comparison data

The IFCDR 0.9.0 candidate uses Unicode 17.0.0 full default case folding for
block and layer name uniqueness. Keep the original string in storage. Compare
the C/F case-folded strings without trimming, normalization or Turkic tailoring.
Unit tokens are not names and remain case-sensitive.

Source: [CaseFolding-17.0.0.txt](https://www.unicode.org/Public/17.0.0/ucd/CaseFolding.txt).
The vendored file retains its original copyright notice and has SHA-256
`ff8d8fefbf123574205085d6714c36149eb946d717a0c585c27f0f4ef58c4183`.
[LICENSE.txt](LICENSE.txt) contains Unicode License V3, retrieved with the data.

Regenerate from the repository root using Python 3.10+:

```text
python scripts/generate_casefold.py
python scripts/generate_casefold.py --check
cargo test -p ifccad --lib ifcdr::names
```

The generator selects only C/F rows, rejects duplicate source scalars, sorts by
scalar value and emits exact Rust Unicode escapes. Other scalars pass through
unchanged. The Rust tests compare every vendored C/F row with the comparison
function, as well as explicit name-collision and non-collision examples.
Python and its own Unicode version are not runtime dependencies or the source
of the folding behavior. Updating this data requires an explicit contract change.
