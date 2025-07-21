// File: crates/storm-digital-human/src/relationships.rs
// Description: Relationship system between NPCs and players - Fixed version
// Manages social networks, reputation, and dynamic relationship evolution

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use crate::behavior::Entity;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RelationshipGraph {
    pub graph: Graph<RelationshipNode, RelationshipEdge, Directed>,
    pub entity_to_node: HashMap<Entity, NodeIndex>,
    pub global_reputation: GlobalReputation,
}

impl RelationshipGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            entity_to_node: HashMap::new(),
            global_reputation: GlobalReputation::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity, node_data: RelationshipNode) -> NodeIndex {
        let node_index = self.graph.add_node(node_data);
        self.entity_to_node.insert(entity, node_index);
        node_index
    }

    pub fn add_relationship(&mut self, from: Entity, to: Entity, relationship: Relationship) {
        if let (Some(&from_node), Some(&to_node)) =
            (self.entity_to_node.get(&from), self.entity_to_node.get(&to)) {

            let edge = RelationshipEdge {
                relationship,
                last_interaction: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
                interaction_count: 1,
            };

            self.graph.add_edge(from_node, to_node, edge);
        }
    }

    pub fn update_relationship(&mut self, from: Entity, to: Entity, delta: RelationshipDelta) {
        let (from_node, to_node) = match (self.entity_to_node.get(&from), self.entity_to_node.get(&to)) {
            (Some(&from_node), Some(&to_node)) => (from_node, to_node),
            _ => return, // Early return if entities not found
        };

        // Check if edge exists
        if let Some(edge_index) = self.graph.find_edge(from_node, to_node) {
            // Update existing relationship
            if let Some(edge) = self.graph.edge_weight_mut(edge_index) {
                edge.relationship.apply_delta(&delta);
                edge.last_interaction = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                edge.interaction_count += 1;
            }
        } else {
            // Create new relationship if it doesn't exist
            let relationship = Relationship::from_delta(delta);
            self.add_relationship(from, to, relationship);
        }
    }

    pub fn get_relationship(&self, from: Entity, to: Entity) -> Option<&Relationship> {
        let from_node = self.entity_to_node.get(&from)?;
        let to_node = self.entity_to_node.get(&to)?;
        let edge_index = self.graph.find_edge(*from_node, *to_node)?;
        self.graph.edge_weight(edge_index).map(|edge| &edge.relationship)
    }

    pub fn get_mutual_friends(&self, entity1: Entity, entity2: Entity) -> Vec<Entity> {
        let mut mutual_friends = Vec::new();

        if let (Some(&node1), Some(&node2)) =
            (self.entity_to_node.get(&entity1), self.entity_to_node.get(&entity2)) {

            let neighbors1: std::collections::HashSet<_> =
                self.graph.neighbors(node1).collect();
            let neighbors2: std::collections::HashSet<_> =
                self.graph.neighbors(node2).collect();

            for common_neighbor in neighbors1.intersection(&neighbors2) {
                // Find the entity corresponding to this node
                for (entity, &node_index) in &self.entity_to_node {
                    if node_index == *common_neighbor {
                        mutual_friends.push(*entity);
                        break;
                    }
                }
            }
        }

        mutual_friends
    }

    pub fn calculate_influence(&self, entity: Entity) -> f32 {
        if let Some(&node) = self.entity_to_node.get(&entity) {
            let connections = self.graph.neighbors(node).count() as f32;
            let relationship_strength: f32 = self.graph.edges(node)
                .map(|edge| edge.weight().relationship.trust)
                .sum();

            (connections * 0.3 + relationship_strength * 0.7) / 10.0
        } else {
            0.0
        }
    }

    pub fn find_path_to_entity(&self, from: Entity, to: Entity) -> Option<Vec<Entity>> {
        use petgraph::algo::dijkstra;

        let from_node = self.entity_to_node.get(&from)?;
        let to_node = self.entity_to_node.get(&to)?;

        // Use inverse of trust as edge weight for shortest path
        let edge_weights = |edge: petgraph::graph::EdgeReference<RelationshipEdge>| {
            OrderedFloat(1.0 / (edge.weight().relationship.trust + 0.1)) // Avoid division by zero
        };

        let path_map = dijkstra(&self.graph, *from_node, Some(*to_node), edge_weights);

        if path_map.contains_key(to_node) {
            // Reconstruct path (simplified - would need proper path reconstruction)
            Some(vec![from, to]) // Placeholder - full implementation would trace the path
        } else {
            None
        }
    }

    pub fn get_reputation_in_group(&self, entity: Entity, group: &str) -> f32 {
        self.global_reputation.get_reputation(entity, group)
    }

    pub fn update_global_reputation(&mut self, entity: Entity, group: &str, delta: f32) {
        self.global_reputation.update_reputation(entity, group, delta);
    }

    pub fn decay_relationships(&mut self, delta_time: f32) {
        let decay_rate = 0.01 * delta_time; // Relationships decay slowly over time

        // Collect edges to remove to avoid borrowing conflicts
        let mut edges_to_remove = Vec::new();

        // First pass: decay relationships and collect weak ones
        for edge_index in self.graph.edge_indices() {
            if let Some(edge) = self.graph.edge_weight_mut(edge_index) {
                edge.relationship.decay(decay_rate);

                // Check if relationship should be removed
                if edge.relationship.is_negligible() {
                    edges_to_remove.push(edge_index);
                }
            }
        }

        // Second pass: remove weak relationships
        for edge_index in edges_to_remove {
            self.graph.remove_edge(edge_index);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNode {
    pub entity: Entity,
    pub name: String,
    pub reputation_score: f32,
    pub social_traits: SocialTraits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialTraits {
    pub charisma: f32,
    pub trustworthiness: f32,
    pub influence: f32,
    pub loyalty: f32,
}

#[derive(Debug, Clone)]
pub struct RelationshipEdge {
    pub relationship: Relationship,
    pub last_interaction: f64,
    pub interaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_type: RelationshipType,
    pub trust: f32,
    pub respect: f32,
    pub affection: f32,
    pub familiarity: f32,
    pub tension: f32,
    pub history: Vec<RelationshipEvent>,
}

impl Relationship {
    pub fn new(relationship_type: RelationshipType) -> Self {
        let (base_trust, base_respect, base_affection) = match relationship_type {
            RelationshipType::Friend => (0.6, 0.5, 0.7),
            RelationshipType::Enemy => (0.1, 0.2, 0.1),
            RelationshipType::Ally => (0.7, 0.8, 0.4),
            RelationshipType::Rival => (0.3, 0.6, 0.2),
            RelationshipType::Family => (0.8, 0.7, 0.9),
            RelationshipType::Romantic => (0.7, 0.6, 0.9),
            RelationshipType::Mentor => (0.6, 0.9, 0.5),
            RelationshipType::Student => (0.5, 0.7, 0.6),
            RelationshipType::Neutral => (0.5, 0.5, 0.5),
            RelationshipType::Stranger => (0.3, 0.3, 0.3),
        };

        Self {
            relationship_type,
            trust: base_trust,
            respect: base_respect,
            affection: base_affection,
            familiarity: 0.1,
            tension: 0.0,
            history: Vec::new(),
        }
    }

    pub fn from_delta(delta: RelationshipDelta) -> Self {
        let mut relationship = Self::new(RelationshipType::Neutral);
        relationship.apply_delta(&delta);
        relationship
    }

    pub fn apply_delta(&mut self, delta: &RelationshipDelta) {
        self.trust = (self.trust + delta.trust_change).clamp(0.0, 1.0);
        self.respect = (self.respect + delta.respect_change).clamp(0.0, 1.0);
        self.affection = (self.affection + delta.affection_change).clamp(0.0, 1.0);
        self.familiarity = (self.familiarity + delta.familiarity_change).clamp(0.0, 1.0);
        self.tension = (self.tension + delta.tension_change).clamp(0.0, 1.0);

        // Record the event
        self.history.push(RelationshipEvent {
            event_type: delta.event_type.clone(),
            impact: delta.calculate_total_impact(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        });

        // Keep history manageable
        if self.history.len() > 20 {
            self.history.remove(0);
        }

        // Update relationship type based on current values
        self.update_relationship_type();
    }

    fn update_relationship_type(&mut self) {
        let overall_positive = (self.trust + self.respect + self.affection) / 3.0;
        let overall_negative = self.tension;

        self.relationship_type = if overall_positive > 0.8 && self.affection > 0.9 {
            RelationshipType::Romantic
        } else if overall_positive > 0.7 && self.tension < 0.2 {
            RelationshipType::Friend
        } else if overall_negative > 0.7 || overall_positive < 0.3 {
            RelationshipType::Enemy
        } else if self.respect > 0.7 && self.tension > 0.4 {
            RelationshipType::Rival
        } else if overall_positive > 0.6 {
            RelationshipType::Ally
        } else {
            RelationshipType::Neutral
        };
    }

    pub fn decay(&mut self, decay_rate: f32) {
        // Relationships tend toward neutral over time without interaction
        let target_value = 0.5;

        self.trust += (target_value - self.trust) * decay_rate;
        self.respect += (target_value - self.respect) * decay_rate;
        self.affection += (target_value - self.affection) * decay_rate;
        self.tension *= 1.0 - decay_rate; // Tension decays faster

        // Familiarity decays slower
        self.familiarity *= 1.0 - (decay_rate * 0.1);
    }

    pub fn is_negligible(&self) -> bool {
        let total_strength = self.trust + self.respect + self.affection + self.tension;
        total_strength < 1.0 && self.familiarity < 0.1
    }

    pub fn get_overall_sentiment(&self) -> f32 {
        let positive = (self.trust + self.respect + self.affection) / 3.0;
        let negative = self.tension;
        positive - negative
    }

    pub fn to_feature_vector(&self) -> Vec<f32> {
        vec![
            self.trust,
            self.respect,
            self.affection,
            self.familiarity,
            self.tension,
            match self.relationship_type {
                RelationshipType::Friend => 1.0,
                RelationshipType::Enemy => -1.0,
                RelationshipType::Ally => 0.8,
                RelationshipType::Rival => -0.5,
                RelationshipType::Family => 0.9,
                RelationshipType::Romantic => 1.0,
                RelationshipType::Mentor => 0.6,
                RelationshipType::Student => 0.6,
                RelationshipType::Neutral => 0.0,
                RelationshipType::Stranger => 0.0,
            },
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RelationshipType {
    Friend,
    Enemy,
    Ally,
    Rival,
    Family,
    Romantic,
    Mentor,
    Student,
    Neutral,
    Stranger,
}

#[derive(Debug, Clone)]
pub struct RelationshipDelta {
    pub trust_change: f32,
    pub respect_change: f32,
    pub affection_change: f32,
    pub familiarity_change: f32,
    pub tension_change: f32,
    pub event_type: String,
}

impl RelationshipDelta {
    pub fn helped_in_combat() -> Self {
        Self {
            trust_change: 0.2,
            respect_change: 0.15,
            affection_change: 0.1,
            familiarity_change: 0.05,
            tension_change: -0.1,
            event_type: "helped_in_combat".to_string(),
        }
    }

    pub fn betrayed_trust() -> Self {
        Self {
            trust_change: -0.5,
            respect_change: -0.3,
            affection_change: -0.2,
            familiarity_change: 0.0,
            tension_change: 0.4,
            event_type: "betrayed_trust".to_string(),
        }
    }

    pub fn shared_secret() -> Self {
        Self {
            trust_change: 0.3,
            respect_change: 0.1,
            affection_change: 0.2,
            familiarity_change: 0.15,
            tension_change: 0.0,
            event_type: "shared_secret".to_string(),
        }
    }

    pub fn completed_quest_together() -> Self {
        Self {
            trust_change: 0.15,
            respect_change: 0.2,
            affection_change: 0.1,
            familiarity_change: 0.1,
            tension_change: -0.05,
            event_type: "completed_quest_together".to_string(),
        }
    }

    pub fn disagreed_publicly() -> Self {
        Self {
            trust_change: -0.1,
            respect_change: -0.2,
            affection_change: -0.05,
            familiarity_change: 0.05,
            tension_change: 0.2,
            event_type: "disagreed_publicly".to_string(),
        }
    }

    pub fn gift_given() -> Self {
        Self {
            trust_change: 0.1,
            respect_change: 0.05,
            affection_change: 0.2,
            familiarity_change: 0.05,
            tension_change: -0.05,
            event_type: "gift_given".to_string(),
        }
    }

    pub fn romantic_gesture() -> Self {
        Self {
            trust_change: 0.1,
            respect_change: 0.0,
            affection_change: 0.3,
            familiarity_change: 0.1,
            tension_change: 0.0,
            event_type: "romantic_gesture".to_string(),
        }
    }

    pub fn calculate_total_impact(&self) -> f32 {
        (self.trust_change.abs() +
            self.respect_change.abs() +
            self.affection_change.abs() +
            self.tension_change.abs()) / 4.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub event_type: String,
    pub impact: f32,
    pub timestamp: f64,
}

// Global reputation system
#[derive(Debug, Clone)]
pub struct GlobalReputation {
    pub reputations: HashMap<Entity, HashMap<String, ReputationEntry>>,
    pub faction_standings: HashMap<Entity, HashMap<String, f32>>,
}

impl GlobalReputation {
    pub fn new() -> Self {
        Self {
            reputations: HashMap::new(),
            faction_standings: HashMap::new(),
        }
    }

    pub fn update_reputation(&mut self, entity: Entity, group: &str, delta: f32) {
        let entity_reps = self.reputations.entry(entity).or_insert_with(HashMap::new);
        let entry = entity_reps.entry(group.to_string()).or_insert_with(|| {
            ReputationEntry {
                value: 0.0,
                events: Vec::new(),
            }
        });

        entry.value = (entry.value + delta).clamp(-1.0, 1.0);
        entry.events.push(ReputationEvent {
            delta,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        });

        // Keep event history manageable
        if entry.events.len() > 10 {
            entry.events.remove(0);
        }
    }

    pub fn get_reputation(&self, entity: Entity, group: &str) -> f32 {
        self.reputations
            .get(&entity)
            .and_then(|reps| reps.get(group))
            .map(|entry| entry.value)
            .unwrap_or(0.0)
    }

    pub fn get_reputation_level(&self, entity: Entity, group: &str) -> ReputationLevel {
        let value = self.get_reputation(entity, group);
        ReputationLevel::from_value(value)
    }

    pub fn update_faction_standing(&mut self, entity: Entity, faction: &str, delta: f32) {
        let standings = self.faction_standings.entry(entity).or_insert_with(HashMap::new);
        let current = standings.get(faction).copied().unwrap_or(0.0);
        standings.insert(faction.to_string(), (current + delta).clamp(-1.0, 1.0));
    }

    pub fn get_faction_standing(&self, entity: Entity, faction: &str) -> f32 {
        self.faction_standings
            .get(&entity)
            .and_then(|standings| standings.get(faction))
            .copied()
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct ReputationEntry {
    pub value: f32,
    pub events: Vec<ReputationEvent>,
}

#[derive(Debug, Clone)]
pub struct ReputationEvent {
    pub delta: f32,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum ReputationLevel {
    Despised,   // -1.0 to -0.8
    Hated,      // -0.8 to -0.6
    Disliked,   // -0.6 to -0.3
    Neutral,    // -0.3 to 0.3
    Liked,      // 0.3 to 0.6
    Respected,  // 0.6 to 0.8
    Revered,    // 0.8 to 1.0
}

impl ReputationLevel {
    pub fn from_value(value: f32) -> Self {
        match value {
            x if x <= -0.8 => ReputationLevel::Despised,
            x if x <= -0.6 => ReputationLevel::Hated,
            x if x <= -0.3 => ReputationLevel::Disliked,
            x if x < 0.3 => ReputationLevel::Neutral,
            x if x < 0.6 => ReputationLevel::Liked,
            x if x < 0.8 => ReputationLevel::Respected,
            _ => ReputationLevel::Revered,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ReputationLevel::Despised => "Despised",
            ReputationLevel::Hated => "Hated",
            ReputationLevel::Disliked => "Disliked",
            ReputationLevel::Neutral => "Neutral",
            ReputationLevel::Liked => "Liked",
            ReputationLevel::Respected => "Respected",
            ReputationLevel::Revered => "Revered",
        }
    }
}

// Import OrderedFloat for dijkstra algorithm
use ordered_float::OrderedFloat;

// Relationship analysis tools
pub struct RelationshipAnalyzer;

impl RelationshipAnalyzer {
    pub fn analyze_social_network(graph: &RelationshipGraph, entity: Entity) -> SocialNetworkAnalysis {
        let influence = graph.calculate_influence(entity);
        let connections = graph.entity_to_node.get(&entity)
            .map(|&node| graph.graph.neighbors(node).count())
            .unwrap_or(0);

        let relationship_quality = graph.entity_to_node.get(&entity)
            .map(|&node| {
                let edges: Vec<_> = graph.graph.edges(node).collect();
                if edges.is_empty() {
                    0.0
                } else {
                    edges.iter()
                        .map(|edge| edge.weight().relationship.get_overall_sentiment())
                        .sum::<f32>() / edges.len() as f32
                }
            })
            .unwrap_or(0.0);

        SocialNetworkAnalysis {
            influence,
            connection_count: connections,
            average_relationship_quality: relationship_quality,
            social_reach: Self::calculate_social_reach(graph, entity),
        }
    }

    fn calculate_social_reach(graph: &RelationshipGraph, entity: Entity) -> usize {
        // Calculate how many entities can be reached within 2 degrees
        if let Some(&node) = graph.entity_to_node.get(&entity) {
            let mut reached = std::collections::HashSet::new();

            // First degree connections
            for neighbor in graph.graph.neighbors(node) {
                reached.insert(neighbor);

                // Second degree connections
                for second_neighbor in graph.graph.neighbors(neighbor) {
                    reached.insert(second_neighbor);
                }
            }

            reached.len()
        } else {
            0
        }
    }

    pub fn predict_relationship_change(
        current: &Relationship,
        event_type: &str,
        context: &RelationshipContext,
    ) -> RelationshipDelta {
        // AI-enhanced relationship prediction would go here
        // For now, use rule-based prediction

        let base_delta = match event_type {
            "combat_assistance" => RelationshipDelta::helped_in_combat(),
            "betrayal" => RelationshipDelta::betrayed_trust(),
            "gift" => RelationshipDelta::gift_given(),
            "quest_completion" => RelationshipDelta::completed_quest_together(),
            _ => RelationshipDelta {
                trust_change: 0.0,
                respect_change: 0.0,
                affection_change: 0.0,
                familiarity_change: 0.01,
                tension_change: 0.0,
                event_type: event_type.to_string(),
            },
        };

        // Modify based on context
        Self::apply_context_modifiers(base_delta, context)
    }

    fn apply_context_modifiers(
        mut delta: RelationshipDelta,
        context: &RelationshipContext,
    ) -> RelationshipDelta {
        // Personality compatibility affects relationship changes
        let compatibility = context.personality_compatibility;
        delta.trust_change *= compatibility;
        delta.affection_change *= compatibility;

        // Stress situations amplify changes
        if context.stress_level > 0.7 {
            delta.trust_change *= 1.5;
            delta.tension_change *= 1.3;
        }

        // Public vs private events have different impacts
        if context.is_public {
            delta.respect_change *= 1.2;
            delta.tension_change *= 1.1;
        }

        delta
    }
}

#[derive(Debug, Clone)]
pub struct SocialNetworkAnalysis {
    pub influence: f32,
    pub connection_count: usize,
    pub average_relationship_quality: f32,
    pub social_reach: usize,
}

#[derive(Debug, Clone)]
pub struct RelationshipContext {
    pub personality_compatibility: f32,
    pub stress_level: f32,
    pub is_public: bool,
    pub location_type: String,
    pub witnesses: Vec<Entity>,
}

impl RelationshipContext {
    pub fn new() -> Self {
        Self {
            personality_compatibility: 1.0,
            stress_level: 0.0,
            is_public: false,
            location_type: "neutral".to_string(),
            witnesses: Vec::new(),
        }
    }

    pub fn with_compatibility(mut self, compatibility: f32) -> Self {
        self.personality_compatibility = compatibility;
        self
    }

    pub fn with_stress(mut self, stress: f32) -> Self {
        self.stress_level = stress;
        self
    }

    pub fn in_public(mut self) -> Self {
        self.is_public = true;
        self
    }
}