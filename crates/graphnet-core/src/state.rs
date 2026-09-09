//! Immutable state versions, payload values and references between GraphNet nodes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::id::{EntityId, MutationId, NodeId};

/// Monotonically increasing logical version of an entity state.
///
/// Milestone 0.1 uses a per-entity integer version. Later milestones may enrich
/// this model with causal clocks or content-addressed version identifiers.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Version(u64);

impl Version {
    /// Initial version assigned when an entity is first inserted into the graph.
    pub const INITIAL: Self = Self(1);

    /// Creates a version from a raw integer value.
    ///
    /// This is primarily useful for tests, deserialization boundaries and future
    /// persistence adapters. Application code should usually use [`Version::INITIAL`]
    /// and [`Version::next`].
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the next sequential version.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Returns the underlying integer value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Generic scalar value stored inside an entity payload.
///
/// The value model is intentionally small in Milestone 0.1. A later schema layer
/// can introduce richer database types without changing the mutation semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    /// Signed 64-bit integer value.
    Int(i64),
    /// UTF-8 text value.
    String(String),
    /// Boolean value.
    Bool(bool),
    /// Arbitrary binary value.
    Bytes(Vec<u8>),
}

/// Deterministically ordered set of named values representing entity state.
///
/// [`BTreeMap`] is used instead of `HashMap` so iteration order is stable. That
/// will be useful in later milestones when state nodes become content-addressed.
pub type Payload = BTreeMap<String, Value>;

/// Stable reference to one immutable version of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateRef {
    /// Identifier of the logical entity.
    pub entity_id: EntityId,
    /// Exact version referenced by a mutation.
    pub version: Version,
}

impl StateRef {
    /// Creates a new state reference.
    #[must_use]
    pub const fn new(entity_id: EntityId, version: Version) -> Self {
        Self { entity_id, version }
    }
}

/// Immutable state node representing one committed version of an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateNode {
    /// Unique identifier of this immutable state node.
    pub id: NodeId,
    /// Logical entity whose state this node represents.
    pub entity_id: EntityId,
    /// Sequential version of the logical entity.
    pub version: Version,
    /// Materialized payload at this version.
    pub payload: Payload,
    /// Previous version of the same entity, if one exists.
    pub previous_version: Option<Version>,
    /// Mutation that created this state node, or `None` for the initial state.
    pub caused_by: Option<MutationId>,
}

impl StateNode {
    /// Creates the initial state node of a newly inserted entity.
    #[must_use]
    pub fn initial(entity_id: EntityId, payload: Payload) -> Self {
        Self {
            id: NodeId::new(),
            entity_id,
            version: Version::INITIAL,
            payload,
            previous_version: None,
            caused_by: None,
        }
    }

    /// Creates the next immutable state node produced by a mutation.
    #[must_use]
    pub fn successor(current: &Self, payload: Payload, mutation_id: MutationId) -> Self {
        Self {
            id: NodeId::new(),
            entity_id: current.entity_id,
            version: current.version.next(),
            payload,
            previous_version: Some(current.version),
            caused_by: Some(mutation_id),
        }
    }

    /// Returns a compact reference to this state node.
    #[must_use]
    pub const fn as_ref(&self) -> StateRef {
        StateRef::new(self.entity_id, self.version)
    }
}
