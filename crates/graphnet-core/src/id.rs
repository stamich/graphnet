//! Strong identifier types that prevent mixing entities, mutations and state nodes.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Strongly typed identifier of a logical entity managed by GraphNet.
///
/// An [`EntityId`] identifies the logical object across all of its versions.
/// Individual immutable versions are identified separately by [`NodeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Creates a new random entity identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    /// Creates a new random entity identifier.
    fn default() -> Self {
        Self::new()
    }
}

/// Strongly typed identifier of a mutation.
///
/// A mutation is an atomic logical operation that may read several state
/// versions and create several new state versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MutationId(Uuid);

impl MutationId {
    /// Creates a new random mutation identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for MutationId {
    /// Creates a new random mutation identifier.
    fn default() -> Self {
        Self::new()
    }
}

/// Strongly typed identifier of an immutable state node.
///
/// One logical entity can have many [`NodeId`] values because every committed
/// version is represented by its own state node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// Creates a new random node identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for NodeId {
    /// Creates a new random node identifier.
    fn default() -> Self {
        Self::new()
    }
}
