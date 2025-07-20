// File: crates/storm-digital-human/src/dialogue.rs
// Description: Dialogue system with AI integration
// Manages conversations, context, and dynamic responses

use storm_ai::prelude::*;
use crate::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub struct DialogueEngine {
    pub conversation_history: Arc<RwLock<ConversationHistory>>,
    pub context_analyzer: ContextAnalyzer,
    pub response_generator: ResponseGenerator,
    pub dialogue_memory: DialogueMemory,
}

impl DialogueEngine {
    pub fn new(personality: &PersonalityMatrix) -> Self {
        Self {
            conversation_history: Arc::new(RwLock::new(ConversationHistory::new())),
            context_analyzer: ContextAnalyzer::new(),
            response_generator: ResponseGenerator::new(personality.clone()),
            dialogue_memory: DialogueMemory::new(),
        }
    }

    pub async fn process_input(
        &mut self,
        input: &str,
        speaker: Entity,
        context: &DialogueContext,
    ) -> DialogueResponse {
        // Analyze input
        let analysis = self.context_analyzer.analyze(input, context);

        // Update conversation history
        {
            let mut history = self.conversation_history.write().await;
            history.add_entry(DialogueEntry {
                speaker,
                text: input.to_string(),
                intent: analysis.intent.clone(),
                emotion: analysis.emotion,
                timestamp: context.current_time,
            });
        }

        // Generate response
        let response = self.response_generator.generate(
            &analysis,
            context,
            &self.dialogue_memory,
        ).await;

        // Update dialogue memory
        self.dialogue_memory.update(&analysis, &response);

        response
    }
}

#[derive(Debug, Clone)]
pub struct DialogueContext {
    pub location: String,
    pub participants: Vec<Entity>,
    pub topic: Option<String>,
    pub mood: ConversationMood,
    pub relationship_level: f32,
    pub current_time: f32,
    pub world_state: WorldContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationMood {
    Friendly,
    Formal,
    Tense,
    Intimate,
    Playful,
    Serious,
}

#[derive(Debug)]
pub struct ConversationHistory {
    entries: VecDeque<DialogueEntry>,
    max_entries: usize,
}

impl ConversationHistory {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 50,
        }
    }

    pub fn add_entry(&mut self, entry: DialogueEntry) {
        self.entries.push_back(entry);

        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&DialogueEntry> {
        self.entries.iter()
            .rev()
            .take(count)
            .collect()
    }

    pub fn find_by_topic(&self, topic: &str) -> Vec<&DialogueEntry> {
        self.entries.iter()
            .filter(|e| e.text.contains(topic))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DialogueEntry {
    pub speaker: Entity,
    pub text: String,
    pub intent: DialogueIntent,
    pub emotion: Option<Emotion>,
    pub timestamp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueIntent {
    Greeting,
    Question,
    Statement,
    Request,
    Offer,
    Acceptance,
    Rejection,
    Emotional,
    Farewell,
}

pub struct ContextAnalyzer {
    intent_classifier: IntentClassifier,
    emotion_detector: EmotionDetector,
}

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self {
            intent_classifier: IntentClassifier::new(),
            emotion_detector: EmotionDetector::new(),
        }
    }

    pub fn analyze(&self, input: &str, context: &DialogueContext) -> DialogueAnalysis {
        DialogueAnalysis {
            intent: self.intent_classifier.classify(input),
            emotion: self.emotion_detector.detect(input),
            topics: self.extract_topics(input),
            sentiment: self.analyze_sentiment(input),
        }
    }

    fn extract_topics(&self, input: &str) -> Vec<String> {
        // Simple keyword extraction for now
        let keywords = ["quest", "help", "trade", "story", "echo", "song"];

        keywords.iter()
            .filter(|k| input.to_lowercase().contains(*k))
            .map(|k| k.to_string())
            .collect()
    }

    fn analyze_sentiment(&self, input: &str) -> f32 {
        // Simple sentiment analysis
        let positive_words = ["good", "great", "thank", "please", "happy", "love"];
        let negative_words = ["bad", "hate", "angry", "sad", "terrible", "awful"];

        let positive_count = positive_words.iter()
            .filter(|w| input.to_lowercase().contains(*w))
            .count() as f32;

        let negative_count = negative_words.iter()
            .filter(|w| input.to_lowercase().contains(*w))
            .count() as f32;

        if positive_count + negative_count == 0.0 {
            0.0
        } else {
            (positive_count - negative_count) / (positive_count + negative_count)
        }
    }
}

pub struct IntentClassifier;

impl IntentClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn classify(&self, input: &str) -> DialogueIntent {
        let lower = input.to_lowercase();

        if lower.contains('?') || lower.starts_with("what") || lower.starts_with("how") {
            DialogueIntent::Question
        } else if lower.contains("hello") || lower.contains("hi ") || lower.contains("greetings") {
            DialogueIntent::Greeting
        } else if lower.contains("goodbye") || lower.contains("farewell") || lower.contains("bye") {
            DialogueIntent::Farewell
        } else if lower.contains("please") || lower.contains("could you") || lower.contains("would you") {
            DialogueIntent::Request
        } else if lower.contains("yes") || lower.contains("okay") || lower.contains("sure") {
            DialogueIntent::Acceptance
        } else if lower.contains("no") || lower.contains("never") || lower.contains("refuse") {
            DialogueIntent::Rejection
        } else {
            DialogueIntent::Statement
        }
    }
}

pub struct EmotionDetector;

impl EmotionDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, input: &str) -> Option<Emotion> {
        let lower = input.to_lowercase();

        if lower.contains("happy") || lower.contains("joy") || lower.contains("excited") {
            Some(Emotion::Joy)
        } else if lower.contains("sad") || lower.contains("depressed") || lower.contains("unhappy") {
            Some(Emotion::Sadness)
        } else if lower.contains("angry") || lower.contains("furious") || lower.contains("mad") {
            Some(Emotion::Anger)
        } else if lower.contains("scared") || lower.contains("afraid") || lower.contains("terrified") {
            Some(Emotion::Fear)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DialogueAnalysis {
    pub intent: DialogueIntent,
    pub emotion: Option<Emotion>,
    pub topics: Vec<String>,
    pub sentiment: f32,
}

pub struct ResponseGenerator {
    personality: PersonalityMatrix,
    response_templates: ResponseTemplates,
    ai_client: Option<AIClient>,
}

impl ResponseGenerator {
    pub fn new(personality: PersonalityMatrix) -> Self {
        Self {
            personality,
            response_templates: ResponseTemplates::new(),
            ai_client: None,
        }
    }

    pub fn with_ai(mut self, ai_client: AIClient) -> Self {
        self.ai_client = Some(ai_client);
        self
    }

    pub async fn generate(
        &self,
        analysis: &DialogueAnalysis,
        context: &DialogueContext,
        memory: &DialogueMemory,
    ) -> DialogueResponse {
        // Check if AI generation is available and appropriate
        if let Some(ai) = &self.ai_client {
            if self.should_use_ai(analysis, context) {
                if let Ok(ai_response) = self.generate_ai_response(ai, analysis, context, memory).await {
                    return ai_response;
                }
            }
        }

        // Fallback to template-based response
        self.generate_template_response(analysis, context, memory)
    }

    fn should_use_ai(&self, analysis: &DialogueAnalysis, context: &DialogueContext) -> bool {
        // Use AI for complex or emotional responses
        analysis.topics.len() > 1 ||
            analysis.emotion.is_some() ||
            context.relationship_level > 0.7
    }

    async fn generate_ai_response(
        &self,
        ai: &AIClient,
        analysis: &DialogueAnalysis,
        context: &DialogueContext,
        memory: &DialogueMemory,
    ) -> Result<DialogueResponse> {
        let prompt = self.build_ai_prompt(analysis, context, memory);

        let ai_text = ai.generate(prompt, GenerationParams {
            max_tokens: 150,
            temperature: 0.8,
            ..Default::default()
        }).await
            .map_err(|e| DigitalHumanError::AIError(e.to_string()))?;

        Ok(DialogueResponse {
            text: ai_text,
            emotion: analysis.emotion,
            action: None,
            next_state: ConversationState::Continuing,
        })
    }

    fn build_ai_prompt(
        &self,
        analysis: &DialogueAnalysis,
        context: &DialogueContext,
        memory: &DialogueMemory,
    ) -> String {
        format!(
            "You are an NPC with these personality traits: {:?}. \
            The player said something with intent '{:?}' about topics: {:?}. \
            Current mood: {:?}. Relationship level: {}. \
            Recent topics discussed: {:?}. \
            Generate a natural, in-character response that fits the fantasy setting.",
            self.personality.traits.keys().collect::<Vec<_>>(),
            analysis.intent,
            analysis.topics,
            context.mood,
            context.relationship_level,
            memory.recent_topics()
        )
    }

    fn generate_template_response(
        &self,
        analysis: &DialogueAnalysis,
        context: &DialogueContext,
        memory: &DialogueMemory,
    ) -> DialogueResponse {
        let template = self.response_templates.get_template(
            &analysis.intent,
            &self.personality,
            context,
        );

        let text = self.personalize_template(template, analysis, context);

        DialogueResponse {
            text,
            emotion: self.calculate_response_emotion(analysis),
            action: self.determine_action(analysis, context),
            next_state: self.determine_next_state(analysis),
        }
    }

    fn personalize_template(
        &self,
        template: &str,
        analysis: &DialogueAnalysis,
        context: &DialogueContext,
    ) -> String {
        // Add personality-based modifications
        let mut response = template.to_string();

        // Add personality flavor
        if self.personality.get_trait_value(PersonalityTrait::Extraversion) > 0.7 {
            response = format!("{} Let me tell you more!", response);
        } else if self.personality.get_trait_value(PersonalityTrait::Extraversion) < 0.3 {
            response = format!("{}...", response);
        }

        response
    }

    fn calculate_response_emotion(&self, analysis: &DialogueAnalysis) -> Option<Emotion> {
        // Mirror positive emotions, respond carefully to negative ones
        match analysis.emotion {
            Some(Emotion::Joy) => Some(Emotion::Joy),
            Some(Emotion::Sadness) => Some(Emotion::Gratitude),
            Some(Emotion::Anger) if self.personality.get_trait_value(PersonalityTrait::Agreeableness) > 0.6 => {
                Some(Emotion::Trust)
            }
            _ => None,
        }
    }

    fn determine_action(&self, analysis: &DialogueAnalysis, context: &DialogueContext) -> Option<NPCAction> {
        match analysis.intent {
            DialogueIntent::Greeting => Some(NPCAction::Animation("wave".to_string())),
            DialogueIntent::Farewell => Some(NPCAction::Animation("bow".to_string())),
            _ => None,
        }
    }

    fn determine_next_state(&self, analysis: &DialogueAnalysis) -> ConversationState {
        match analysis.intent {
            DialogueIntent::Farewell => ConversationState::Ending,
            DialogueIntent::Question => ConversationState::WaitingForMore,
            _ => ConversationState::Continuing,
        }
    }
}

pub struct ResponseTemplates {
    templates: HashMap<String, Vec<String>>,
}

impl ResponseTemplates {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        templates.insert("greeting_friendly".to_string(), vec![
            "Hello there, friend!".to_string(),
            "Greetings, traveler!".to_string(),
            "Well met!".to_string(),
        ]);

        templates.insert("greeting_formal".to_string(), vec![
            "Greetings.".to_string(),
            "Good day to you.".to_string(),
        ]);

        templates.insert("question_response".to_string(), vec![
            "That's an interesting question...".to_string(),
            "Let me think about that...".to_string(),
            "I believe...".to_string(),
        ]);

        Self { templates }
    }

    pub fn get_template(
        &self,
        intent: &DialogueIntent,
        personality: &PersonalityMatrix,
        context: &DialogueContext,
    ) -> &str {
        let key = match (intent, context.mood) {
            (DialogueIntent::Greeting, ConversationMood::Friendly) => "greeting_friendly",
            (DialogueIntent::Greeting, ConversationMood::Formal) => "greeting_formal",
            (DialogueIntent::Question, _) => "question_response",
            _ => "greeting_friendly",
        };

        self.templates.get(key)
            .and_then(|v| v.get(0))
            .map(|s| s.as_str())
            .unwrap_or("Hello.")
    }
}

#[derive(Debug)]
pub struct DialogueMemory {
    topics_discussed: VecDeque<(String, f32)>, // (topic, timestamp)
    important_facts: Vec<ImportantFact>,
    relationship_events: Vec<RelationshipEvent>,
}

impl DialogueMemory {
    pub fn new() -> Self {
        Self {
            topics_discussed: VecDeque::new(),
            important_facts: Vec::new(),
            relationship_events: Vec::new(),
        }
    }

    pub fn update(&mut self, analysis: &DialogueAnalysis, response: &DialogueResponse) {
        // Add new topics
        for topic in &analysis.topics {
            self.topics_discussed.push_back((topic.clone(), 0.0)); // timestamp would be real
        }

        // Keep size manageable
        while self.topics_discussed.len() > 20 {
            self.topics_discussed.pop_front();
        }
    }

    pub fn recent_topics(&self) -> Vec<String> {
        self.topics_discussed.iter()
            .map(|(topic, _)| topic.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ImportantFact {
    pub fact: String,
    pub source: Entity,
    pub confidence: f32,
    pub timestamp: f32,
}

#[derive(Debug, Clone)]
pub struct RelationshipEvent {
    pub event_type: String,
    pub impact: f32,
    pub timestamp: f32,
}

#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub text: String,
    pub emotion: Option<Emotion>,
    pub action: Option<NPCAction>,
    pub next_state: ConversationState,
}

#[derive(Debug, Clone, Copy)]
pub enum ConversationState {
    Starting,
    Continuing,
    WaitingForMore,
    Ending,
}