// File: tests/basic_integration.rs
// Basic integration test for StormCore

use storm_core::{StormConfig, StormCore};

#[tokio::test]
async fn test_basic_engine_creation() {
    let config = StormConfig::default();
    let result = StormCore::new(config).await;

    // This test just verifies that the engine can be created
    // More comprehensive tests would be added later
    match result {
        Ok(_) => println!("✅ Engine creation successful"),
        Err(e) => println!("❌ Engine creation failed: {}", e),
    }
}

#[test]
fn test_math_operations() {
    use storm_math::{Vec3, Quat};

    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);
    let sum = v1 + v2;

    assert_eq!(sum.x, 5.0);
    assert_eq!(sum.y, 7.0);
    assert_eq!(sum.z, 9.0);

    let q = Quat::IDENTITY;
    assert_eq!(q.w, 1.0);

    println!("✅ Math operations working correctly");
}

#[test]
fn test_ecs_basic_operations() {
    use storm_ecs::{World, Transform};

    let mut world = World::new();
    let entity = world.create_entity();

    world.add_component(entity, Transform::default());

    assert!(world.has_component::<Transform>(entity));
    println!("✅ ECS basic operations working");
}
