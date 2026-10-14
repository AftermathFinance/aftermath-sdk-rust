<!-- cargo-rdme start -->

Price update formats and supporting helpers for the Aftermath, Pyth, Pyth Lazer, and Stork
oracles.

This crate owns the data-format layer shared by price-update producers, relays, APIs, and
consumers. It provides:

- validated wrappers and parsers for binary price-update payloads;
- a single-allocation serializer for Aftermath updates that leaves signing to the caller;
- serializable price-update data structures where the upstream format is structured data;
- feed ID types and canonical feed ID sets;
- reusable sorted/deduplicated collections, process-lifetime interners, and their backing arena.

Each oracle has its own module:

- `aftermath` handles signed Aftermath updates accepted by the on-chain `af_verifier` package.
- `pyth` handles Pyth accumulator (`PNAU`) updates served by Hermes.
- `pyth_lazer` handles Pyth Lazer `leEcdsa` updates and binary WebSocket answers.
- `stork` models Stork signed price updates and their BCS representation.

# Validation and signing boundary

Constructors validate the structure of encoded updates. They do not authenticate the source or
verify embedded signatures; signed payloads are intended to be relayed to the corresponding
on-chain verifier. This crate also does not create signatures—signing belongs to the service
that owns the relevant key. `aftermath::UnsignedPriceUpdates::new` serializes an Aftermath
update into the caller's buffer and exposes the digest to sign; the caller appends a signature
made with any secp256k1 implementation. `keccak::keccak256` computes the same kind of digest
for other payloads, e.g. to verify Pyth Lazer signatures. Aftermath, Pyth, and Pyth Lazer
wrappers retain their original bytes and expose parsed feed views on demand.

# Feed ID sets and helpers

Oracle-specific `PriceFeedIdSet` types store trusted, bounded ID sets in process-lifetime
interners. Equal sets share one canonical slice, making the sets `Copy` with constant-time
equality and hashing. Dynamic input must first be sorted and deduplicated into a
`sorted_set::SortedSet`, then passed to the oracle-specific `PriceFeedIdSet::intern` method.
Fixed-size arrays also implement `From`, for bounded, program-controlled values such as array
literals.

`sorted_set::SortedSet` is the general owned, `Vec`-backed alternative,
`sorted_set::SortedSetSlice` is its zero-allocation borrowed view, and
`sorted_set::ArcSortedSet` provides cheap clones without process-lifetime storage. The
`slice_interner` and `str_interner` modules expose the lower-level canonicalization helpers,
backed by `leaking_arena`.

Interned values are never freed, so memory grows with every distinct value. Do not intern
untrusted input, such as feed ID sets requested by clients or arbitrary Stork asset IDs; keep
such input in a `sorted_set::SortedSet` or another reclaimable owner, or validate it first.
Intern Stork IDs explicitly with `stork::SmallAssetId::intern`. The `find` methods of the
interned types, such as `aftermath::PriceFeedIdSet::find`, look up existing values without
interning. `find`, `is_superset`, and `matches_set` take borrowed
`sorted_set::SortedSetSlice` references, obtained from a `SortedSet` or checked from a plain
slice with `sorted_set::SortedSetSlice::try_from_slice`.

```rust
use af_price_updates::{
    aftermath::{PriceFeedId, PriceFeedIdSet},
    sorted_set::SortedSet,
};

let ids = SortedSet::from(vec![PriceFeedId(7), PriceFeedId(3), PriceFeedId(7)]);
assert_eq!(ids.as_slice(), &[PriceFeedId(3), PriceFeedId(7)]);
let interned = PriceFeedIdSet::intern(&ids);
assert_eq!(interned.as_slice(), ids.as_slice());
```

<!-- cargo-rdme end -->
