//! In-memory GraphNet state engine responsible for atomic mutation execution.

use std::collections::{HashMap, HashSet};

use crate::{
    commit::{Commit, StateTransition},
    entity::Entity,
    error::GraphError,
    id::{EntityId, MutationId},
    mutation::Mutation,
    state::{StateNode, StateRef},
};

/// In-memory single-node GraphNet execution engine for Milestone 0.1.
///
/// The engine maintains the current materialized state of every entity, complete
/// per-entity version history, committed mutations and explicit dependency edges.
/// Mutations are validated fully before any state is changed, providing atomic
/// all-or-nothing semantics inside one process.
#[derive(Debug, Default)]
pub struct GraphState {
    current: HashMap<EntityId, StateNode>,
    history: HashMap<EntityId, Vec<StateNode>>,
    mutations: HashMap<MutationId, Mutation>,
    commits: HashMap<MutationId, Commit>,
    dependencies: HashMap<MutationId, Vec<StateRef>>,
}

impl GraphState {
    /// Creates an empty in-memory graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new logical entity and creates its initial immutable state node.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::DuplicateEntity`] when an entity with the same
    /// identifier has already been inserted.
    pub fn insert_entity(&mut self, entity: Entity) -> Result<StateRef, GraphError> {
        if self.current.contains_key(&entity.id) {
            return Err(GraphError::DuplicateEntity {
                entity_id: entity.id,
            });
        }

        let node = StateNode::initial(entity.id, entity.initial_state);
        let state_ref = node.as_ref();

        self.current.insert(entity.id, node.clone());
        self.history.insert(entity.id, vec![node]);

        Ok(state_ref)
    }

    /// Returns the current materialized state of an entity.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::EntityNotFound`] when the entity is unknown.
    pub fn current_state(&self, entity_id: EntityId) -> Result<&StateNode, GraphError> {
        self.current
            .get(&entity_id)
            .ok_or(GraphError::EntityNotFound { entity_id })
    }

    /// Returns the complete ordered version history of an entity.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::EntityNotFound`] when the entity is unknown.
    pub fn history(&self, entity_id: EntityId) -> Result<&[StateNode], GraphError> {
        self.history
            .get(&entity_id)
            .map(Vec::as_slice)
            .ok_or(GraphError::EntityNotFound { entity_id })
    }

    /// Returns a committed mutation by identifier.
    #[must_use]
    pub fn mutation(&self, mutation_id: MutationId) -> Option<&Mutation> {
        self.mutations.get(&mutation_id)
    }

    /// Returns a commit record by mutation identifier.
    #[must_use]
    pub fn commit(&self, mutation_id: MutationId) -> Option<&Commit> {
        self.commits.get(&mutation_id)
    }

    /// Returns explicit state dependencies recorded for a committed mutation.
    #[must_use]
    pub fn dependencies_of(&self, mutation_id: MutationId) -> Option<&[StateRef]> {
        self.dependencies.get(&mutation_id).map(Vec::as_slice)
    }

    /// Applies an atomic mutation to the graph.
    ///
    /// The method executes in three conceptual phases:
    ///
    /// 1. Validate every read dependency and every expected write version.
    /// 2. Build all successor state nodes without modifying shared state.
    /// 3. Publish every new state, mutation, dependency and commit together.
    ///
    /// Because all validation happens before publication, a failing mutation does
    /// not partially modify any entity.
    ///
    /// # Errors
    ///
    /// Returns a [`GraphError`] when the mutation is empty, contains duplicate
    /// writes, refers to an unknown state, or fails an optimistic version check.
    pub fn apply(&mut self, mutation: Mutation) -> Result<Commit, GraphError> {
        self.validate_mutation(&mutation)?;

        let mut new_states = Vec::with_capacity(mutation.writes.len());
        let mut transitions = Vec::with_capacity(mutation.writes.len());

        for update in &mutation.writes {
            let current = self
                .current
                .get(&update.entity_id)
                .expect("validated entity must exist");

            let successor =
                StateNode::successor(current, update.new_payload.clone(), mutation.id);

            transitions.push(StateTransition {
                entity_id: update.entity_id,
                from_version: current.version,
                to_version: successor.version,
                new_node_id: successor.id,
            });

            new_states.push(successor);
        }

        let commit = Commit {
            mutation_id: mutation.id,
            dependencies: mutation.reads.clone(),
            transitions,
        };

        for state in new_states {
            self.current.insert(state.entity_id, state.clone());
            self.history.entry(state.entity_id).or_default().push(state);
        }

        self.dependencies
            .insert(mutation.id, mutation.reads.clone());
        self.commits.insert(mutation.id, commit.clone());
        self.mutations.insert(mutation.id, mutation);

        Ok(commit)
    }

    /// Validates a mutation without changing graph state.
    ///
    /// This method verifies structural constraints, dependency existence and
    /// optimistic write versions. It intentionally does not yet perform the more
    /// advanced conflict-domain analysis planned for later milestones.
    ///
    /// # Errors
    ///
    /// Returns a [`GraphError`] describing the first invalid condition found.
    fn validate_mutation(&self, mutation: &Mutation) -> Result<(), GraphError> {
        if mutation.writes.is_empty() {
            return Err(GraphError::EmptyMutation);
        }

        let mut written_entities = HashSet::with_capacity(mutation.writes.len());

        for update in &mutation.writes {
            if !written_entities.insert(update.entity_id) {
                return Err(GraphError::DuplicateWrite {
                    entity_id: update.entity_id,
                });
            }

            let current = self.current_state(update.entity_id)?;

            if current.version != update.expected_version {
                return Err(GraphError::VersionConflict {
                    entity_id: update.entity_id,
                    expected: update.expected_version,
                    actual: current.version,
                });
            }
        }

        for dependency in &mutation.reads {
            if !self.state_ref_exists(*dependency) {
                return Err(GraphError::StateReferenceNotFound {
                    state_ref: *dependency,
                });
            }
        }

        Ok(())
    }

    /// Checks whether a state reference exists in immutable entity history.
    #[must_use]
    fn state_ref_exists(&self, state_ref: StateRef) -> bool {
        self.history
            .get(&state_ref.entity_id)
            .is_some_and(|versions| versions.iter().any(|node| node.version == state_ref.version))
    }
}
