// File: crates/storm-digital-human/src/animation.rs
// Description: Animation system for digital humans
// Manages facial expressions, gestures, and emotional animations
// Implements a blend tree for smooth transitions between animations
// Handles emotional layering, facial control, and gesture triggering
// Ensures thread-safe updates and priority-based animation queuing

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use storm_math::{Vec3, Quat};
use storm_ecs::prelude::*;
use crate::emotion::Emotion;
use crate::behavior::BehaviorState;

#[derive(Component, Debug, Clone)]
pub struct AnimationController {
    pub current_animations: Vec<ActiveAnimation>,
    pub animation_queue: Vec<QueuedAnimation>,
    pub blend_tree: AnimationBlendTree,
    pub emotional_layer: EmotionalAnimationLayer,
    pub facial_controller: FacialAnimationController,
    pub gesture_controller: GestureController,
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            current_animations: Vec::new(),
            animation_queue: Vec::new(),
            blend_tree: AnimationBlendTree::new(),
            emotional_layer: EmotionalAnimationLayer::new(),
            facial_controller: FacialAnimationController::new(),
            gesture_controller: GestureController::new(),
        }
    }

    pub fn play_animation(&mut self, animation: AnimationRequest) {
        match animation.priority {
            AnimationPriority::Immediate => {
                // Stop conflicting animations and play immediately
                self.stop_conflicting_animations(&animation);
                self.start_animation(animation);
            }
            AnimationPriority::High => {
                // Queue with high priority
                self.animation_queue.insert(0, QueuedAnimation {
                    request: animation,
                    queued_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f32(),
                });
            }
            AnimationPriority::Normal => {
                // Add to end of queue
                self.animation_queue.push(QueuedAnimation {
                    request: animation,
                    queued_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f32(),
                });
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update current animations
        self.current_animations.retain_mut(|anim| {
            anim.current_time += delta_time;
            anim.current_time < anim.duration
        });

        // Process animation queue
        self.process_queue();

        // Update blend tree
        self.blend_tree.update(delta_time);

        // Update emotional animations
        self.emotional_layer.update(delta_time);

        // Update facial animations
        self.facial_controller.update(delta_time);

        // Update gestures
        self.gesture_controller.update(delta_time);
    }

    fn start_animation(&mut self, request: AnimationRequest) {
        let active_animation = ActiveAnimation {
            name: request.name.clone(),
            animation_type: request.animation_type,
            current_time: 0.0,
            duration: request.duration,
            loop_count: request.loop_count,
            blend_weight: 1.0,
            layer: request.layer,
            bone_masks: request.bone_masks.unwrap_or_default(),
        };

        self.current_animations.push(active_animation);
    }

    fn stop_conflicting_animations(&mut self, new_animation: &AnimationRequest) {
        self.current_animations.retain(|anim| {
            !self.animations_conflict(&anim.animation_type, &new_animation.animation_type)
        });
    }

    fn animations_conflict(&self, anim1: &AnimationType, anim2: &AnimationType) -> bool {
        use AnimationType::*;

        match (anim1, anim2) {
            (Locomotion(_), Locomotion(_)) => true,
            (FullBody(_), FullBody(_)) => true,
            (FullBody(_), Locomotion(_)) => true,
            (Locomotion(_), FullBody(_)) => true,
            (UpperBody(_), UpperBody(_)) => true,
            (FullBody(_), UpperBody(_)) => true,
            (UpperBody(_), FullBody(_)) => true,
            _ => false,
        }
    }

    fn process_queue(&mut self) {
        if let Some(queued) = self.animation_queue.first() {
            // Check if we can start the next animation
            if self.can_start_animation(&queued.request) {
                let animation = self.animation_queue.remove(0);
                self.start_animation(animation.request);
            }
        }
    }

    fn can_start_animation(&self, request: &AnimationRequest) -> bool {
        // Check if any current animations would conflict
        !self.current_animations.iter().any(|anim| {
            self.animations_conflict(&anim.animation_type, &request.animation_type)
        })
    }

    pub fn set_emotional_state(&mut self, emotion: Emotion, intensity: f32) {
        self.emotional_layer.set_emotion(emotion, intensity);
        self.facial_controller.express_emotion(emotion, intensity);
    }

    pub fn trigger_gesture(&mut self, gesture: GestureType, intensity: f32) {
        self.gesture_controller.trigger_gesture(gesture, intensity);
    }

    pub fn set_facial_expression(&mut self, expression: FacialExpression, intensity: f32) {
        self.facial_controller.set_expression(expression, intensity);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationRequest {
    pub name: String,
    pub animation_type: AnimationType,
    pub duration: f32,
    pub loop_count: u32, // 0 = infinite
    pub priority: AnimationPriority,
    pub layer: AnimationLayer,
    pub bone_masks: Option<Vec<String>>, // Which bones this animation affects
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationType {
    Idle(IdleType),
    Locomotion(LocomotionType),
    UpperBody(UpperBodyType),
    FullBody(FullBodyType),
    Facial(FacialType),
    Gesture(GestureType),
    Emotional(Emotion),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdleType {
    Default,
    Bored,
    Alert,
    Relaxed,
    Nervous,
    Confident,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocomotionType {
    Walk,
    Run,
    Sneak,
    Strafe,
    Backward,
    Jump,
    Turn(f32), // Turn angle in radians
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpperBodyType {
    Wave,
    Point,
    Shrug,
    CrossArms,
    HandsOnHips,
    Scratch,
    Salute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FullBodyType {
    Bow,
    Kneel,
    Sit,
    Lie,
    Dance,
    Combat(CombatAnimationType),
    Interact(InteractionType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatAnimationType {
    Attack,
    Block,
    Dodge,
    Cast,
    Reload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    PickUp,
    Use,
    Open,
    Close,
    Push,
    Pull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FacialType {
    Smile,
    Frown,
    Surprise,
    Angry,
    Sad,
    Neutral,
    Wink,
    Blink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureType {
    Nod,
    Shake,
    Shrug,
    ThumbsUp,
    ThumbsDown,
    Peace,
    Okay,
    Stop,
    Come,
    Go,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationPriority {
    Immediate, // Stop everything and play now
    High,      // Play as soon as possible
    Normal,    // Queue normally
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationLayer {
    Base,      // Full body base layer
    Additive,  // Additive layer for emotions/reactions
    Override,  // Override layer for important actions
    Facial,    // Facial expression layer
    Gesture,   // Hand gesture layer
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlendMode {
    Replace,   // Replace existing animation
    Additive,  // Add to existing animation
    Multiply,  // Multiply with existing animation
}

#[derive(Debug, Clone)]
pub struct ActiveAnimation {
    pub name: String,
    pub animation_type: AnimationType,
    pub current_time: f32,
    pub duration: f32,
    pub loop_count: u32,
    pub blend_weight: f32,
    pub layer: AnimationLayer,
    pub bone_masks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct QueuedAnimation {
    pub request: AnimationRequest,
    pub queued_time: f32,
}

// Animation blend tree for smooth transitions
#[derive(Debug, Clone)]
pub struct AnimationBlendTree {
    pub nodes: Vec<BlendNode>,
    pub root_node: Option<usize>,
}

impl AnimationBlendTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_node: None,
        }
    }

    pub fn add_node(&mut self, node: BlendNode) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn set_root(&mut self, node_index: usize) {
        self.root_node = Some(node_index);
    }

    pub fn update(&mut self, delta_time: f32) {
        // Traverse the blend tree from root and update all nodes
        if let Some(root) = self.root_node {
            self.update_node(root, delta_time);
        }
    }

    fn update_node(&mut self, index: usize, delta_time: f32) {
        // Recursively update child nodes and compute blends
        match &mut self.nodes[index] {
            BlendNode::Clip { current_time, speed, .. } => {
                // Advance clip time
                *current_time += delta_time * *speed;
                // Handle looping logic if needed (assuming external handling for now)
            }
            BlendNode::Blend { children, weights, .. } => {
                // Update children recursively
                for &child in children {
                    self.update_node(child, delta_time);
                }
                // Normalize weights if sum != 1.0
                let total_weight: f32 = weights.iter().sum();
                if total_weight != 1.0 {
                    for weight in weights.iter_mut() {
                        *weight /= total_weight;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlendNode {
    Clip {
        animation_name: String,
        speed: f32,
        current_time: f32,
    },
    Blend {
        children: Vec<usize>,
        weights: Vec<f32>,
        blend_time: f32,
    },
}

// Emotional animation layer for overlaying emotion-based animations
#[derive(Debug, Clone)]
pub struct EmotionalAnimationLayer {
    current_emotion: Option<Emotion>,
    intensity: f32,
    blend_time: f32,
    current_blend: f32,
}

impl EmotionalAnimationLayer {
    pub fn new() -> Self {
        Self {
            current_emotion: None,
            intensity: 0.0,
            blend_time: 0.5, // Default blend duration
            current_blend: 0.0,
        }
    }

    pub fn set_emotion(&mut self, emotion: Emotion, intensity: f32) {
        self.current_emotion = Some(emotion);
        self.intensity = intensity.clamp(0.0, 1.0);
        self.current_blend = 0.0; // Reset blend for new emotion
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.current_blend < self.blend_time {
            self.current_blend += delta_time;
            self.current_blend = self.current_blend.min(self.blend_time);
        }
        // Apply emotional animation overlay based on current_emotion and intensity
        // In a full system, this would blend poses or parameters
    }
}

// Facial animation controller for expressions
#[derive(Debug, Clone)]
pub struct FacialAnimationController {
    current_expression: Option<FacialExpression>,
    intensity: f32,
    morph_targets: HashMap<String, f32>, // Bone/morph target weights
}

impl FacialAnimationController {
    pub fn new() -> Self {
        Self {
            current_expression: None,
            intensity: 0.0,
            morph_targets: HashMap::new(),
        }
    }

    pub fn express_emotion(&mut self, emotion: Emotion, intensity: f32) {
        // Map emotion to facial expression
        let expression = match emotion {
            Emotion::Joy => FacialExpression::Smile,
            Emotion::Sadness => FacialExpression::Sad,
            Emotion::Anger => FacialExpression::Angry,
            Emotion::Fear => FacialExpression::Surprise,
            _ => FacialExpression::Neutral,
        };
        self.set_expression(expression, intensity);
    }

    pub fn set_expression(&mut self, expression: FacialExpression, intensity: f32) {
        self.current_expression = Some(expression);
        self.intensity = intensity.clamp(0.0, 1.0);
        // Update morph targets based on expression
        // Example: self.morph_targets.insert("mouth_smile".to_string(), intensity);
        // In full implementation, this would set blend shape weights
    }

    pub fn update(&mut self, delta_time: f32) {
        // Smoothly interpolate facial morph targets if needed
        // For example, transition between expressions
    }
}

// Gesture controller for hand and body gestures
#[derive(Debug, Clone)]
pub struct GestureController {
    active_gestures: Vec<ActiveGesture>,
}

impl GestureController {
    pub fn new() -> Self {
        Self {
            active_gestures: Vec::new(),
        }
    }

    pub fn trigger_gesture(&mut self, gesture: GestureType, intensity: f32) {
        let active_gesture = ActiveGesture {
            gesture_type: gesture,
            intensity,
            current_time: 0.0,
            duration: 1.0, // Default duration, can be per-gesture
        };
        self.active_gestures.push(active_gesture);
    }

    pub fn update(&mut self, delta_time: f32) {
        self.active_gestures.retain_mut(|gesture| {
            gesture.current_time += delta_time;
            if gesture.current_time >= gesture.duration {
                false // Remove completed gestures
            } else {
                // Update gesture animation based on type and intensity
                // In full system, blend gesture animations
                true
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct ActiveGesture {
    pub gesture_type: GestureType,
    pub intensity: f32,
    pub current_time: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FacialExpression {
    Smile,
    Frown,
    Surprise,
    Angry,
    Sad,
    Neutral,
    Wink,
    Blink,
}