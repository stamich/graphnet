use graphnet_core::{
    Entity, EntityUpdate, GraphError, GraphState, Mutation, Payload, StateRef, Value, Version,
};

/// Creates an account payload used by the integration tests.
fn account_payload(balance: i64) -> Payload {
    let mut payload = Payload::new();
    payload.insert("balance".to_string(), Value::Int(balance));
    payload
}

/// Extracts the numeric balance field from an account payload.
fn balance_of(payload: &Payload) -> i64 {
    match payload.get("balance") {
        Some(Value::Int(value)) => *value,
        other => panic!("expected integer balance, got {other:?}"),
    }
}

/// Verifies that an atomic transfer creates two successor versions and a commit.
#[test]
fn should_apply_transfer_mutation() {
    let mut graph = GraphState::new();

    let account_a = Entity::new("Account", account_payload(100));
    let account_b = Entity::new("Account", account_payload(50));
    let a_id = account_a.id;
    let b_id = account_b.id;

    graph.insert_entity(account_a).unwrap();
    graph.insert_entity(account_b).unwrap();

    let mutation = Mutation::new(
        vec![
            StateRef::new(a_id, Version::INITIAL),
            StateRef::new(b_id, Version::INITIAL),
        ],
        vec![
            EntityUpdate::new(a_id, Version::INITIAL, account_payload(80)),
            EntityUpdate::new(b_id, Version::INITIAL, account_payload(70)),
        ],
    );
    let mutation_id = mutation.id;

    let commit = graph.apply(mutation).unwrap();

    assert_eq!(commit.transitions.len(), 2);
    assert_eq!(commit.dependencies.len(), 2);
    assert_eq!(graph.current_state(a_id).unwrap().version, Version::new(2));
    assert_eq!(graph.current_state(b_id).unwrap().version, Version::new(2));
    assert_eq!(balance_of(&graph.current_state(a_id).unwrap().payload), 80);
    assert_eq!(balance_of(&graph.current_state(b_id).unwrap().payload), 70);
    assert_eq!(graph.history(a_id).unwrap().len(), 2);
    assert_eq!(graph.history(b_id).unwrap().len(), 2);
    assert_eq!(graph.dependencies_of(mutation_id).unwrap().len(), 2);
    assert!(graph.commit(mutation_id).is_some());
    assert!(graph.mutation(mutation_id).is_some());
}

/// Verifies optimistic concurrency by rejecting a mutation prepared on stale data.
#[test]
fn should_reject_stale_mutation() {
    let mut graph = GraphState::new();
    let account = Entity::new("Account", account_payload(100));
    let account_id = account.id;
    graph.insert_entity(account).unwrap();

    let first = Mutation::new(
        vec![StateRef::new(account_id, Version::INITIAL)],
        vec![EntityUpdate::new(
            account_id,
            Version::INITIAL,
            account_payload(90),
        )],
    );
    graph.apply(first).unwrap();

    let stale = Mutation::new(
        vec![StateRef::new(account_id, Version::INITIAL)],
        vec![EntityUpdate::new(
            account_id,
            Version::INITIAL,
            account_payload(80),
        )],
    );

    let result = graph.apply(stale);

    assert!(matches!(
        result,
        Err(GraphError::VersionConflict {
            entity_id,
            expected,
            actual
        }) if entity_id == account_id
            && expected == Version::INITIAL
            && actual == Version::new(2)
    ));
    assert_eq!(balance_of(&graph.current_state(account_id).unwrap().payload), 90);
}

/// Verifies that failed multi-entity validation leaves every entity unchanged.
#[test]
fn should_keep_multi_entity_mutation_atomic() {
    let mut graph = GraphState::new();

    let account_a = Entity::new("Account", account_payload(100));
    let account_b = Entity::new("Account", account_payload(50));
    let a_id = account_a.id;
    let b_id = account_b.id;

    graph.insert_entity(account_a).unwrap();
    graph.insert_entity(account_b).unwrap();

    let invalid = Mutation::new(
        vec![
            StateRef::new(a_id, Version::INITIAL),
            StateRef::new(b_id, Version::INITIAL),
        ],
        vec![
            EntityUpdate::new(a_id, Version::INITIAL, account_payload(80)),
            EntityUpdate::new(b_id, Version::new(999), account_payload(70)),
        ],
    );

    assert!(matches!(
        graph.apply(invalid),
        Err(GraphError::VersionConflict { entity_id, .. }) if entity_id == b_id
    ));

    assert_eq!(graph.current_state(a_id).unwrap().version, Version::INITIAL);
    assert_eq!(graph.current_state(b_id).unwrap().version, Version::INITIAL);
    assert_eq!(balance_of(&graph.current_state(a_id).unwrap().payload), 100);
    assert_eq!(balance_of(&graph.current_state(b_id).unwrap().payload), 50);
    assert_eq!(graph.history(a_id).unwrap().len(), 1);
    assert_eq!(graph.history(b_id).unwrap().len(), 1);
}

/// Verifies that mutations touching disjoint entities can be committed independently.
#[test]
fn should_apply_independent_mutations() {
    let mut graph = GraphState::new();

    let account_a = Entity::new("Account", account_payload(100));
    let account_b = Entity::new("Account", account_payload(50));
    let a_id = account_a.id;
    let b_id = account_b.id;

    graph.insert_entity(account_a).unwrap();
    graph.insert_entity(account_b).unwrap();

    graph
        .apply(Mutation::new(
            vec![StateRef::new(a_id, Version::INITIAL)],
            vec![EntityUpdate::new(
                a_id,
                Version::INITIAL,
                account_payload(95),
            )],
        ))
        .unwrap();

    graph
        .apply(Mutation::new(
            vec![StateRef::new(b_id, Version::INITIAL)],
            vec![EntityUpdate::new(
                b_id,
                Version::INITIAL,
                account_payload(60),
            )],
        ))
        .unwrap();

    assert_eq!(graph.current_state(a_id).unwrap().version, Version::new(2));
    assert_eq!(graph.current_state(b_id).unwrap().version, Version::new(2));
    assert_eq!(balance_of(&graph.current_state(a_id).unwrap().payload), 95);
    assert_eq!(balance_of(&graph.current_state(b_id).unwrap().payload), 60);
}

/// Verifies that a read dependency must point to an existing historical version.
#[test]
fn should_reject_unknown_dependency() {
    let mut graph = GraphState::new();
    let account = Entity::new("Account", account_payload(100));
    let account_id = account.id;
    graph.insert_entity(account).unwrap();

    let mutation = Mutation::new(
        vec![StateRef::new(account_id, Version::new(999))],
        vec![EntityUpdate::new(
            account_id,
            Version::INITIAL,
            account_payload(90),
        )],
    );

    assert!(matches!(
        graph.apply(mutation),
        Err(GraphError::StateReferenceNotFound { .. })
    ));
    assert_eq!(graph.current_state(account_id).unwrap().version, Version::INITIAL);
}

/// Verifies that duplicate writes to one entity are rejected before state changes.
#[test]
fn should_reject_duplicate_writes() {
    let mut graph = GraphState::new();
    let account = Entity::new("Account", account_payload(100));
    let account_id = account.id;
    graph.insert_entity(account).unwrap();

    let mutation = Mutation::new(
        vec![StateRef::new(account_id, Version::INITIAL)],
        vec![
            EntityUpdate::new(account_id, Version::INITIAL, account_payload(90)),
            EntityUpdate::new(account_id, Version::INITIAL, account_payload(80)),
        ],
    );

    assert!(matches!(
        graph.apply(mutation),
        Err(GraphError::DuplicateWrite { entity_id }) if entity_id == account_id
    ));
    assert_eq!(graph.current_state(account_id).unwrap().version, Version::INITIAL);
}

/// Verifies that a mutation without writes is rejected as semantically empty.
#[test]
fn should_reject_empty_mutation() {
    let mut graph = GraphState::new();
    let account = Entity::new("Account", account_payload(100));
    let account_id = account.id;
    graph.insert_entity(account).unwrap();

    let mutation = Mutation::new(vec![StateRef::new(account_id, Version::INITIAL)], vec![]);

    assert_eq!(graph.apply(mutation), Err(GraphError::EmptyMutation));
}
