// File: crates/storm-digital-human/src/emotion.rs
// Description: Emotional state and response system
// Manages complex emotional states and transitions

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use ordered_float::OrderedFloat;
use crate::behavior::{WorldContext, SongResonance};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    pub emotions: HashMap<Emotion, f32>,
    pub mood: Mood,
    pub emotional_memory: Vec<EmotionalMemory>,
    pub stability: f32,
}

impl EmotionalState {
    pub fn new() -> Self {
        let mut emotions = HashMap::new();
        for emotion in Emotion::all() {
            emotions.insert(emotion, 0.0);
        }

        Self {
            emotions,
            mood: Mood::Neutral,
            emotional_memory: Vec::new(),
            stability: 0.7,
        }
    }

    pub fn process_trigger(&mut self, trigger: &str, context: &WorldContext) -> EmotionalResponse {
        let mut response = EmotionalResponse::new();

        match trigger {
            "threat_detected" => {
                self.add_emotion(Emotion::Fear, 0.6);
                self.add_emotion(Emotion::Anger, 0.3);
                response.add_emotion(Emotion::Fear, 0.6);
            }
            "friend_greeting" => {
                self.add_emotion(Emotion::Joy, 0.4);
                self.add_emotion(Emotion::Trust, 0.2);
                response.add_emotion(Emotion::Joy, 0.4);
            }
            "quest_completed" => {
                self.add_emotion(Emotion::Joy, 0.5);
                self.add_emotion(Emotion::Pride, 0.3);
                response.add_emotion(Emotion::Pride, 0.5);
            }
            "betrayal" => {
                self.add_emotion(Emotion::Anger, 0.7);
                self.add_emotion(Emotion::Sadness, 0.5);
                self.add_emotion(Emotion::Trust, -0.8);
                response.add_emotion(Emotion::Anger, 0.7);
            }
            "beautiful_scenery" => {
                self.add_emotion(Emotion::Awe, 0.6);
                self.add_emotion(Emotion::Joy, 0.2);
                response.add_emotion(Emotion::Awe, 0.6);
            }
            "song_resonance" => {
                if let Some(resonance) = &context.player_resonance {
                    let intensity = resonance.harmony_level;
                    self.add_emotion(Emotion::Awe, intensity * 0.5);
                    self.add_emotion(Emotion::Joy, intensity * 0.3);
                    response.add_emotion(Emotion::Awe, intensity * 0.5);
                }
            }
            _ => {}
        }

        // Update mood based on emotional state
        self.update_mood();

        // Add to emotional memory
        self.emotional_memory.push(EmotionalMemory {
            trigger: trigger.to_string(),
            emotions: response.emotions.clone(),
            timestamp: context.time_of_day,
        });

        // Keep memory size reasonable
        if self.emotional_memory.len() > 100 {
            self.emotional_memory.remove(0);
        }

        response.intensity = self.calculate_intensity();
        response
    }

    pub fn add_emotion(&mut self, emotion: Emotion, delta: f32) {
        let current = self.emotions.get(&emotion).copied().unwrap_or(0.0);
        let new_value = (current + delta).clamp(-1.0, 1.0);
        self.emotions.insert(emotion, new_value);
    }

    pub fn decay_emotions(&mut self, delta_time: f32) {
        let decay_rate = 0.1 * delta_time;

        for value in self.emotions.values_mut() {
            if value.abs() > 0.01 {
                *value *= 1.0 - decay_rate;
            } else {
                *value = 0.0;
            }
        }
    }

    fn update_mood(&mut self) {
        let positive_sum: f32 = self.emotions.iter()
            .filter(|(e, _)| e.is_positive())
            .map(|(_, v)| v)
            .sum();

        let negative_sum: f32 = self.emotions.iter()
            .filter(|(e, _)| e.is_negative())
            .map(|(_, v)| v.abs())
            .sum();

        self.mood = if positive_sum > negative_sum + 0.3 {
            Mood::Positive
        } else if negative_sum > positive_sum + 0.3 {
            Mood::Negative
        } else {
            Mood::Neutral
        };
    }

    fn calculate_intensity(&self) -> f32 {
        let total: f32 = self.emotions.values()
            .map(|v| v.abs())
            .sum();

        (total / self.emotions.len() as f32).clamp(0.0, 1.0)
    }

    pub fn get_dominant_emotion(&self) -> Option<&Emotion> {
        self.emotions.iter()
            .max_by_key(|(_, v)| OrderedFloat(**v))
            .map(|(e, _)| e)
    }

    pub fn to_feature_vector(&self) -> Vec<f32> {
        let mut features = vec![];

        // Add emotion values in consistent order
        for emotion in Emotion::all() {
            features.push(self.emotions.get(&emotion).copied().unwrap_or(0.0));
        }

        // Add mood as one-hot encoding
        features.extend(match self.mood {
            Mood::Positive => vec![1.0, 0.0, 0.0],
            Mood::Neutral => vec![0.0, 1.0, 0.0],
            Mood::Negative => vec![0.0, 0.0, 1.0],
        });

        // Add stability
        features.push(self.stability);

        features
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Emotion {
    // Basic emotions
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,

    // Complex emotions
    Trust,
    Anticipation,
    Pride,
    Shame,
    Guilt,
    Envy,
    Gratitude,
    Awe,
    Love,
    Contempt,
}

impl Emotion {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Joy, Self::Sadness, Self::Anger, Self::Fear,
            Self::Surprise, Self::Disgust, Self::Trust, Self::Anticipation,
            Self::Pride, Self::Shame, Self::Guilt, Self::Envy,
            Self::Gratitude, Self::Awe, Self::Love, Self::Contempt,
        ]
    }

    pub fn is_positive(&self) -> bool {
        matches!(self,
            Self::Joy | Self::Trust | Self::Anticipation |
            Self::Pride | Self::Gratitude | Self::Awe | Self::Love
        )
    }

    pub fn is_negative(&self) -> bool {
        matches!(self,
            Self::Sadness | Self::Anger | Self::Fear |
            Self::Disgust | Self::Shame | Self::Guilt |
            Self::Envy | Self::Contempt
        )
    }

    pub fn opposite(&self) -> Option<Self> {
        match self {
            Self::Joy => Some(Self::Sadness),
            Self::Sadness => Some(Self::Joy),
            Self::Anger => Some(Self::Gratitude),
            Self::Fear => Some(Self::Trust),
            Self::Love => Some(Self::Contempt),
            Self::Pride => Some(Self::Shame),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mood {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalMemory {
    pub trigger: String,
    pub emotions: HashMap<Emotion, f32>,
    pub timestamp: f32,
}

pub struct EmotionalResponse {
    pub emotions: HashMap<Emotion, f32>,
    pub intensity: f32,
}

impl EmotionalResponse {
    pub fn new() -> Self {
        Self {
            emotions: HashMap::new(),
            intensity: 0.0,
        }
    }

    pub fn add_emotion(&mut self, emotion: Emotion, value: f32) {
        self.emotions.insert(emotion, value);
    }

    pub fn dominant_emotion(&self) -> Option<Emotion> {
        self.emotions.iter()
            .max_by_key(|(_, v)| OrderedFloat(**v))
            .map(|(e, _)| *e)
    }
}