//! Commit records and state-transition descriptions produced by accepted mutations.

use serde::{Deserialize, Serialize};

use crate::{
    id::{EntityId, MutationId, NodeId},
    state::{StateRef, Version},
};

/// Description of one entity version transition produced by a commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransition {
    /// Logical entity that changed.
    pub entity_id: EntityId,
    /// Version observed before the mutation.
    pub from_version: Version,
    /// Version created by the mutation.
    pub to_version: Version,
    /// Identifier of the newly created immutable state node.
    pub new_node_id: NodeId,
}

/// Durable logical result of an accepted mutation.
///
/// Milestone 0.1 stores commits only in memory, but this representation is already
/// shaped so it can later be appended to a WAL or replicated to peers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    /// Mutation that produced this commit.
    pub mutation_id: MutationId,
    /// Explicit immutable states the mutation depended on.
    pub dependencies: Vec<StateRef>,
    /// State transitions created atomically by the mutation.
    pub transitions: Vec<StateTransition>,
}
