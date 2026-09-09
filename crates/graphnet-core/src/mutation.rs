//! Mutation commands and entity updates submitted to the GraphNet engine.

use serde::{Deserialize, Serialize};

use crate::{
    id::{EntityId, MutationId},
    state::{Payload, StateRef, Version},
};

/// Requested update of one entity as part of an atomic mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityUpdate {
    /// Entity whose state should be replaced.
    pub entity_id: EntityId,
    /// Version the caller observed when preparing the mutation.
    pub expected_version: Version,
    /// Complete payload of the next entity version.
    pub new_payload: Payload,
}

impl EntityUpdate {
    /// Creates a new entity update guarded by an expected version.
    #[must_use]
    pub fn new(entity_id: EntityId, expected_version: Version, new_payload: Payload) -> Self {
        Self {
            entity_id,
            expected_version,
            new_payload,
        }
    }
}

/// Atomic GraphNet mutation.
///
/// `reads` explicitly records the immutable state versions used to derive the
/// operation. `writes` contains complete next-state payloads and expected version
/// guards. Later milestones will derive conflicts and causal relationships from
/// this information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mutation {
    /// Unique identifier of the mutation.
    pub id: MutationId,
    /// Immutable state versions read by the operation.
    pub reads: Vec<StateRef>,
    /// Entity state replacements requested by the operation.
    pub writes: Vec<EntityUpdate>,
}

impl Mutation {
    /// Creates a mutation with a generated identifier.
    #[must_use]
    pub fn new(reads: Vec<StateRef>, writes: Vec<EntityUpdate>) -> Self {
        Self {
            id: MutationId::new(),
            reads,
            writes,
        }
    }

    /// Creates a mutation with an explicitly supplied identifier.
    #[must_use]
    pub const fn with_id(
        id: MutationId,
        reads: Vec<StateRef>,
        writes: Vec<EntityUpdate>,
    ) -> Self {
        Self { id, reads, writes }
    }
}
