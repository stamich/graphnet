//! Logical entity definition used to create the first version of GraphNet state.

use serde::{Deserialize, Serialize};

use crate::{id::EntityId, state::Payload};

/// Logical entity introduced into GraphNet.
///
/// An entity owns a stable [`EntityId`] and an initial payload. Once inserted,
/// future changes are represented as immutable [`crate::StateNode`] versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// Stable logical identifier of the entity.
    pub id: EntityId,
    /// Application-defined type name, for example `Account` or `Order`.
    pub entity_type: String,
    /// Initial materialized state stored as version 1.
    pub initial_state: Payload,
}

impl Entity {
    /// Creates a new entity with a generated identifier.
    #[must_use]
    pub fn new(entity_type: impl Into<String>, initial_state: Payload) -> Self {
        Self {
            id: EntityId::new(),
            entity_type: entity_type.into(),
            initial_state,
        }
    }

    /// Creates an entity with an explicitly supplied identifier.
    ///
    /// This is useful for deterministic tests and future import or replication
    /// scenarios where the identifier is already known.
    #[must_use]
    pub fn with_id(
        id: EntityId,
        entity_type: impl Into<String>,
        initial_state: Payload,
    ) -> Self {
        Self {
            id,
            entity_type: entity_type.into(),
            initial_state,
        }
    }
}
