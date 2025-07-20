// File: crates/storm-digital-human/src/memory.rs
// Description: Memory system for NPCs
// Manages short-term and long-term memories with importance scoring

use serde::{Serialize, Deserialize};
use std::collections::{VecDeque, HashMap, BTreeMap};
use uuid::Uuid;
use ordered_float::OrderedFloat;
use storm_ecs::prelude::*;
use crate::emotion::Emotion;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NPCMemory {
    pub short_term: ShortTermMemory,
    pub long_term: LongTermMemory,
    pub working_memory: WorkingMemory,
    pub episodic_memory: EpisodicMemory,
    pub semantic_memory: SemanticMemory,
}

impl NPCMemory {
    pub fn new() -> Self {
        Self {
            short_term: ShortTermMemory::new(50),
            long_term: LongTermMemory::new(),
            working_memory: WorkingMemory::new(),
            episodic_memory: EpisodicMemory::new(),
            semantic_memory: SemanticMemory::new(),
        }
    }

    pub fn store_event(&mut self, event: MemoryEvent) {
        // Calculate importance using multiple factors
        let importance = self.calculate_importance(&event);

        let memory = Memory {
            id: Uuid::new_v4(),
            event,
            importance,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            recall_count: 0,
            associations: vec![],
            decay_rate: 0.1,
        };

        // Always store in short-term first
        self.short_term.add(memory.clone());

        // Store in episodic memory for context
        self.episodic_memory.add_episode(&memory);

        // Extract semantic knowledge if significant
        if importance > 0.7 {
            self.semantic_memory.extract_knowledge(&memory);
        }

        // Store in long-term if important enough
        if importance > 0.6 {
            self.long_term.add(memory);
        }

        // Update working memory context
        self.working_memory.update_context(&memory.event);
    }

    fn calculate_importance(&self, event: &MemoryEvent) -> f32 {
        let base_importance = match &event.memory_type {
            MemoryType::Social { relationship_impact, .. } => {
                0.5 + relationship_impact.abs() * 0.5
            }
            MemoryType::Emotional { intensity, .. } => {
                *intensity
            }
            MemoryType::Knowledge { relevance, .. } => {
                *relevance
            }
            MemoryType::Spatial { significance, .. } => {
                *significance
            }
            MemoryType::Procedural { success_rate, .. } => {
                if *success_rate > 0.8 { 0.7 } else { 0.3 }
            }
        };

        // Apply modifiers based on context
        let context_modifier = self.calculate_context_modifier(event);
        let recency_modifier = self.calculate_recency_modifier(event);
        let uniqueness_modifier = self.calculate_uniqueness_modifier(event);

        (base_importance * context_modifier * recency_modifier * uniqueness_modifier)
            .clamp(0.0, 1.0)
    }

    fn calculate_context_modifier(&self, event: &MemoryEvent) -> f32 {
        // Events during high emotional states are more memorable
        if let Some(emotion) = &event.associated_emotion {
            match emotion {
                Emotion::Fear | Emotion::Anger | Emotion::Joy => 1.2,
                Emotion::Love | Emotion::Awe => 1.3,
                _ => 1.0,
            }
        } else {
            1.0
        }
    }

    fn calculate_recency_modifier(&self, _event: &MemoryEvent) -> f32 {
        // Recent events are slightly more important
        1.1
    }

    fn calculate_uniqueness_modifier(&self, event: &MemoryEvent) -> f32 {
        // Check if similar events exist
        let similar_count = self.short_term.entries.iter()
            .filter(|m| self.are_events_similar(&m.event, event))
            .count();

        if similar_count == 0 {
            1.2 // Unique events are more memorable
        } else {
            1.0 / (similar_count as f32 + 1.0)
        }
    }

    fn are_events_similar(&self, event1: &MemoryEvent, event2: &MemoryEvent) -> bool {
        std::mem::discriminant(&event1.memory_type) ==
            std::mem::discriminant(&event2.memory_type) &&
            event1.description.contains(&event2.description[..event2.description.len().min(10)])
    }

    pub fn recall(&mut self, query: &MemoryQuery) -> Vec<Memory> {
        let mut results = Vec::new();

        // Search short-term memory
        results.extend(self.short_term.search(query));

        // Search long-term memory
        results.extend(self.long_term.search(query));

        // Search episodic memory for context
        results.extend(self.episodic_memory.search(query));

        // Update recall counts and strengthen memories
        for memory in &mut results {
            memory.recall_count += 1;
            // Memories that are recalled become stronger
            memory.importance = (memory.importance * 1.1).min(1.0);
        }

        // Sort by relevance and importance
        results.sort_by(|a, b| {
            let relevance_a = self.calculate_memory_relevance(a, query);
            let relevance_b = self.calculate_memory_relevance(b, query);
            relevance_b.partial_cmp(&relevance_a).unwrap()
        });

        results
    }

    fn calculate_memory_relevance(&self, memory: &Memory, query: &MemoryQuery) -> f32 {
        let mut relevance = memory.importance;

        // Boost relevance for matching types
        if let Some(query_type) = &query.memory_type {
            if std::mem::discriminant(&memory.event.memory_type) ==
                std::mem::discriminant(query_type) {
                relevance *= 1.5;
            }
        }

        // Boost relevance for matching entities
        if let Some(query_entity) = query.entity {
            if memory.event.entities.contains(&query_entity) {
                relevance *= 1.3;
            }
        }

        // Boost relevance for matching keywords
        for keyword in &query.keywords {
            if memory.event.description.to_lowercase().contains(&keyword.to_lowercase()) {
                relevance *= 1.2;
            }
        }

        relevance
    }

    pub fn consolidate_memories(&mut self, delta_time: f32) {
        // Decay memories over time
        self.short_term.decay_memories(delta_time);
        self.long_term.decay_memories(delta_time);

        // Move important short-term memories to long-term
        let mut to_promote = Vec::new();
        for memory in &self.short_term.entries {
            if memory.importance > 0.6 && memory.recall_count > 2 {
                to_promote.push(memory.clone());
            }
        }

        for memory in to_promote {
            self.long_term.add(memory);
        }

        // Consolidate semantic knowledge
        self.semantic_memory.consolidate();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub event: MemoryEvent,
    pub importance: f32,
    pub timestamp: f64,
    pub recall_count: u32,
    pub associations: Vec<Uuid>, // IDs of associated memories
    pub decay_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub description: String,
    pub memory_type: MemoryType,
    pub entities: Vec<Entity>,
    pub location: Option<String>,
    pub associated_emotion: Option<Emotion>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Social {
        relationship_impact: f32,
        participants: Vec<Entity>,
    },
    Emotional {
        intensity: f32,
        primary_emotion: Emotion,
    },
    Knowledge {
        relevance: f32,
        topic: String,
    },
    Spatial {
        significance: f32,
        coordinates: Option<storm_math::Vec3>,
    },
    Procedural {
        success_rate: f32,
        skill: String,
    },
}

#[derive(Debug, Clone)]
pub enum MemoryImportance {
    Low,      // 0.0 - 0.3
    Medium,   // 0.3 - 0.6
    High,     // 0.6 - 0.8
    Critical, // 0.8 - 1.0
}

impl From<f32> for MemoryImportance {
    fn from(value: f32) -> Self {
        match value {
            x if x < 0.3 => MemoryImportance::Low,
            x if x < 0.6 => MemoryImportance::Medium,
            x if x < 0.8 => MemoryImportance::High,
            _ => MemoryImportance::Critical,
        }
    }
}

// Short-term memory with limited capacity
#[derive(Debug, Clone)]
pub struct ShortTermMemory {
    pub entries: VecDeque<Memory>,
    pub capacity: usize,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, memory: Memory) {
        // Remove oldest if at capacity
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(memory);
    }

    pub fn search(&self, query: &MemoryQuery) -> Vec<Memory> {
        self.entries.iter()
            .filter(|memory| self.matches_query(memory, query))
            .cloned()
            .collect()
    }

    fn matches_query(&self, memory: &Memory, query: &MemoryQuery) -> bool {
        // Check time range
        if let Some(time_range) = &query.time_range {
            if memory.timestamp < time_range.start || memory.timestamp > time_range.end {
                return false;
            }
        }

        // Check importance threshold
        if let Some(min_importance) = query.min_importance {
            if memory.importance < min_importance {
                return false;
            }
        }

        // Check entity involvement
        if let Some(entity) = query.entity {
            if !memory.event.entities.contains(&entity) {
                return false;
            }
        }

        // Check keywords
        if !query.keywords.is_empty() {
            let description_lower = memory.event.description.to_lowercase();
            let matches = query.keywords.iter()
                .any(|keyword| description_lower.contains(&keyword.to_lowercase()));
            if !matches {
                return false;
            }
        }

        true
    }

    pub fn decay_memories(&mut self, delta_time: f32) {
        for memory in &mut self.entries {
            memory.importance *= 1.0 - (memory.decay_rate * delta_time);
        }

        // Remove memories that have decayed too much
        self.entries.retain(|memory| memory.importance > 0.1);
    }
}

// Long-term memory with indexed storage
#[derive(Debug, Clone)]
pub struct LongTermMemory {
    pub memories: Vec<Memory>,
    pub indices: MemoryIndices,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            indices: MemoryIndices::new(),
        }
    }

    pub fn add(&mut self, memory: Memory) {
        self.indices.index_memory(&memory);
        self.memories.push(memory);
    }

    pub fn search(&self, query: &MemoryQuery) -> Vec<Memory> {
        // Use indices for faster search
        let candidate_ids = self.indices.find_candidates(query);

        self.memories.iter()
            .filter(|m| candidate_ids.contains(&m.id))
            .cloned()
            .collect()
    }

    pub fn decay_memories(&mut self, delta_time: f32) {
        for memory in &mut self.memories {
            // Long-term memories decay slower
            memory.importance *= 1.0 - (memory.decay_rate * delta_time * 0.1);
        }

        // Remove completely faded memories
        let original_len = self.memories.len();
        self.memories.retain(|memory| memory.importance > 0.05);

        // Rebuild indices if memories were removed
        if self.memories.len() != original_len {
            self.indices = MemoryIndices::new();
            for memory in &self.memories {
                self.indices.index_memory(memory);
            }
        }
    }
}

// Memory indexing for efficient search
#[derive(Debug, Clone)]
pub struct MemoryIndices {
    pub entity_index: HashMap<Entity, Vec<Uuid>>,
    pub type_index: HashMap<String, Vec<Uuid>>,
    pub keyword_index: HashMap<String, Vec<Uuid>>,
    pub time_index: BTreeMap<u64, Vec<Uuid>>, // Timestamp buckets
}

impl MemoryIndices {
    pub fn new() -> Self {
        Self {
            entity_index: HashMap::new(),
            type_index: HashMap::new(),
            keyword_index: HashMap::new(),
            time_index: BTreeMap::new(),
        }
    }

    pub fn index_memory(&mut self, memory: &Memory) {
        let memory_id = memory.id;

        // Index by entities
        for entity in &memory.event.entities {
            self.entity_index.entry(*entity)
                .or_insert_with(Vec::new)
                .push(memory_id);
        }

        // Index by type
        let type_name = format!("{:?}", memory.event.memory_type);
        self.type_index.entry(type_name)
            .or_insert_with(Vec::new)
            .push(memory_id);

        // Index by keywords (from description and tags)
        let words: Vec<String> = memory.event.description
            .split_whitespace()
            .chain(memory.event.tags.iter().map(|s| s.as_str()))
            .filter(|word| word.len() > 2) // Skip short words
            .map(|word| word.to_lowercase())
            .collect();

        for word in words {
            self.keyword_index.entry(word)
                .or_insert_with(Vec::new)
                .push(memory_id);
        }

        // Index by time (in hour buckets)
        let time_bucket = (memory.timestamp / 3600.0) as u64; // Hour buckets
        self.time_index.entry(time_bucket)
            .or_insert_with(Vec::new)
            .push(memory_id);
    }

    pub fn find_candidates(&self, query: &MemoryQuery) -> std::collections::HashSet<Uuid> {
        let mut candidates = std::collections::HashSet::new();
        let mut first_constraint = true;

        // Filter by entity
        if let Some(entity) = query.entity {
            if let Some(entity_memories) = self.entity_index.get(&entity) {
                if first_constraint {
                    candidates.extend(entity_memories);
                    first_constraint = false;
                } else {
                    candidates.retain(|id| entity_memories.contains(id));
                }
            } else if first_constraint {
                return candidates; // No memories for this entity
            }
        }

        // Filter by keywords
        for keyword in &query.keywords {
            if let Some(keyword_memories) = self.keyword_index.get(&keyword.to_lowercase()) {
                if first_constraint {
                    candidates.extend(keyword_memories);
                    first_constraint = false;
                } else {
                    candidates.retain(|id| keyword_memories.contains(id));
                }
            } else if first_constraint {
                return candidates; // No memories for this keyword
            }
        }

        // If no constraints applied, return all memories
        if first_constraint {
            // Collect all memory IDs from any index
            for ids in self.entity_index.values() {
                candidates.extend(ids);
            }
        }

        candidates
    }
}

// Working memory for current context
#[derive(Debug, Clone)]
pub struct WorkingMemory {
    pub current_context: Vec<ContextItem>,
    pub attention_focus: Vec<Entity>,
    pub active_goals: Vec<String>,
    pub max_items: usize,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            current_context: Vec::new(),
            attention_focus: Vec::new(),
            active_goals: Vec::new(),
            max_items: 7, // Miller's rule: 7±2 items
        }
    }

    pub fn update_context(&mut self, event: &MemoryEvent) {
        let context_item = ContextItem {
            description: event.description.clone(),
            relevance: 1.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };

        self.current_context.push(context_item);

        // Keep only the most recent items
        if self.current_context.len() > self.max_items {
            self.current_context.remove(0);
        }

        // Update attention focus
        for entity in &event.entities {
            if !self.attention_focus.contains(entity) {
                self.attention_focus.push(*entity);
            }
        }

        // Keep attention focus limited
        if self.attention_focus.len() > 3 {
            self.attention_focus.remove(0);
        }
    }

    pub fn get_relevant_context(&self, keywords: &[String]) -> Vec<&ContextItem> {
        self.current_context.iter()
            .filter(|item| {
                keywords.iter().any(|keyword|
                    item.description.to_lowercase().contains(&keyword.to_lowercase())
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ContextItem {
    pub description: String,
    pub relevance: f32,
    pub timestamp: f64,
}

// Episodic memory for sequential experiences
#[derive(Debug, Clone)]
pub struct EpisodicMemory {
    pub episodes: Vec<Episode>,
    pub current_episode: Option<Episode>,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        Self {
            episodes: Vec::new(),
            current_episode: None,
        }
    }

    pub fn add_episode(&mut self, memory: &Memory) {
        // Check if this continues the current episode
        if let Some(ref mut episode) = self.current_episode {
            if self.is_episode_continuation(memory, episode) {
                episode.memories.push(memory.id);
                episode.end_time = memory.timestamp;
                return;
            } else {
                // End current episode and start new one
                self.episodes.push(episode.clone());
            }
        }

        // Start new episode
        self.current_episode = Some(Episode {
            id: Uuid::new_v4(),
            memories: vec![memory.id],
            start_time: memory.timestamp,
            end_time: memory.timestamp,
            theme: self.extract_episode_theme(memory),
        });
    }

    fn is_episode_continuation(&self, memory: &Memory, episode: &Episode) -> bool {
        // Episodes continue if they're within a reasonable time window
        // and share thematic elements
        let time_gap = memory.timestamp - episode.end_time;
        time_gap < 3600.0 && // Within 1 hour
            memory.event.description.contains(&episode.theme)
    }

    fn extract_episode_theme(&self, memory: &Memory) -> String {
        // Extract main theme from memory description
        let words: Vec<&str> = memory.event.description.split_whitespace().collect();
        if words.len() > 2 {
            format!("{} {}", words[0], words[1])
        } else {
            memory.event.description.clone()
        }
    }

    pub fn search(&self, query: &MemoryQuery) -> Vec<Memory> {
        // For now, return empty - would need access to full memory system
        // In full implementation, would search through episodes
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub id: Uuid,
    pub memories: Vec<Uuid>,
    pub start_time: f64,
    pub end_time: f64,
    pub theme: String,
}

// Semantic memory for general knowledge
#[derive(Debug, Clone)]
pub struct SemanticMemory {
    pub knowledge_base: HashMap<String, KnowledgeNode>,
    pub relationships: Vec<KnowledgeRelation>,
}

impl SemanticMemory {
    pub fn new() -> Self {
        Self {
            knowledge_base: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn extract_knowledge(&mut self, memory: &Memory) {
        // Extract semantic knowledge from significant memories
        match &memory.event.memory_type {
            MemoryType::Knowledge { topic, relevance, .. } => {
                let knowledge = KnowledgeNode {
                    topic: topic.clone(),
                    confidence: *relevance,
                    evidence: vec![memory.id],
                    last_updated: memory.timestamp,
                };
                self.knowledge_base.insert(topic.clone(), knowledge);
            }
            MemoryType::Procedural { skill, success_rate, .. } => {
                let knowledge = KnowledgeNode {
                    topic: format!("skill_{}", skill),
                    confidence: *success_rate,
                    evidence: vec![memory.id],
                    last_updated: memory.timestamp,
                };
                self.knowledge_base.insert(skill.clone(), knowledge);
            }
            _ => {}
        }
    }

    pub fn consolidate(&mut self) {
        // Consolidate related knowledge nodes
        // This would involve finding patterns and creating generalizations
        // Simplified implementation for now
    }

    pub fn get_knowledge(&self, topic: &str) -> Option<&KnowledgeNode> {
        self.knowledge_base.get(topic)
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub topic: String,
    pub confidence: f32,
    pub evidence: Vec<Uuid>, // Supporting memory IDs
    pub last_updated: f64,
}

#[derive(Debug, Clone)]
pub struct KnowledgeRelation {
    pub from: String,
    pub to: String,
    pub relation_type: RelationType,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub enum RelationType {
    IsA,      // Category relationship
    PartOf,   // Composition relationship
    Causes,   // Causal relationship
    Similar,  // Similarity relationship
}

// Memory query structure
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub keywords: Vec<String>,
    pub entity: Option<Entity>,
    pub memory_type: Option<MemoryType>,
    pub time_range: Option<TimeRange>,
    pub min_importance: Option<f32>,
    pub max_results: Option<usize>,
}

impl MemoryQuery {
    pub fn new() -> Self {
        Self {
            keywords: Vec::new(),
            entity: None,
            memory_type: None,
            time_range: None,
            min_importance: None,
            max_results: None,
        }
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }

    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
}

impl TimeRange {
    pub fn last_hours(hours: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            start: now - (hours * 3600.0),
            end: now,
        }
    }

    pub fn last_days(days: f64) -> Self {
        Self::last_hours(days * 24.0)
    }
}