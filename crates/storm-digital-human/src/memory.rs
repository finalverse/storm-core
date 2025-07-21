// File: crates/storm-digital-human/src/memory.rs
// Description: Memory system for NPCs - Fully corrected version
// Manages short-term and long-term memories with importance scoring and proper borrowing

use serde::{Serialize, Deserialize};
use std::collections::{VecDeque, HashMap, BTreeMap, HashSet};
use uuid::Uuid;
use crate::{emotion::Emotion, behavior::{Entity, Vec3}};
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
        // Calculate importance first to avoid borrowing conflicts
        let importance = Self::calculate_importance_static(&event, &self.short_term);

        let memory = Memory {
            id: Uuid::new_v4(),
            event: event.clone(),
            importance,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            recall_count: 0,
            associations: vec![],
            decay_rate: 0.1,
        };

        // Store in different memory systems
        self.short_term.add(memory.clone());
        self.episodic_memory.add_episode(&memory);

        if importance > 0.7 {
            self.semantic_memory.extract_knowledge(&memory);
        }

        if importance > 0.6 {
            self.long_term.add(memory.clone());
        }

        self.working_memory.update_context(&event);
    }

    // Static method to calculate importance without borrowing self
    fn calculate_importance_static(event: &MemoryEvent, short_term: &ShortTermMemory) -> f32 {
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

        // Apply modifiers
        let context_modifier = Self::calculate_context_modifier_static(event);
        let recency_modifier = 1.1; // Recent events are slightly more important
        let uniqueness_modifier = Self::calculate_uniqueness_modifier_static(event, short_term);

        (base_importance * context_modifier * recency_modifier * uniqueness_modifier)
            .clamp(0.0, 1.0)
    }

    fn calculate_context_modifier_static(event: &MemoryEvent) -> f32 {
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

    fn calculate_uniqueness_modifier_static(event: &MemoryEvent, short_term: &ShortTermMemory) -> f32 {
        let similar_count = short_term.entries.iter()
            .filter(|m| Self::are_events_similar_static(&m.event, event))
            .count();

        if similar_count == 0 {
            1.2
        } else {
            1.0 / (similar_count as f32 + 1.0)
        }
    }

    fn are_events_similar_static(event1: &MemoryEvent, event2: &MemoryEvent) -> bool {
        std::mem::discriminant(&event1.memory_type) ==
            std::mem::discriminant(&event2.memory_type) &&
            event1.description.len() >= 10 && event2.description.len() >= 10 &&
            event1.description.contains(&event2.description[..10])
    }

    pub fn recall(&mut self, query: &MemoryQuery) -> Vec<Memory> {
        // Collect results from all memory systems
        let mut results = Vec::new();

        // Search each memory system separately to avoid borrowing conflicts
        results.extend(self.search_short_term(query));
        results.extend(self.search_long_term(query));
        results.extend(self.search_episodic(query));

        // Update recall statistics for found memories
        self.update_recall_statistics(&results);

        // Sort by relevance
        results.sort_by(|a, b| {
            let relevance_a = Self::calculate_memory_relevance_static(a, query);
            let relevance_b = Self::calculate_memory_relevance_static(b, query);
            relevance_b.partial_cmp(&relevance_a).unwrap()
        });

        results
    }

    fn search_short_term(&self, query: &MemoryQuery) -> Vec<Memory> {
        self.short_term.search(query)
    }

    fn search_long_term(&self, query: &MemoryQuery) -> Vec<Memory> {
        self.long_term.search(query)
    }

    fn search_episodic(&self, query: &MemoryQuery) -> Vec<Memory> {
        self.episodic_memory.search(query)
    }

    fn update_recall_statistics(&mut self, memory_ids: &[Memory]) {
        // Update recall counts in short-term memory
        for memory in &mut self.short_term.entries {
            if memory_ids.iter().any(|m| m.id == memory.id) {
                memory.recall_count += 1;
                memory.importance = (memory.importance * 1.1).min(1.0);
            }
        }

        // Update recall counts in long-term memory
        for memory in &mut self.long_term.memories {
            if memory_ids.iter().any(|m| m.id == memory.id) {
                memory.recall_count += 1;
                memory.importance = (memory.importance * 1.1).min(1.0);
            }
        }
    }

    fn calculate_memory_relevance_static(memory: &Memory, query: &MemoryQuery) -> f32 {
        let mut relevance = memory.importance;

        if let Some(query_type) = &query.memory_type {
            if std::mem::discriminant(&memory.event.memory_type) ==
                std::mem::discriminant(query_type) {
                relevance *= 1.5;
            }
        }

        if let Some(query_entity) = query.entity {
            if memory.event.entities.contains(&query_entity) {
                relevance *= 1.3;
            }
        }

        for keyword in &query.keywords {
            if memory.event.description.to_lowercase().contains(&keyword.to_lowercase()) {
                relevance *= 1.2;
            }
        }

        relevance
    }

    pub fn decay_memories(&mut self, delta_time: f32) {
        // Decay memories in each system separately
        self.decay_short_term_memories(delta_time);
        self.decay_long_term_memories(delta_time);
        self.promote_memories();
        self.semantic_memory.consolidate();
    }

    fn decay_short_term_memories(&mut self, delta_time: f32) {
        for memory in &mut self.short_term.entries {
            memory.importance *= 1.0 - (memory.decay_rate * delta_time);
        }
        self.short_term.entries.retain(|memory| memory.importance > 0.1);
    }

    fn decay_long_term_memories(&mut self, delta_time: f32) {
        let original_len = self.long_term.memories.len();

        for memory in &mut self.long_term.memories {
            memory.importance *= 1.0 - (memory.decay_rate * delta_time * 0.1);
        }

        self.long_term.memories.retain(|memory| memory.importance > 0.05);

        if self.long_term.memories.len() != original_len {
            self.long_term.rebuild_indices();
        }
    }

    fn promote_memories(&mut self) {
        let candidates: Vec<Memory> = self.short_term.entries.iter()
            .filter(|memory| memory.importance > 0.6 && memory.recall_count > 2)
            .cloned()
            .collect();

        for memory in candidates {
            self.long_term.add(memory);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub event: MemoryEvent,
    pub importance: f32,
    pub timestamp: f64,
    pub recall_count: u32,
    pub associations: Vec<Uuid>,
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
        coordinates: Option<Vec3>,
    },
    Procedural {
        success_rate: f32,
        skill: String,
    },
}

#[derive(Debug, Clone)]
pub enum MemoryImportance {
    Low,
    Medium,
    High,
    Critical,
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
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(memory);
    }

    pub fn search(&self, query: &MemoryQuery) -> Vec<Memory> {
        self.entries.iter()
            .filter(|memory| Self::matches_query_static(memory, query))
            .cloned()
            .collect()
    }

    fn matches_query_static(memory: &Memory, query: &MemoryQuery) -> bool {
        if let Some(time_range) = &query.time_range {
            if memory.timestamp < time_range.start || memory.timestamp > time_range.end {
                return false;
            }
        }

        if let Some(min_importance) = query.min_importance {
            if memory.importance < min_importance {
                return false;
            }
        }

        if let Some(entity) = query.entity {
            if !memory.event.entities.contains(&entity) {
                return false;
            }
        }

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
}

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
        let candidate_ids = self.indices.find_candidates(query);

        self.memories.iter()
            .filter(|m| candidate_ids.contains(&m.id))
            .cloned()
            .collect()
    }

    pub fn rebuild_indices(&mut self) {
        self.indices = MemoryIndices::new();
        for memory in &self.memories {
            self.indices.index_memory(memory);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryIndices {
    pub entity_index: HashMap<Entity, Vec<Uuid>>,
    pub type_index: HashMap<String, Vec<Uuid>>,
    pub keyword_index: HashMap<String, Vec<Uuid>>,
    pub time_index: BTreeMap<u64, Vec<Uuid>>,
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

        // Index by keywords
        let words: Vec<String> = memory.event.description
            .split_whitespace()
            .chain(memory.event.tags.iter().map(|s| s.as_str()))
            .filter(|word| word.len() > 2)
            .map(|word| word.to_lowercase())
            .collect();

        for word in words {
            self.keyword_index.entry(word)
                .or_insert_with(Vec::new)
                .push(memory_id);
        }

        // Index by time buckets
        let time_bucket = (memory.timestamp / 3600.0) as u64;
        self.time_index.entry(time_bucket)
            .or_insert_with(Vec::new)
            .push(memory_id);
    }

    pub fn find_candidates(&self, query: &MemoryQuery) -> HashSet<Uuid> {
        let mut candidates = HashSet::new();
        let mut first_constraint = true;

        if let Some(entity) = query.entity {
            if let Some(entity_memories) = self.entity_index.get(&entity) {
                if first_constraint {
                    candidates.extend(entity_memories);
                    first_constraint = false;
                } else {
                    candidates.retain(|id| entity_memories.contains(id));
                }
            } else {
                return candidates;
            }
        }

        for keyword in &query.keywords {
            if let Some(keyword_memories) = self.keyword_index.get(&keyword.to_lowercase()) {
                if first_constraint {
                    candidates.extend(keyword_memories);
                    first_constraint = false;
                } else {
                    candidates.retain(|id| keyword_memories.contains(id));
                }
            } else {
                return candidates;
            }
        }

        if first_constraint {
            for ids in self.entity_index.values() {
                candidates.extend(ids);
            }
        }

        candidates
    }
}

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
            max_items: 7,
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

        if self.current_context.len() > self.max_items {
            self.current_context.remove(0);
        }

        for entity in &event.entities {
            if !self.attention_focus.contains(entity) {
                self.attention_focus.push(*entity);
            }
        }

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
        let should_continue = if let Some(ref episode) = self.current_episode {
            Self::is_episode_continuation_static(memory, episode)
        } else {
            false
        };

        if should_continue {
            if let Some(ref mut episode) = self.current_episode {
                episode.memories.push(memory.id);
                episode.end_time = memory.timestamp;
            }
            return;
        }

        // End current episode if it exists
        if let Some(episode) = self.current_episode.take() {
            self.episodes.push(episode);
        }

        // Start new episode
        self.current_episode = Some(Episode {
            id: Uuid::new_v4(),
            memories: vec![memory.id],
            start_time: memory.timestamp,
            end_time: memory.timestamp,
            theme: Self::extract_episode_theme_static(memory),
        });
    }

    fn is_episode_continuation_static(memory: &Memory, episode: &Episode) -> bool {
        let time_gap = memory.timestamp - episode.end_time;
        time_gap < 3600.0 && memory.event.description.contains(&episode.theme)
    }

    fn extract_episode_theme_static(memory: &Memory) -> String {
        let words: Vec<&str> = memory.event.description.split_whitespace().collect();
        if words.len() > 2 {
            format!("{} {}", words[0], words[1])
        } else {
            memory.event.description.clone()
        }
    }

    pub fn search(&self, _query: &MemoryQuery) -> Vec<Memory> {
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
        // Placeholder for knowledge consolidation logic
    }

    pub fn get_knowledge(&self, topic: &str) -> Option<&KnowledgeNode> {
        self.knowledge_base.get(topic)
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub topic: String,
    pub confidence: f32,
    pub evidence: Vec<Uuid>,
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
    IsA,
    PartOf,
    Causes,
    Similar,
}

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