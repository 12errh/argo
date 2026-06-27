use crate::error::MemoryError;
use crate::redis::StoredTurn;

#[test]
fn test_memory_error_display() {
    let err = MemoryError::Redis("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}

#[test]
fn test_stored_turn_serialization() {
    let turn = StoredTurn {
        role: "user".to_string(),
        content: "Hello".to_string(),
    };
    let json = serde_json::to_string(&turn).unwrap();
    assert!(json.contains("user"));
    assert!(json.contains("Hello"));
}
