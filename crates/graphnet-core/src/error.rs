//! Error model for single-node GraphNet validation and mutation execution.

use thiserror::Error;

use crate::{
    id::EntityId,
    state::{StateRef, Version},
};

/// Errors produced by the Milestone 0.1 GraphNet execution engine.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    /// Requested entity does not exist in the current graph state.
    #[error("entity {entity_id:?} was not found")]
    EntityNotFound {
        /// Missing entity identifier.
        entity_id: EntityId,
    },

    /// Caller attempted to insert an entity that already exists.
    #[error("entity {entity_id:?} already exists")]
    DuplicateEntity {
        /// Duplicate entity identifier.
        entity_id: EntityId,
    },

    /// Mutation was prepared against an older or otherwise unexpected version.
    #[error(
        "version conflict for entity {entity_id:?}: expected {expected:?}, actual {actual:?}"
    )]
    VersionConflict {
        /// Entity for which the optimistic version check failed.
        entity_id: EntityId,
        /// Version expected by the mutation.
        expected: Version,
        /// Current version stored by GraphNet.
        actual: Version,
    },

    /// Mutation references a state version that does not exist in history.
    #[error("referenced state {state_ref:?} does not exist")]
    StateReferenceNotFound {
        /// Invalid dependency reference.
        state_ref: StateRef,
    },

    /// Mutation contains two writes to the same logical entity.
    #[error("mutation contains duplicate write for entity {entity_id:?}")]
    DuplicateWrite {
        /// Entity written more than once by the same mutation.
        entity_id: EntityId,
    },

    /// Mutation contains no writes and therefore cannot create a state transition.
    #[error("mutation must contain at least one write")]
    EmptyMutation,
}
