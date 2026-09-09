//! Minimal command-line demonstration of an atomic GraphNet account transfer.

use graphnet_core::{
    Entity, EntityUpdate, GraphState, Mutation, Payload, StateRef, Value, Version,
};

/// Creates the payload used by demo account entities.
fn account_payload(balance: i64) -> Payload {
    let mut payload = Payload::new();
    payload.insert("balance".to_string(), Value::Int(balance));
    payload
}

/// Reads the numeric balance from a demo account payload.
fn account_balance(payload: &Payload) -> i64 {
    match payload.get("balance") {
        Some(Value::Int(value)) => *value,
        other => panic!("expected integer balance, got {other:?}"),
    }
}

/// Runs a small single-node GraphNet scenario demonstrating one atomic transfer.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = GraphState::new();

    let alice = Entity::new("Account", account_payload(100));
    let bob = Entity::new("Account", account_payload(50));
    let alice_id = alice.id;
    let bob_id = bob.id;

    graph.insert_entity(alice)?;
    graph.insert_entity(bob)?;

    println!("Initial state:");
    println!(
        "  Alice v{} = {}",
        graph.current_state(alice_id)?.version.value(),
        account_balance(&graph.current_state(alice_id)?.payload)
    );
    println!(
        "  Bob   v{} = {}",
        graph.current_state(bob_id)?.version.value(),
        account_balance(&graph.current_state(bob_id)?.payload)
    );

    let transfer = Mutation::new(
        vec![
            StateRef::new(alice_id, Version::INITIAL),
            StateRef::new(bob_id, Version::INITIAL),
        ],
        vec![
            EntityUpdate::new(alice_id, Version::INITIAL, account_payload(80)),
            EntityUpdate::new(bob_id, Version::INITIAL, account_payload(70)),
        ],
    );

    let commit = graph.apply(transfer)?;

    println!("\nCommitted mutation {:?}", commit.mutation_id.as_uuid());
    for transition in &commit.transitions {
        println!(
            "  {:?}: v{} -> v{}",
            transition.entity_id.as_uuid(),
            transition.from_version.value(),
            transition.to_version.value()
        );
    }

    println!("\nCurrent state:");
    println!(
        "  Alice v{} = {}",
        graph.current_state(alice_id)?.version.value(),
        account_balance(&graph.current_state(alice_id)?.payload)
    );
    println!(
        "  Bob   v{} = {}",
        graph.current_state(bob_id)?.version.value(),
        account_balance(&graph.current_state(bob_id)?.payload)
    );

    Ok(())
}
