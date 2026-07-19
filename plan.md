# BogosortCoin Implementation Plan

## 1. Project Overview

BogosortCoin is a proof-of-work cryptocurrency whose mining process uses a deterministic permutation generated from:

* The previous block hash.
* The candidate block header.
* A miner-selected nonce.
* A cryptographic pseudorandom generator.

A mining attempt succeeds when the generated permutation is sorted and the resulting proof ticket satisfies the current difficulty target.

The bogosort mechanism is a consensus rule and proof representation. The system's practical security still depends on standard cryptographic hashing, cumulative proof-of-work, strict consensus validation, secure networking, transaction signatures, and robust implementation practices.

---

## 2. Project Goals

### Primary goals

1. Define a deterministic and independently verifiable BogoPoW consensus algorithm.
2. Implement a secure transaction and block-validation engine.
3. Support decentralized peer-to-peer block and transaction propagation.
4. Provide wallets, node software, mining software, and developer tools.
5. Launch a public testnet before considering a mainnet release.
6. Require independent security review before production deployment.

### Non-goals for the first release

* Smart contracts.
* Anonymous transactions.
* Cross-chain bridges.
* Proof-of-stake.
* On-chain governance.
* Stablecoins or token issuance.
* Mobile wallets.
* Exchange integrations.

These features should not be considered until the base protocol is stable.

---

## 3. Recommended Technology Stack

### Core node

* Language: Rust.
* Async runtime: Tokio.
* Serialization: custom canonical binary encoding.
* Hashing: SHA3-256 and SHAKE256.
* Signatures: Ed25519 or Schnorr over secp256k1.
* Storage: RocksDB or an equivalent embedded key-value database.
* Networking: libp2p or a narrowly scoped custom protocol.
* Command-line interface: clap.
* Logging and tracing: tracing.

### Supporting tools

* Python for test-vector generation and research simulations.
* Docker for reproducible node environments.
* GitHub Actions or equivalent for continuous integration.
* Prometheus-compatible metrics.
* Property-based testing with proptest.
* Fuzzing with cargo-fuzz and libFuzzer.

The consensus-critical implementation should avoid unnecessary dependencies.

---

## 4. High-Level Architecture

The repository should use a workspace structure similar to:

```text
bogosortcoin/
├── Cargo.toml
├── crates/
│   ├── primitives/
│   ├── consensus/
│   ├── cryptography/
│   ├── serialization/
│   ├── storage/
│   ├── state/
│   ├── mempool/
│   ├── networking/
│   ├── node/
│   ├── miner/
│   ├── wallet/
│   └── rpc/
├── tools/
│   ├── test-vector-generator/
│   ├── chain-inspector/
│   └── benchmark/
├── specifications/
│   ├── consensus.md
│   ├── serialization.md
│   ├── networking.md
│   └── wallet.md
├── tests/
│   ├── consensus/
│   ├── integration/
│   └── adversarial/
└── deployments/
    ├── docker/
    ├── devnet/
    └── testnet/
```

### Component responsibilities

| Component       | Responsibility                                                          |
| --------------- | ----------------------------------------------------------------------- |
| `primitives`    | Blocks, transactions, hashes, amounts, addresses, and identifiers       |
| `serialization` | Canonical consensus encoding and decoding                               |
| `cryptography`  | Hash functions, signatures, key handling, and domain separation         |
| `consensus`     | BogoPoW, difficulty adjustment, chain-work calculation, and block rules |
| `state`         | UTXO or account-state transitions                                       |
| `storage`       | Blocks, indexes, state snapshots, and recovery                          |
| `mempool`       | Unconfirmed transaction validation and fee selection                    |
| `networking`    | Peer discovery, synchronization, propagation, and peer scoring          |
| `node`          | Runtime coordination                                                    |
| `miner`         | Candidate construction and proof search                                 |
| `wallet`        | Key generation, signing, balances, and transaction construction         |
| `rpc`           | External node and wallet interfaces                                     |

---

## 5. Consensus Specification

Before implementation begins, create a versioned consensus specification.

The specification must define every byte and every validation step. It must not rely on implementation-specific behavior.

### 5.1 Block header

The initial block header should contain:

```text
network_id
protocol_version
height
previous_block_hash
transaction_merkle_root
state_root
timestamp
difficulty_target
permutation_size
miner_commitment
extra_nonce
nonce
```

Each field must have:

* A fixed type.
* A fixed byte length or length-prefix rule.
* A defined byte order.
* A defined valid range.
* Explicit rejection rules.

### 5.2 Canonical encoding

Consensus data must use a custom canonical binary encoding.

Requirements:

* Integers use a specified byte order.
* No optional alternate encodings.
* No floating-point values.
* No ambiguous strings.
* Lengths are bounded before allocation.
* Unknown consensus fields cause rejection unless an upgrade explicitly permits them.
* Decoding followed by encoding must produce identical bytes.
* Non-canonical encodings must be rejected.

JSON, MessagePack, and general-purpose object serialization must not be used for consensus hashes.

### 5.3 Seed derivation

For each mining attempt:

```text
header_bytes = CanonicalEncode(candidate_header)

seed = SHA3-256(
    "BOGOPOW-v1/seed" ||
    header_bytes
)
```

The nonce is already included in the encoded header.

All domain strings must be fixed protocol constants.

### 5.4 Deterministic permutation generation

Create the initial permutation:

```text
[0, 1, 2, ..., N - 1]
```

Generate a deterministic random stream:

```text
stream = SHAKE256(
    "BOGOPOW-v1/permutation" ||
    seed
)
```

Apply Fisher-Yates from the final position to the first.

For every index, use rejection sampling when converting stream output into a bounded integer. Modulo-biased selection must not be permitted.

### 5.5 Bogo condition

The permutation portion succeeds only when:

```text
permutation[i] == i
```

for every valid index.

The probability of success is:

```text
1 / N!
```

### 5.6 Difficulty ticket

Derive a separate proof ticket:

```text
ticket = SHA3-256(
    "BOGOPOW-v1/ticket" ||
    seed
)
```

A block is valid when:

```text
permutation_is_sorted
AND
uint256_big_endian(ticket) <= difficulty_target
```

Domain separation prevents the permutation stream and ticket from being treated as interchangeable outputs.

### 5.7 Chain work

Nodes must select the valid chain with the greatest cumulative work.

Suggested work calculation:

```text
block_work =
    floor(
        N! × 2^256 /
        (difficulty_target + 1)
    )
```

The exact formula must be specified using bounded integer arithmetic and tested against overflow.

Block height alone must never determine the preferred chain.

### 5.8 Difficulty adjustment

The first testnet should use a conservative adjustment algorithm.

Requirements:

* Target block interval is fixed.
* Adjustments use a defined rolling window.
* Median timestamps are used where appropriate.
* Per-adjustment changes are bounded.
* Integer rounding behavior is specified.
* Minimum and maximum targets are specified.
* Permutation-size changes are rare and upgrade-controlled.

Difficulty should normally be adjusted through the hash target. Changing `N` produces large factorial jumps and should not be used for routine adjustment.

### 5.9 Timestamp rules

Define:

* Maximum accepted future offset.
* Median-past-time requirement.
* Time source assumptions.
* Behavior when local system time is incorrect.

Timestamp rules must not make consensus depend directly on a single node's local wall clock.

---

## 6. Transaction Model

A UTXO model is recommended for the first version because it simplifies independent transaction validation and parallel verification.

### Transaction fields

```text
version
inputs[]
outputs[]
lock_time
fee
```

Each input should reference:

```text
previous_transaction_id
output_index
unlocking_witness
sequence
```

Each output should contain:

```text
amount
locking_script_or_public_key
```

### Transaction rules

* Total input value must cover total output value and fees.
* Amount arithmetic must use checked integers.
* Duplicate inputs are invalid.
* Referenced outputs must exist and remain unspent.
* Signatures must commit to the intended transaction fields.
* Transaction identifiers must use canonical serialized bytes.
* Coinbase transactions follow separate maturity rules.
* Transactions must have strict size and input-count limits.

### Monetary units

Use a fixed smallest unit.

Example:

```text
1 BOGO = 100,000,000 bogoshis
```

Do not use floating-point arithmetic for currency values.

---

## 7. Monetary Policy

The initial policy must be simple and predictable.

Define:

* Maximum supply.
* Initial block subsidy.
* Subsidy reduction schedule.
* Minimum transaction fee policy.
* Coinbase maturity.
* Maximum block reward.
* Treatment of transaction fees.

Example policy:

```text
Initial subsidy: 50 BOGO
Halving interval: 1,050,000 blocks
Target interval: 60 seconds
Coinbase maturity: 100 blocks
```

These values are placeholders and require economic simulation before adoption.

Consensus must validate:

```text
coinbase_value <= block_subsidy(height) + total_block_fees
```

---

## 8. Development Phases

## Phase 0: Research and Threat Modeling

### Deliverables

* Formal BogoPoW specification.
* Canonical serialization specification.
* Threat model.
* Economic security analysis.
* Mining simulation.
* Initial network parameters.
* Decision record for UTXO versus account model.

### Research questions

* Can miners test the winning permutation more efficiently than expected?
* Does specialized hardware create unexpected centralization pressure?
* What permutation size provides useful proof granularity?
* How should work be represented when both `N` and the ticket target vary?
* How vulnerable is the proposed block interval to selfish mining?
* What hash-rate assumptions are required to resist reorganization attacks?
* What adjustment algorithm remains stable during rapid hash-rate changes?

### Exit criteria

* Consensus algorithm is deterministic on all supported platforms.
* Independent implementations generate identical test vectors.
* No unresolved ambiguity remains in proof verification.
* Economic simulations show stable adjustment under expected conditions.

---

## Phase 1: Consensus Prototype

### Tasks

1. Implement primitive fixed-width types.
2. Implement canonical encoding and decoding.
3. Implement SHA3 and SHAKE domain-separated helpers.
4. Implement deterministic Fisher-Yates.
5. Implement rejection sampling.
6. Implement sorted-permutation verification.
7. Implement target validation.
8. Implement chain-work calculation.
9. Implement block-header validation.
10. Publish test vectors.

### Required tests

* Known seed-to-permutation vectors.
* Known header-to-seed vectors.
* Known seed-to-ticket vectors.
* Boundary targets.
* Invalid encoding rejection.
* Maximum and minimum permutation sizes.
* Cross-platform test consistency.
* Integer-overflow tests.
* Malformed header fuzzing.

### Exit criteria

* Two independent implementations agree on all vectors.
* Consensus crate has no unsafe code unless separately audited.
* Fuzzing finds no crash or inconsistent acceptance behavior.
* Benchmarks establish verification cost.

---

## Phase 2: Transactions and State

### Tasks

1. Define transaction serialization.
2. Implement key generation and signatures.
3. Implement signature-hash rules.
4. Implement transaction identifiers.
5. Implement UTXO state.
6. Implement transaction validation.
7. Implement block-level state transitions.
8. Implement coinbase and fee accounting.
9. Implement undo data for chain reorganizations.
10. Implement state-root commitment.

### Required tests

* Valid and invalid signatures.
* Duplicate-spend detection.
* Intra-block spending rules.
* Coinbase maturity.
* Overflow and underflow cases.
* Reorganization rollback.
* State-root reproducibility.
* Transaction malleability checks.
* Large transaction rejection.
* Invalid witness encodings.

### Exit criteria

* State transitions are deterministic.
* Reorganizations restore the exact prior state.
* Randomized transaction sequences preserve supply invariants.
* No transaction can create value outside permitted rewards.

---

## Phase 3: Blockchain Storage

### Tasks

1. Define database schema.
2. Store block headers and full blocks separately.
3. Store chain-work indexes.
4. Store transaction and UTXO indexes.
5. Implement atomic block connection.
6. Implement atomic block disconnection.
7. Add write-ahead recovery.
8. Add database versioning and migrations.
9. Add pruning mode.
10. Add integrity checks.

### Security requirements

* A crash during block application must not corrupt consensus state.
* Database contents are treated as untrusted during startup.
* Indexes must be reconstructable from validated block data.
* Partial writes must be detected.
* Startup must verify the active-chain tip and state metadata.

### Exit criteria

* Forced termination tests recover correctly.
* Corrupted records are detected rather than trusted.
* Full reindex produces the same state root.
* Reorganization tests pass across process restarts.

---

## Phase 4: Mempool and Block Construction

### Tasks

1. Implement mempool admission rules.
2. Track transaction dependencies.
3. Reject conflicts and double spends.
4. Add fee-rate ordering.
5. Limit memory consumption.
6. Add transaction expiry and eviction.
7. Construct block templates.
8. Calculate Merkle and state roots.
9. Maintain miner extra-nonce fields.
10. Expose mining-template RPC methods.

### Abuse controls

* Maximum transaction size.
* Maximum ancestor and descendant counts.
* Minimum relay fee.
* Per-peer submission limits.
* Signature-verification budgeting.
* Duplicate-message suppression.
* Mempool memory cap.

### Exit criteria

* Invalid transactions never enter the mempool.
* Block templates contain only valid transactions.
* Template construction remains responsive under adversarial load.
* Fee accounting is exact.

---

## Phase 5: Miner

### Tasks

1. Request or construct a valid block template.
2. Divide nonce ranges across worker threads.
3. Support extra-nonce updates.
4. Generate the deterministic permutation.
5. Check the sorted condition.
6. Compute the ticket target.
7. Submit valid blocks.
8. Report hash-equivalent attempt rate.
9. Support remote mining through an authenticated protocol.
10. Add hardware benchmarking.

### Mining safety requirements

* No consensus reliance on the miner's local random-number generator.
* The nonce may be selected randomly, sequentially, or through partitioned ranges.
* Every proof must be reproducible solely from the block header.
* Mining jobs must identify the exact template.
* Stale templates must be canceled when a new tip is accepted.
* Pool protocols must bind shares to a specific job and miner.

### Exit criteria

* Multithreaded mining produces deterministic valid proofs.
* Submitted proofs are independently verified by the node.
* Nonce-range partitioning avoids accidental duplicate work.
* Stale work is promptly abandoned.

---

## Phase 6: Peer-to-Peer Network

### Protocol messages

Initial messages may include:

```text
hello
version
ping
pong
get_headers
headers
get_blocks
block
transaction
get_peers
peers
reject
```

### Tasks

1. Implement encrypted or authenticated transport where practical.
2. Implement protocol-version negotiation.
3. Implement peer discovery.
4. Implement header-first synchronization.
5. Implement block download scheduling.
6. Implement transaction and block propagation.
7. Implement peer scoring.
8. Implement temporary bans.
9. Limit inbound and outbound resource use.
10. Add network-specific magic values.

### Security controls

* Message length limits.
* Decode-before-allocation limits.
* Connection-rate limits.
* Per-peer bandwidth budgets.
* Request timeouts.
* Duplicate-request suppression.
* Orphan-block limits.
* Orphan-transaction limits.
* Peer diversity requirements.
* Protection against address poisoning.
* Protection against eclipse attacks.
* Invalid-data penalties.
* Separate validation and networking queues.

### Exit criteria

* Nodes synchronize from genesis.
* Nodes recover from interrupted synchronization.
* Malicious peers cannot cause unbounded memory allocation.
* A single peer cannot monopolize block download.
* Test nodes converge after forks and partitions.

---

## Phase 7: Wallet and Key Management

### Tasks

1. Generate cryptographically secure wallet seeds.
2. Derive addresses deterministically.
3. Encrypt private-key storage.
4. Build and sign transactions.
5. Track wallet-owned outputs.
6. Estimate fees.
7. Support backups and recovery.
8. Add watch-only wallets.
9. Add transaction history.
10. Add address validation.

### Security requirements

* Never derive wallet keys from BogoPoW randomness.
* Never log private keys or seed phrases.
* Clear sensitive buffers where practical.
* Require authenticated RPC access.
* Use operating-system random generation.
* Protect wallet files with memory-hard password derivation.
* Provide explicit backup confirmation.
* Separate wallet and node processes where possible.

### Exit criteria

* Seed recovery reproduces addresses and funds.
* Corrupted wallet files fail safely.
* Incorrect passwords reveal no useful key information.
* Signed transactions are compatible with independent verification tools.

---

## Phase 8: RPC, CLI, and Operations

### RPC areas

* Node status.
* Peer management.
* Blockchain queries.
* Transaction submission.
* Block submission.
* Mining templates.
* Wallet operations.
* Metrics and diagnostics.

### Security requirements

* RPC is disabled externally by default.
* Authentication is mandatory.
* TLS is required for remote access.
* Sensitive methods can be disabled.
* Request sizes and execution time are bounded.
* Wallet and administrative methods are separated.
* RPC responses do not expose private information unnecessarily.

### Operational features

* Structured logs.
* Configurable log levels.
* Prometheus metrics.
* Health endpoints.
* Graceful shutdown.
* Disk-space monitoring.
* Database integrity commands.
* Network and chain identification in every status display.

---

## Phase 9: Devnet

Create a private developer network before public testnet.

### Devnet characteristics

* Low difficulty.
* Fast block interval.
* Faucet.
* Frequent chain resets.
* Instrumented nodes.
* Controlled adversarial peers.
* Automated deployment.
* Deterministic genesis generation.

### Devnet scenarios

* Normal synchronization.
* Competing miners.
* Deep reorganizations.
* Network partitions.
* Sudden hash-rate changes.
* Invalid block floods.
* Invalid transaction floods.
* Database crashes.
* Clock skew.
* Peer eclipse simulations.
* Software upgrades.

### Exit criteria

* Devnet runs continuously for at least four weeks.
* No consensus split occurs among supported platforms.
* Recovery procedures are documented and tested.
* Critical metrics and alerts are operational.

---

## Phase 10: Public Testnet

### Testnet requirements

* Separate genesis block.
* Separate network identifier.
* Separate address prefix.
* No conversion path from test coins to mainnet coins.
* Public seed nodes.
* Block explorer.
* Faucet with abuse controls.
* Published binaries and reproducible build instructions.
* Bug-reporting and responsible-disclosure process.

### Testnet goals

* Observe real mining behavior.
* Measure mining centralization.
* Validate difficulty adjustment.
* Test long-running synchronization.
* Test wallet recovery.
* Exercise software upgrades.
* Test malicious and outdated peers.
* Identify economic and denial-of-service weaknesses.

### Minimum testnet period

A production candidate should operate on public testnet for at least six months after the last consensus-breaking change.

Any major consensus change restarts the stability period.

---

## 9. Security Program

## 9.1 Threat model

The threat model must include:

* Malicious miners.
* Majority and near-majority hash power.
* Selfish mining.
* Block withholding.
* Long-range reorganizations.
* Transaction malleability.
* Double spending.
* Eclipse attacks.
* Sybil attacks.
* Peer flooding.
* Malformed message attacks.
* Database corruption.
* Supply inflation bugs.
* Signature implementation flaws.
* Dependency compromise.
* Build-system compromise.
* Wallet theft.
* RPC exposure.
* Time manipulation.
* Difficulty manipulation.
* Consensus implementation divergence.

## 9.2 Secure development requirements

* Mandatory code review.
* Protected main branch.
* Signed releases.
* Reproducible builds.
* Dependency locking.
* Software-bill-of-materials generation.
* Static analysis.
* Continuous fuzzing.
* Property-based testing.
* Secret scanning.
* Minimal unsafe code.
* Documented consensus changes.
* No silent consensus-rule modifications.

## 9.3 Independent review

Before mainnet:

1. Cryptographic review.
2. Consensus and economic review.
3. Network-protocol review.
4. Wallet and key-management review.
5. Source-code audit.
6. Reproducible-build verification.
7. Adversarial testnet exercise.

All critical and high-severity findings must be resolved and retested.

## 9.4 Bug bounty

Launch a public bug bounty during testnet.

Suggested categories:

* Consensus split.
* Unauthorized coin creation.
* Remote code execution.
* Private-key disclosure.
* Network-wide denial of service.
* Persistent eclipse attack.
* Node database corruption.
* Wallet fund loss.
* Difficulty bypass.
* Proof-verification inconsistency.

---

## 10. Testing Strategy

## 10.1 Unit tests

Cover:

* Serialization.
* Hash domains.
* Permutation generation.
* Rejection sampling.
* Proof validation.
* Transactions.
* Signatures.
* Merkle roots.
* State roots.
* Difficulty calculations.
* Chain-work calculations.
* Monetary policy.

## 10.2 Property-based tests

Key invariants:

```text
decode(encode(x)) == x
encode(decode(bytes)) == canonical_bytes
total_supply_after <= permitted_supply
connect(block); disconnect(block) restores original state
accepted proof verifies identically on all platforms
preferred chain has maximum cumulative valid work
```

## 10.3 Fuzz testing

Fuzz targets:

* Block decoder.
* Transaction decoder.
* Network message decoder.
* Script or witness parser.
* Block validation.
* Transaction validation.
* Database recovery metadata.
* RPC input parsing.
* Peer-handshake state machine.

## 10.4 Differential testing

Build at least two independent consensus implementations or one implementation plus an independent reference verifier.

Compare:

* Header hashes.
* Permutations.
* Proof tickets.
* Validity decisions.
* State roots.
* Chain-work totals.
* Difficulty adjustments.
* Transaction signature hashes.

## 10.5 Integration testing

Test complete workflows:

* Mine genesis successor.
* Send and confirm transactions.
* Restart during block connection.
* Reorganize competing chains.
* Synchronize a new node.
* Restore a wallet.
* Upgrade node versions.
* Reject invalid historical blocks.
* Recover from temporary network partitions.

## 10.6 Performance testing

Measure:

* Proof verification time.
* Mining attempts per second.
* Signature verifications per second.
* Block-validation time.
* Initial synchronization time.
* Reorganization time.
* Database growth.
* Memory under transaction flood.
* Bandwidth during synchronization.
* RPC saturation behavior.

---

## 11. Consensus Upgrade Strategy

Consensus upgrades must be explicit and versioned.

### Requirements

* Every upgrade has a specification.
* Activation rules are deterministic.
* Nodes can determine activation from chain data.
* Unknown mandatory rules cause safe rejection.
* Emergency changes are not pushed silently.
* Old and new validation behavior is covered by test vectors.
* Activation is tested on devnet and testnet first.

A version-bits-style miner signaling system may be considered, but activation should not depend solely on miner signaling. Node and ecosystem readiness must also be considered.

---

## 12. Genesis Block Plan

The genesis block must define:

* Network identifier.
* Initial timestamp.
* Initial difficulty.
* Initial permutation size.
* Initial monetary parameters.
* Initial protocol version.
* Human-readable launch message.
* Genesis public key or unspendable reward policy.
* Initial state root.
* Exact canonical bytes.
* Expected genesis hash.

Generate genesis data with a dedicated deterministic tool.

Publish:

* Input parameters.
* Generated header.
* Full serialized block.
* Block hash.
* Seed.
* Permutation.
* Ticket.
* Verification transcript.

---

## 13. Mainnet Launch Gates

Mainnet must not launch until every gate passes.

### Consensus gates

* Consensus specification is complete.
* Independent implementations agree.
* Difficulty adjustment is stable.
* Cumulative-work selection is tested.
* No unresolved supply or validation ambiguity exists.

### Security gates

* Independent audits are complete.
* All critical and high findings are resolved.
* Continuous fuzzing has run without unresolved crashes.
* Reproducible builds are confirmed.
* Release signing procedures are tested.
* Responsible disclosure is active.

### Network gates

* Public testnet has run for at least six stable months.
* Multiple independent node operators exist.
* Multiple mining implementations or operators exist.
* Eclipse and denial-of-service defenses have been tested.
* Seed-node failure does not stop discovery.

### Operational gates

* Explorer is operational.
* Monitoring and incident procedures are documented.
* Upgrade and rollback procedures are tested.
* Wallet backup and recovery documentation is complete.
* Genesis and release binaries are publicly reproducible.

### Governance gates

* Maintainer responsibilities are documented.
* Release approval rules are documented.
* Emergency communication channels exist.
* Consensus-change procedures are public.
* Trademark, licensing, and legal responsibilities are reviewed.

---

## 14. Proposed Milestones

| Milestone | Deliverable                                              |
| --------- | -------------------------------------------------------- |
| M0        | Threat model and formal BogoPoW specification            |
| M1        | Consensus library and public test vectors                |
| M2        | Transaction, UTXO, and state-transition engine           |
| M3        | Persistent blockchain storage and reorganization support |
| M4        | Miner and block-template interface                       |
| M5        | Peer-to-peer synchronization                             |
| M6        | Wallet, CLI, and secured RPC                             |
| M7        | Automated private devnet                                 |
| M8        | Public testnet launch                                    |
| M9        | First independent security audit                         |
| M10       | Consensus freeze candidate                               |
| M11       | Final audit and release candidate                        |
| M12       | Mainnet readiness decision                               |

---

## 15. Initial Work Breakdown

### Epic A: Specification

* A-001: Define primitive types.
* A-002: Define block-header fields.
* A-003: Define canonical encoding.
* A-004: Define BogoPoW seed derivation.
* A-005: Define permutation algorithm.
* A-006: Define rejection sampling.
* A-007: Define target validation.
* A-008: Define chain-work formula.
* A-009: Define difficulty adjustment.
* A-010: Publish consensus test vectors.

### Epic B: Consensus implementation

* B-001: Implement fixed-width hash types.
* B-002: Implement canonical codec.
* B-003: Implement hash domain helpers.
* B-004: Implement SHAKE stream reader.
* B-005: Implement Fisher-Yates.
* B-006: Implement proof verification.
* B-007: Implement work calculation.
* B-008: Implement header validation.
* B-009: Add fuzz targets.
* B-010: Add reference verifier.

### Epic C: Ledger

* C-001: Define transactions.
* C-002: Implement signatures.
* C-003: Implement UTXO database.
* C-004: Implement block rewards.
* C-005: Implement fee accounting.
* C-006: Implement state roots.
* C-007: Implement connect and disconnect.
* C-008: Add supply-invariant tests.
* C-009: Add reorganization tests.
* C-010: Add wallet transaction builder.

### Epic D: Node and networking

* D-001: Implement peer handshake.
* D-002: Implement header synchronization.
* D-003: Implement block synchronization.
* D-004: Implement propagation.
* D-005: Implement peer scoring.
* D-006: Add resource limits.
* D-007: Add seed discovery.
* D-008: Add network metrics.
* D-009: Add adversarial peer tests.
* D-010: Add protocol documentation.

### Epic E: Mining

* E-001: Implement candidate templates.
* E-002: Implement nonce scanning.
* E-003: Implement multithreading.
* E-004: Implement extra-nonce handling.
* E-005: Implement stale-work cancellation.
* E-006: Implement block submission.
* E-007: Add mining benchmarks.
* E-008: Define pool protocol.
* E-009: Implement share validation.
* E-010: Test distributed mining.

### Epic F: Release engineering

* F-001: Create reproducible builds.
* F-002: Add signed release artifacts.
* F-003: Create Docker deployments.
* F-004: Create devnet automation.
* F-005: Add continuous fuzzing.
* F-006: Generate software bill of materials.
* F-007: Document incident response.
* F-008: Launch bug bounty.
* F-009: Commission external audits.
* F-010: Complete mainnet checklist.

---

## 16. Key Technical Risks

### Risk: Bogosort optimization

Miners may derive shortcuts, batch proofs, or design specialized hardware that tests the sorted-permutation condition more efficiently than a direct implementation.

Mitigation:

* Treat the algorithm as hash-based proof-of-work.
* Benchmark optimized implementations.
* Avoid claims that literal sorting effort provides security.
* Model specialized-hardware advantages before launch.

### Risk: Coarse factorial difficulty

Changing permutation size causes very large difficulty jumps.

Mitigation:

* Keep permutation size stable.
* Use the ticket target for routine adjustment.
* Change permutation size only through a protocol upgrade.

### Risk: Consensus divergence

Small serialization or arithmetic differences can split the chain.

Mitigation:

* Use canonical encoding.
* Specify all integer behavior.
* Publish extensive test vectors.
* Maintain an independent verifier.
* Use differential testing.

### Risk: Low initial mining participation

A new chain with little proof-of-work can be reorganized cheaply.

Mitigation:

* Do not imply economic security before sufficient participation exists.
* Use testnet first.
* Monitor mining concentration.
* Consider conservative confirmation recommendations.
* Delay valuable ecosystem integrations.

### Risk: Denial of service

Proof verification may be inexpensive, but transactions, signatures, synchronization, and orphan handling can still exhaust resources.

Mitigation:

* Apply strict limits.
* Prioritize validation queues.
* Rate-limit peers.
* Bound all allocations.
* Fuzz all network parsers.

### Risk: Wallet compromise

Consensus security does not protect poorly stored private keys.

Mitigation:

* Separate wallet and node responsibilities.
* Encrypt wallet storage.
* Support hardware signing later.
* Provide safe backup and recovery workflows.

---

## 17. Definition of Done

BogosortCoin is not considered production-ready merely because it can mine blocks and transfer coins.

The project is complete for a mainnet-readiness review only when:

1. The protocol has a complete, versioned specification.
2. Independent implementations agree on consensus.
3. Public testnet has demonstrated long-term stability.
4. Security and economic reviews are complete.
5. Critical vulnerabilities are resolved.
6. Builds are reproducible.
7. Wallet recovery has been independently tested.
8. Network-abuse protections have been stress-tested.
9. Mainnet parameters are justified by simulation and testnet data.
10. Maintainers are prepared to operate and respond to incidents.

The final launch decision should be based on evidence from audits, testnet behavior, adversarial testing, mining distribution, and operational readiness—not solely on implementation completion.
