# BogoCoin

A proof-of-work scheme where a mining attempt succeeds only when a
deterministic, hash-derived permutation of `[0, 1, ..., N-1]` comes out
sorted — combined with a standard hash-target check for fine-grained
difficulty. See [plan.md](plan.md) for the full protocol design and
multi-phase roadmap (transactions, storage, networking, wallets, testnet,
mainnet gates, etc).

**What's implemented here is Phase 1 only: the BogoPoW consensus core**, plus
a CLI miner and a web visualizer. Everything else in the plan (P2P
networking, UTXO state, storage, RPC, wallets) is future work.

## How mining works

For each candidate block header:

1. `seed = SHA3-256("BOGOPOW-v1/seed" || canonical_header_bytes)`
2. A deterministic permutation of `[0..N)` is generated from
   `SHAKE256("BOGOPOW-v1/permutation" || seed)` via Fisher-Yates, using
   rejection sampling so there's no modulo bias.
3. `ticket = SHA3-256("BOGOPOW-v1/ticket" || seed)`
4. The block is valid iff the permutation is sorted (`permutation[i] == i`
   for all `i`, probability `1/N!`) **and** `ticket <= difficulty_target`
   (interpreted as a big-endian 256-bit integer).

Only the nonce (part of the header) is free for a miner to vary — seed and
permutation are fully determined by it, so there is no shortcut around
brute-forcing nonces and re-hashing. See the "is this quantum secure?" /
"can you reverse the seed?" discussion in project notes for why this holds.

## Project layout

```text
bogocoin/
├── crates/
│   ├── primitives/   fixed-width types, canonical big-endian encode/decode
│   ├── consensus/     BlockHeader, seed/permutation/ticket derivation, chain-work
│   └── miner/          CLI miner binary (bogocoin-miner)
├── web/               browser visualizer for a live mining run
└── plan.md            full project plan and roadmap (all phases)
```

## Building and testing

Requires Rust (stable, via [rustup](https://rustup.rs)).

```sh
cargo build --release
cargo test
```

17 unit tests cover canonical-encoding roundtrips, seed/permutation
determinism, permutation validity, ticket/target comparison, chain-work
monotonicity, and an end-to-end small-N mining convergence test.

## CLI miner

```sh
cargo run --release -p bogocoin-miner -- --help
```

Key options:

| Flag | Default | Meaning |
|---|---|---|
| `--permutation-size` | `8` | `N` — mining succeeds with probability `1/N!` per attempt |
| `--target` | `0f` × 32 (easy) | big-endian hex difficulty target; smaller = harder |
| `--previous-hash`, `--merkle-root`, `--state-root`, `--miner-commitment` | zero | header fields, hex-encoded |
| `--start-nonce` | `0` | nonce to begin scanning from |
| `--stream-file` | none | write a live JSON snapshot (~30/s) for the web visualizer |

Example — an easy demo run that finishes in well under a second:

```sh
cargo run --release -p bogocoin-miner
```

A harder one (tighter target, more leading zero bits):

```sh
cargo run --release -p bogocoin-miner -- \
  --permutation-size 8 \
  --target 3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f
```

Increasing `--permutation-size` grows expected attempts roughly by a factor
of `N` each step (`1/N!` odds) — N ≥ 12 or so will take a long time.

## Web visualizer

Watch a mining run live: a tile/bar view of the current permutation
(green where a value lands on its own index), attempt/rate/elapsed
counters, and the seed/ticket/target hex with pass/fail indicators.

```sh
python3 web/run.py --permutation-size 10
```

This builds the release miner if needed, launches it with `--stream-file`,
serves `web/` locally, and opens your browser. Options: `--target <hex>`,
`--port`, `--no-browser`.

## Status

Consensus prototype only (plan.md Phase 1). Not production software, not
consensus-final, and not audited — see plan.md §13 for the full mainnet
launch gate checklist this project would need to clear first.
