//! Core domain model and in-memory execution engine for GraphNet Milestone 0.1.
//!
//! The milestone deliberately focuses on a single-node, in-memory execution model.
//! It provides immutable entity versions, explicit mutation dependencies, atomic
//! multi-entity state transitions, optimistic version checks, commit records and
//! historical state inspection.
//!
//! Networking, persistence, consensus, cryptographic proofs and asynchronous
//! execution are intentionally outside the scope of this milestone.

pub mod commit;
pub mod entity;
pub mod error;
pub mod graph;
pub mod id;
pub mod mutation;
pub mod state;

pub use commit::{Commit, StateTransition};
pub use entity::Entity;
pub use error::GraphError;
pub use graph::GraphState;
pub use id::{EntityId, MutationId, NodeId};
pub use mutation::{EntityUpdate, Mutation};
pub use state::{Payload, StateNode, StateRef, Value, Version};
