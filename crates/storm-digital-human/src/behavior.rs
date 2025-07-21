// File: crates/storm-digital-human/src/behavior.rs
// Description: AI-driven NPC behavior system with HFSM
// Implements hierarchical finite state machines with ML integration

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;

// Mock imports for dependencies
pub use storm_ecs::prelude::*;

// Mock ECS Entity type for compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(pub u32);

impl Entity {
    pub fn new(id: u32) -> Self {
        Entity(id)
    }
}

// Mock Component trait
pub trait Component: Send + Sync + 'static {}

// Mock Vec3 type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

// Mock SongResonance type
#[derive(Debug, Clone)]
pub struct SongResonance {
    pub harmony_level: f32,
}

// Mock AI types
#[async_trait]
pub trait AIModel: Send + Sync + std::fmt::Debug {
    async fn predict(&self, features: &[f32]) -> Result<Vec<f32>, String>;
}

#[derive(Debug)]
pub struct AIClient;

impl AIClient {
    pub async fn generate(&self, prompt: String, params: GenerationParams) -> Result<String, String> {
        Ok("AI generated response".to_string())
    }
}

#[derive(Default)]
pub struct GenerationParams {
    pub max_tokens: usize,
    pub temperature: f32,
}

use crate::{emotion::Emotion, personality::PersonalityMatrix, memory::NPCMemory, dialogue::DialogueEngine};

#[derive(Debug)]
pub struct NPCBehavior {
    pub id: uuid::Uuid,
    pub personality: PersonalityMatrix,
    pub current_state: BehaviorState,
    pub behavior_tree: Arc<RwLock<BehaviorTree>>,
    pub memory: NPCMemory,
    pub dialogue_engine: DialogueEngine,
    pub last_update: f64,
}

impl Component for NPCBehavior {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorState {
    Idle {
        animation_set: String,
        duration: f32,
    },
    Interacting {
        target: Entity,
        interaction_type: InteractionType,
        progress: f32,
    },
    Following {
        target: Entity,
        min_distance: f32,
        max_distance: f32,
    },
    Performing {
        action: NPCAction,
        progress: f32,
        interruptible: bool,
    },
    Emotional {
        emotion: Emotion,
        intensity: f32,
        trigger: String,
    },
    Conversing {
        partner: Entity,
        topic: String,
        turn: ConversationTurn,
    },
    Exploring {
        destination: Option<Vec3>,
        curiosity_level: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Greeting,
    Trading,
    Teaching,
    Storytelling,
    Helping,
    Challenging,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NPCAction {
    Animation(String),
    Movement(Vec3),
    Speech(String),
    Emote(String),
    UseAbility(String),
    CreateObject(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationTurn {
    NPCSpeaking,
    WaitingForResponse,
    Processing,
    Ending,
}

// Behavior Tree Implementation
#[derive(Debug)]
pub struct BehaviorTree {
    root: Box<dyn BehaviorNode>,
    blackboard: Blackboard,
    ml_predictor: Option<MLBehaviorPredictor>,
}

#[async_trait]
pub trait BehaviorNode: Send + Sync + std::fmt::Debug {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult;
    fn reset(&mut self);
}

pub struct BehaviorContext<'a> {
    pub npc: &'a mut NPCBehavior,
    pub world_state: &'a WorldContext,
    pub delta_time: f32,
    pub blackboard: &'a mut Blackboard,
}

#[derive(Debug, Clone)]
pub enum NodeResult {
    Success,
    Failure,
    Running,
}

#[derive(Debug)]
pub struct Blackboard {
    data: HashMap<String, BlackboardValue>,
}

#[derive(Debug, Clone)]
pub enum BlackboardValue {
    Bool(bool),
    Float(f32),
    String(String),
    Entity(Entity),
    Vector(Vec3),
}

// Behavior Nodes
#[derive(Debug)]
pub struct SequenceNode {
    children: Vec<Box<dyn BehaviorNode>>,
    current_index: usize,
}

#[async_trait]
impl BehaviorNode for SequenceNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        while self.current_index < self.children.len() {
            match self.children[self.current_index].execute(context).await {
                NodeResult::Success => self.current_index += 1,
                NodeResult::Failure => {
                    self.reset();
                    return NodeResult::Failure;
                }
                NodeResult::Running => return NodeResult::Running,
            }
        }
        self.reset();
        NodeResult::Success
    }

    fn reset(&mut self) {
        self.current_index = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

#[derive(Debug)]
pub struct SelectorNode {
    children: Vec<Box<dyn BehaviorNode>>,
    current_index: usize,
}

#[async_trait]
impl BehaviorNode for SelectorNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        while self.current_index < self.children.len() {
            match self.children[self.current_index].execute(context).await {
                NodeResult::Success => {
                    self.reset();
                    return NodeResult::Success;
                }
                NodeResult::Failure => self.current_index += 1,
                NodeResult::Running => return NodeResult::Running,
            }
        }
        self.reset();
        NodeResult::Failure
    }

    fn reset(&mut self) {
        self.current_index = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

// AI-Enhanced Behavior Nodes
#[derive(Debug)]
pub struct AIDecisionNode {
    decision_type: String,
    options: Vec<String>,
}

#[async_trait]
impl BehaviorNode for AIDecisionNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        // Use AI to make decision based on personality and context
        let decision = self.make_ai_decision(context).await;

        context.blackboard.set(
            &format!("decision_{}", self.decision_type),
            BlackboardValue::String(decision),
        );

        NodeResult::Success
    }

    fn reset(&mut self) {}
}

impl AIDecisionNode {
    async fn make_ai_decision(&self, context: &BehaviorContext<'_>) -> String {
        // Simple personality-based decision for now
        // In real implementation, this would use the AI system
        let personality = &context.npc.personality;

        // Weight options based on personality traits
        let weights: Vec<f32> = self.options.iter().map(|option| {
            self.calculate_option_weight(option, personality)
        }).collect();

        // Select based on weights
        let total_weight: f32 = weights.iter().sum();
        let mut random = rand::random::<f32>() * total_weight;

        for (i, weight) in weights.iter().enumerate() {
            random -= weight;
            if random <= 0.0 {
                return self.options[i].clone();
            }
        }

        self.options[0].clone()
    }

    fn calculate_option_weight(&self, option: &str, personality: &PersonalityMatrix) -> f32 {
        // Map options to personality traits
        match option {
            "help" => personality.get_trait_value(crate::personality::PersonalityTrait::Compassion),
            "flee" => 1.0 - personality.get_trait_value(crate::personality::PersonalityTrait::Courage),
            "fight" => personality.get_trait_value(crate::personality::PersonalityTrait::Aggression),
            "negotiate" => personality.get_trait_value(crate::personality::PersonalityTrait::Diplomacy),
            _ => 0.5,
        }
    }
}

// Emotional Response Node
#[derive(Debug)]
pub struct EmotionalResponseNode {
    trigger: String,
}

#[async_trait]
impl BehaviorNode for EmotionalResponseNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        let emotional_state = &mut context.npc.personality.emotional_state;
        let response = emotional_state.process_trigger(&self.trigger, context.world_state);

        if let Some(emotion) = response.dominant_emotion() {
            context.npc.current_state = BehaviorState::Emotional {
                emotion: emotion.clone(),
                intensity: response.intensity,
                trigger: self.trigger.clone(),
            };
        }

        NodeResult::Success
    }

    fn reset(&mut self) {}
}

// ML Behavior Predictor
#[derive(Debug)]
pub struct MLBehaviorPredictor {
    model: Arc<dyn AIModel>,
}

impl MLBehaviorPredictor {
    pub async fn predict_next_state(
        &self,
        personality: &PersonalityMatrix,
        current_state: &BehaviorState,
        world_context: &WorldContext,
    ) -> BehaviorState {
        // Prepare features for ML model
        let features = self.extract_features(personality, current_state, world_context);

        // Get prediction from model
        let prediction = self.model.predict(&features).await.unwrap_or_default();

        // Convert prediction to behavior state
        self.prediction_to_state(prediction, world_context)
    }

    fn extract_features(
        &self,
        personality: &PersonalityMatrix,
        current_state: &BehaviorState,
        world_context: &WorldContext,
    ) -> Vec<f32> {
        let mut features = vec![];

        // Personality features
        features.extend(personality.to_feature_vector());

        // Current state features
        features.extend(self.state_to_features(current_state));

        // World context features
        features.extend(world_context.to_feature_vector());

        features
    }

    fn state_to_features(&self, state: &BehaviorState) -> Vec<f32> {
        // One-hot encode state type and extract relevant parameters
        match state {
            BehaviorState::Idle { duration, .. } => vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, *duration],
            BehaviorState::Interacting { progress, .. } => vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, *progress],
            BehaviorState::Following { min_distance, .. } => vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, *min_distance],
            BehaviorState::Performing { progress, .. } => vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, *progress],
            BehaviorState::Emotional { intensity, .. } => vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, *intensity],
            BehaviorState::Conversing { .. } => vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            BehaviorState::Exploring { curiosity_level, .. } => vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, *curiosity_level],
        }
    }

    fn prediction_to_state(&self, prediction: Vec<f32>, _world_context: &WorldContext) -> BehaviorState {
        // Find highest probability state
        let state_index = prediction.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        match state_index {
            0 => BehaviorState::Idle {
                animation_set: "idle_default".to_string(),
                duration: 5.0,
            },
            1 => BehaviorState::Exploring {
                destination: None,
                curiosity_level: 0.7,
            },
            // ... other states
            _ => BehaviorState::Idle {
                animation_set: "idle_default".to_string(),
                duration: 3.0,
            },
        }
    }
}

// World Context for behavior decisions
#[derive(Debug, Clone)]
pub struct WorldContext {
    pub time_of_day: f32,
    pub weather: WeatherType,
    pub nearby_entities: Vec<(Entity, f32)>, // (entity, distance)
    pub current_location: LocationType,
    pub global_events: Vec<String>,
    pub player_resonance: Option<SongResonance>,
}

impl WorldContext {
    pub fn to_feature_vector(&self) -> Vec<f32> {
        let mut features = vec![];

        // Time of day (normalized 0-1)
        features.push(self.time_of_day / 24.0);

        // Weather (one-hot encoded)
        features.extend(match self.weather {
            WeatherType::Clear => vec![1.0, 0.0, 0.0, 0.0],
            WeatherType::Rain => vec![0.0, 1.0, 0.0, 0.0],
            WeatherType::Storm => vec![0.0, 0.0, 1.0, 0.0],
            WeatherType::Fog => vec![0.0, 0.0, 0.0, 1.0],
        });

        // Number of nearby entities
        features.push(self.nearby_entities.len() as f32 / 10.0);

        // Player harmony level if present
        features.push(
            self.player_resonance
                .as_ref()
                .map(|r| r.harmony_level)
                .unwrap_or(0.0)
        );

        features
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WeatherType {
    Clear,
    Rain,
    Storm,
    Fog,
}

#[derive(Debug, Clone, Copy)]
pub enum LocationType {
    Town,
    Wilderness,
    Dungeon,
    Sacred,
    Corrupted,
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&BlackboardValue> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: BlackboardValue) {
        self.data.insert(key.to_string(), value);
    }
}

// Behavior Factory
pub struct BehaviorTreeFactory;

impl BehaviorTreeFactory {
    pub fn create_for_archetype(archetype: &NPCArchetype) -> BehaviorTree {
        let root = match archetype {
            NPCArchetype::Villager => Self::create_villager_tree(),
            NPCArchetype::Guardian => Self::create_guardian_tree(),
            NPCArchetype::Merchant => Self::create_merchant_tree(),
            NPCArchetype::Scholar => Self::create_scholar_tree(),
            NPCArchetype::EchoTouched => Self::create_echo_touched_tree(),
        };

        BehaviorTree {
            root,
            blackboard: Blackboard::new(),
            ml_predictor: None,
        }
    }

    fn create_villager_tree() -> Box<dyn BehaviorNode> {
        Box::new(SelectorNode {
            children: vec![
                // Priority 1: Respond to player interaction
                Box::new(SequenceNode {
                    children: vec![
                        Box::new(CheckConditionNode {
                            condition: "player_nearby".to_string(),
                        }),
                        Box::new(AIDecisionNode {
                            decision_type: "greeting".to_string(),
                            options: vec![
                                "wave".to_string(),
                                "verbal_greeting".to_string(),
                                "ignore".to_string(),
                            ],
                        }),
                    ],
                    current_index: 0,
                }),
                // Priority 2: Daily routine
                Box::new(DailyRoutineNode {
                    schedule: vec![
                        (6.0, "wake_up"),
                        (7.0, "breakfast"),
                        (8.0, "work"),
                        (12.0, "lunch"),
                        (13.0, "work"),
                        (18.0, "dinner"),
                        (20.0, "socialize"),
                        (22.0, "sleep"),
                    ],
                }),
            ],
            current_index: 0,
        })
    }

    fn create_guardian_tree() -> Box<dyn BehaviorNode> {
        Box::new(SelectorNode {
            children: vec![
                // Priority 1: Detect threats
                Box::new(SequenceNode {
                    children: vec![
                        Box::new(DetectThreatNode {
                            detection_radius: 20.0,
                        }),
                        Box::new(CombatBehaviorNode),
                    ],
                    current_index: 0,
                }),
                // Priority 2: Patrol
                Box::new(PatrolNode {
                    waypoints: vec![],
                    current_index: 0,
                }),
            ],
            current_index: 0,
        })
    }

    fn create_merchant_tree() -> Box<dyn BehaviorNode> {
        // Similar implementation for merchant behavior
        Box::new(SelectorNode {
            children: vec![],
            current_index: 0,
        })
    }

    fn create_scholar_tree() -> Box<dyn BehaviorNode> {
        // Similar implementation for scholar behavior
        Box::new(SelectorNode {
            children: vec![],
            current_index: 0,
        })
    }

    fn create_echo_touched_tree() -> Box<dyn BehaviorNode> {
        // Special behavior for Echo-touched NPCs
        Box::new(SelectorNode {
            children: vec![
                Box::new(EmotionalResponseNode {
                    trigger: "song_resonance".to_string(),
                }),
            ],
            current_index: 0,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NPCArchetype {
    Villager,
    Guardian,
    Merchant,
    Scholar,
    EchoTouched,
}

// Additional behavior nodes
#[derive(Debug)]
pub struct CheckConditionNode {
    condition: String,
}

#[async_trait]
impl BehaviorNode for CheckConditionNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        match self.condition.as_str() {
            "player_nearby" => {
                let nearby = context.world_state.nearby_entities.iter()
                    .any(|(_, dist)| *dist < 5.0);
                if nearby { NodeResult::Success } else { NodeResult::Failure }
            }
            _ => NodeResult::Failure,
        }
    }

    fn reset(&mut self) {}
}

#[derive(Debug)]
pub struct DailyRoutineNode {
    schedule: Vec<(f32, &'static str)>,
}

#[async_trait]
impl BehaviorNode for DailyRoutineNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        let current_time = context.world_state.time_of_day;

        // Find current activity based on time
        for i in 0..self.schedule.len() {
            let (time, activity) = self.schedule[i];
            let next_time = if i + 1 < self.schedule.len() {
                self.schedule[i + 1].0
            } else {
                24.0
            };

            if current_time >= time && current_time < next_time {
                context.blackboard.set(
                    "current_activity",
                    BlackboardValue::String(activity.to_string()),
                );
                return NodeResult::Success;
            }
        }

        NodeResult::Running
    }

    fn reset(&mut self) {}
}

#[derive(Debug)]
pub struct DetectThreatNode {
    detection_radius: f32,
}

#[async_trait]
impl BehaviorNode for DetectThreatNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        // Check for threats within radius
        for (entity, distance) in &context.world_state.nearby_entities {
            if *distance <= self.detection_radius {
                // In real implementation, check if entity is hostile
                context.blackboard.set(
                    "threat_target",
                    BlackboardValue::Entity(*entity),
                );
                return NodeResult::Success;
            }
        }
        NodeResult::Failure
    }

    fn reset(&mut self) {}
}

#[derive(Debug)]
pub struct CombatBehaviorNode;

#[async_trait]
impl BehaviorNode for CombatBehaviorNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        if let Some(BlackboardValue::Entity(target)) = context.blackboard.get("threat_target") {
            context.npc.current_state = BehaviorState::Performing {
                action: NPCAction::UseAbility("attack".to_string()),
                progress: 0.0,
                interruptible: false,
            };
            NodeResult::Running
        } else {
            NodeResult::Failure
        }
    }

    fn reset(&mut self) {}
}

#[derive(Debug)]
pub struct PatrolNode {
    waypoints: Vec<Vec3>,
    current_index: usize,
}

#[async_trait]
impl BehaviorNode for PatrolNode {
    async fn execute(&mut self, context: &mut BehaviorContext) -> NodeResult {
        if self.waypoints.is_empty() {
            return NodeResult::Failure;
        }

        let target = self.waypoints[self.current_index];
        context.npc.current_state = BehaviorState::Performing {
            action: NPCAction::Movement(target),
            progress: 0.0,
            interruptible: true,
        };

        // In real implementation, check if reached waypoint
        self.current_index = (self.current_index + 1) % self.waypoints.len();

        NodeResult::Running
    }

    fn reset(&mut self) {
        self.current_index = 0;
    }
}

// Mock storm_ecs module
pub mod storm_ecs {
    pub mod prelude {
        pub use super::super::{Entity, Component};
    }
}
