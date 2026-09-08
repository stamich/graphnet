# GraphNet Milestone 0.1

Single-node, in-memory foundation of GraphNet implemented in Rust.

## Scope

- strongly typed `EntityId`, `NodeId` and `MutationId`
- immutable `StateNode` versions
- explicit `StateRef` read dependencies
- atomic multi-entity `Mutation`
- optimistic concurrency through `expected_version`
- `Commit` and `StateTransition`
- current materialized state
- complete per-entity history
- committed mutation/dependency lookup
- integration tests
- small CLI demo

## Intentionally not implemented yet

- persistence / WAL
- networking
- consensus
- conflict domains
- asynchronous execution
- cryptographic identities and proofs
- Scala, Java or Python layers

## Build and test

```bash
cargo test --workspace
```

## Run demo

```bash
cargo run -p graphnet-demo
```

## Generate Rustdoc

```bash
cargo doc --workspace --no-deps --open
```
