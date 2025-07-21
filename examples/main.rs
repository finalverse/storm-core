// File: examples/main.rs
// Description: Example usage of the storm-digital-human system
// Demonstrates creating NPCs with personalities, behaviors, and relationships

use storm_digital_human::prelude::*;
use storm_digital_human::{
    behavior::{Entity, NPCBehavior, BehaviorState, WorldContext, WeatherType, LocationType, SongResonance, BehaviorTreeFactory, NPCArchetype},
    personality::{PersonalityMatrix, PersonalityArchetypes},
    emotion::{Emotion, EmotionalState},
    memory::{NPCMemory, MemoryEvent, MemoryType},
    relationships::{RelationshipGraph, RelationshipNode, SocialTraits, Relationship, RelationshipType, RelationshipDelta},
    animation::{AnimationController, AnimationRequest, AnimationType, IdleType, AnimationPriority, AnimationLayer, BlendMode, GestureType},
    dialogue::{DialogueEngine, DialogueContext, ConversationMood},
};
use uuid::Uuid;
use std::sync::Arc;
use parking_lot::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Storm Digital Human System Demo");
    println!("=====================================");

    // Create entities
    let player_entity = Entity::new(1);
    let npc_entity = Entity::new(2);
    let merchant_entity = Entity::new(3);

    // Create world context
    let world_context = WorldContext {
        time_of_day: 14.5, // 2:30 PM
        weather: WeatherType::Clear,
        nearby_entities: vec![(player_entity, 3.0), (merchant_entity, 8.0)],
        current_location: LocationType::Town,
        global_events: vec!["Festival of Harmony".to_string()],
        player_resonance: Some(SongResonance { harmony_level: 0.7 }),
    };

    println!("\n🌍 World Context:");
    println!("  Time: {:.1} hours", world_context.time_of_day);
    println!("  Weather: {:?}", world_context.weather);
    println!("  Location: {:?}", world_context.current_location);
    println!("  Nearby entities: {}", world_context.nearby_entities.len());

    if let Some(resonance) = &world_context.player_resonance {
        println!("  Player harmony: {:.1}", resonance.harmony_level);
    }

    // Create personality matrices
    let hero_personality = PersonalityArchetypes::hero();
    let scholar_personality = PersonalityArchetypes::scholar();
    let merchant_personality = PersonalityArchetypes::guardian();

    println!("\n👤 Creating NPCs with different personalities...");

    // Create NPCs
    let mut elara_npc = create_npc(
        npc_entity,
        "Elara Vayne".to_string(),
        hero_personality,
        NPCArchetype::Guardian,
    );

    let mut scholar_npc = create_npc(
        Entity::new(4),
        "Master Aldric".to_string(),
        scholar_personality,
        NPCArchetype::Scholar,
    );

    println!("  ✅ Elara Vayne (Guardian) - Courageous and compassionate");
    println!("  ✅ Master Aldric (Scholar) - Curious and knowledgeable");

    // Create relationship graph
    let mut relationship_graph = RelationshipGraph::new();

    // Add entities to relationship graph
    relationship_graph.add_entity(player_entity, RelationshipNode {
        entity: player_entity,
        name: "Player".to_string(),
        reputation_score: 0.6,
        social_traits: SocialTraits {
            charisma: 0.7,
            trustworthiness: 0.8,
            influence: 0.5,
            loyalty: 0.9,
        },
    });

    relationship_graph.add_entity(npc_entity, RelationshipNode {
        entity: npc_entity,
        name: "Elara Vayne".to_string(),
        reputation_score: 0.8,
        social_traits: SocialTraits {
            charisma: 0.6,
            trustworthiness: 0.9,
            influence: 0.7,
            loyalty: 0.8,
        },
    });

    // Create initial relationship
    let initial_relationship = Relationship::new(RelationshipType::Neutral);
    relationship_graph.add_relationship(player_entity, npc_entity, initial_relationship);

    println!("\n🤝 Relationship System:");
    println!("  Initial relationship: {:?}", RelationshipType::Neutral);

    // Simulate a positive interaction
    let help_delta = RelationshipDelta::helped_in_combat();
    relationship_graph.update_relationship(player_entity, npc_entity, help_delta);

    if let Some(updated_relationship) = relationship_graph.get_relationship(player_entity, npc_entity) {
        println!("  After helping in combat:");
        println!("    Trust: {:.2}", updated_relationship.trust);
        println!("    Respect: {:.2}", updated_relationship.respect);
        println!("    Affection: {:.2}", updated_relationship.affection);
        println!("    Relationship Type: {:?}", updated_relationship.relationship_type);
    }

    // Test emotional responses
    println!("\n😊 Emotional System:");
    elara_npc.personality.emotional_state.process_trigger("friend_greeting", &world_context);

    if let Some(dominant_emotion) = elara_npc.personality.emotional_state.get_dominant_emotion() {
        println!("  Elara's dominant emotion: {:?}", dominant_emotion);
    }

    // Test memory system
    println!("\n🧠 Memory System:");
    let memory_event = MemoryEvent {
        description: "Player helped defend the village from bandits".to_string(),
        memory_type: MemoryType::Social {
            relationship_impact: 0.3,
            participants: vec![player_entity],
        },
        entities: vec![player_entity],
        location: Some("Village Square".to_string()),
        associated_emotion: Some(Emotion::Gratitude),
        tags: vec!["combat".to_string(), "heroic".to_string()],
    };

    elara_npc.memory.store_event(memory_event);
    println!("  ✅ Stored heroic memory for Elara");

    // Test dialogue system
    println!("\n💬 Dialogue System:");
    let dialogue_context = DialogueContext {
        location: "Village Square".to_string(),
        participants: vec![player_entity, npc_entity],
        topic: Some("recent_events".to_string()),
        mood: ConversationMood::Friendly,
        relationship_level: 0.7,
        current_time: world_context.time_of_day,
        world_state: world_context.clone(),
    };

    let response = elara_npc.dialogue_engine.process_input(
        "Hello Elara, how are you feeling after the battle?",
        player_entity,
        &dialogue_context,
    ).await;

    println!("  Player: \"Hello Elara, how are you feeling after the battle?\"");
    println!("  Elara: \"{}\"", response.text);
    if let Some(emotion) = response.emotion {
        println!("  (Emotion: {:?})", emotion);
    }

    // Test animation system
    println!("\n🎭 Animation System:");
    let animation_request = AnimationRequest {
        name: "friendly_wave".to_string(),
        animation_type: AnimationType::Gesture(GestureType::ThumbsUp),
        duration: 2.0,
        loop_count: 1,
        priority: AnimationPriority::Normal,
        layer: AnimationLayer::Gesture,
        bone_masks: None,
        blend_mode: BlendMode::Additive,
    };

    // Add animation controller to NPC (in a real ECS system)
    let mut animation_controller = AnimationController::new();
    animation_controller.play_animation(animation_request);
    animation_controller.set_emotional_state(Emotion::Joy, 0.6);

    println!("  ✅ Elara plays friendly wave animation");
    println!("  ✅ Applied joyful emotional state");

    // Simulate behavior tree execution
    println!("\n🌳 Behavior Tree:");
    // Note: In a real implementation, this would be async and continuous
    println!("  ✅ Guardian behavior tree active");
    println!("  Current state: Interacting with player");

    // Test AI decision making
    println!("\n🤖 AI Decision Making:");
    let tendency_help = elara_npc.personality.calculate_behavior_tendency("help_others");
    let tendency_fight = elara_npc.personality.calculate_behavior_tendency("fight");
    let tendency_negotiate = elara_npc.personality.calculate_behavior_tendency("negotiate");

    println!("  Elara's behavioral tendencies:");
    println!("    Help others: {:.2}", tendency_help);
    println!("    Fight: {:.2}", tendency_fight);
    println!("    Negotiate: {:.2}", tendency_negotiate);

    // Reputation system
    println!("\n🏆 Reputation System:");
    relationship_graph.update_global_reputation(player_entity, "Village Guards", 0.2);
    relationship_graph.update_global_reputation(player_entity, "Merchants", 0.1);

    let guard_rep = relationship_graph.get_reputation_in_group(player_entity, "Village Guards");
    let merchant_rep = relationship_graph.get_reputation_in_group(player_entity, "Merchants");

    println!("  Player reputation:");
    println!("    Village Guards: {:.2}", guard_rep);
    println!("    Merchants: {:.2}", merchant_rep);

    // Memory consolidation simulation
    println!("\n🔄 Memory Consolidation:");
    elara_npc.memory.consolidate_memories(1.0); // 1 second delta
    println!("  ✅ Memories consolidated and prioritized");

    // Social network analysis
    println!("\n🕸️ Social Network:");
    let social_analysis = crate::relationships::RelationshipAnalyzer::analyze_social_network(
        &relationship_graph,
        npc_entity
    );

    println!("  Elara's social network:");
    println!("    Influence: {:.2}", social_analysis.influence);
    println!("    Connections: {}", social_analysis.connection_count);
    println!("    Average relationship quality: {:.2}", social_analysis.average_relationship_quality);

    println!("\n✨ Storm Digital Human System Demo Complete!");
    println!("   All systems working harmoniously together.");

    Ok(())
}

fn create_npc(
    entity: Entity,
    name: String,
    personality: PersonalityMatrix,
    archetype: NPCArchetype,
) -> NPCBehavior {
    NPCBehavior {
        id: Uuid::new_v4(),
        personality,
        current_state: BehaviorState::Idle {
            animation_set: "default_idle".to_string(),
            duration: 5.0,
        },
        behavior_tree: Arc::new(RwLock::new(BehaviorTreeFactory::create_for_archetype(&archetype))),
        memory: NPCMemory::new(),
        dialogue_engine: DialogueEngine::new(&PersonalityArchetypes::hero()),
        last_update: 0.0,
    }
}