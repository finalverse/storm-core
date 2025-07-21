// File: crates/storm-digital-human/src/personality.rs
// Description: Personality system for digital humans
// Defines traits, values, and personality evolution

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::emotion::EmotionalState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityMatrix {
    pub traits: HashMap<PersonalityTrait, TraitValue>,
    pub emotional_state: EmotionalState,
    pub values: HashMap<String, f32>,
    pub quirks: Vec<PersonalityQuirk>,
}

impl PersonalityMatrix {
    pub fn new() -> Self {
        let mut traits = HashMap::new();

        // Initialize default traits
        traits.insert(PersonalityTrait::Openness, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Conscientiousness, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Extraversion, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Agreeableness, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Neuroticism, TraitValue::new(0.5));

        // Additional traits for game context
        traits.insert(PersonalityTrait::Courage, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Compassion, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Curiosity, TraitValue::new(0.5));
        traits.insert(PersonalityTrait::Aggression, TraitValue::new(0.3));
        traits.insert(PersonalityTrait::Diplomacy, TraitValue::new(0.5));

        Self {
            traits,
            emotional_state: EmotionalState::new(),
            values: Self::default_values(),
            quirks: vec![],
        }
    }

    fn default_values() -> HashMap<String, f32> {
        let mut values = HashMap::new();
        values.insert("honor".to_string(), 0.5);
        values.insert("freedom".to_string(), 0.5);
        values.insert("knowledge".to_string(), 0.5);
        values.insert("harmony".to_string(), 0.5);
        values.insert("power".to_string(), 0.3);
        values
    }

    pub fn get_trait_value(&self, trait_type: PersonalityTrait) -> f32 {
        self.traits.get(&trait_type)
            .map(|t| t.value)
            .unwrap_or(0.5)
    }

    pub fn modify_trait(&mut self, trait_type: PersonalityTrait, delta: f32) {
        if let Some(trait_value) = self.traits.get_mut(&trait_type) {
            trait_value.modify(delta);
        }
    }

    pub fn to_feature_vector(&self) -> Vec<f32> {
        let mut features = vec![];

        // Add trait values in consistent order
        let trait_order = [
            PersonalityTrait::Openness,
            PersonalityTrait::Conscientiousness,
            PersonalityTrait::Extraversion,
            PersonalityTrait::Agreeableness,
            PersonalityTrait::Neuroticism,
            PersonalityTrait::Courage,
            PersonalityTrait::Compassion,
            PersonalityTrait::Curiosity,
            PersonalityTrait::Aggression,
            PersonalityTrait::Diplomacy,
        ];

        for trait_type in &trait_order {
            features.push(self.get_trait_value(*trait_type));
        }

        // Add emotional state features
        features.extend(self.emotional_state.to_feature_vector());

        features
    }

    pub fn calculate_behavior_tendency(&self, behavior: &str) -> f32 {
        match behavior {
            "help_others" => {
                self.get_trait_value(PersonalityTrait::Compassion) * 0.6 +
                    self.get_trait_value(PersonalityTrait::Agreeableness) * 0.4
            }
            "explore" => {
                self.get_trait_value(PersonalityTrait::Curiosity) * 0.7 +
                    self.get_trait_value(PersonalityTrait::Openness) * 0.3
            }
            "fight" => {
                self.get_trait_value(PersonalityTrait::Aggression) * 0.5 +
                    self.get_trait_value(PersonalityTrait::Courage) * 0.3 +
                    (1.0 - self.get_trait_value(PersonalityTrait::Agreeableness)) * 0.2
            }
            "negotiate" => {
                self.get_trait_value(PersonalityTrait::Diplomacy) * 0.6 +
                    self.get_trait_value(PersonalityTrait::Conscientiousness) * 0.4
            }
            _ => 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PersonalityTrait {
    // Big Five
    Openness,
    Conscientiousness,
    Extraversion,
    Agreeableness,
    Neuroticism,

    // Game-specific
    Courage,
    Compassion,
    Curiosity,
    Aggression,
    Diplomacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitValue {
    pub value: f32,
    pub innate: f32,  // Base value
    pub learned: f32, // Modifications from experience
}

impl TraitValue {
    pub fn new(innate: f32) -> Self {
        Self {
            value: innate,
            innate,
            learned: 0.0,
        }
    }

    pub fn modify(&mut self, delta: f32) {
        self.learned = (self.learned + delta).clamp(-0.5, 0.5);
        self.value = (self.innate + self.learned).clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityQuirk {
    pub name: String,
    pub description: String,
    pub behavior_modifiers: HashMap<String, f32>,
}

impl PersonalityQuirk {
    pub fn perfectionist() -> Self {
        let mut modifiers = HashMap::new();
        modifiers.insert("work_quality".to_string(), 1.2);
        modifiers.insert("work_speed".to_string(), 0.8);
        modifiers.insert("stress_from_mistakes".to_string(), 1.5);

        Self {
            name: "Perfectionist".to_string(),
            description: "Always strives for perfection in everything".to_string(),
            behavior_modifiers: modifiers,
        }
    }

    pub fn night_owl() -> Self {
        let mut modifiers = HashMap::new();
        modifiers.insert("alertness_night".to_string(), 1.3);
        modifiers.insert("alertness_morning".to_string(), 0.7);

        Self {
            name: "Night Owl".to_string(),
            description: "More active and alert during nighttime".to_string(),
            behavior_modifiers: modifiers,
        }
    }

    pub fn mischievous() -> Self {
        let mut modifiers = HashMap::new();
        modifiers.insert("prank_tendency".to_string(), 1.5);
        modifiers.insert("humor_appreciation".to_string(), 1.3);
        modifiers.insert("rule_following".to_string(), 0.7);

        Self {
            name: "Mischievous".to_string(),
            description: "Loves pranks and playful trouble".to_string(),
            behavior_modifiers: modifiers,
        }
    }
}

// Personality archetypes for quick generation
pub struct PersonalityArchetypes;

impl PersonalityArchetypes {
    pub fn hero() -> PersonalityMatrix {
        let mut personality = PersonalityMatrix::new();
        personality.modify_trait(PersonalityTrait::Courage, 0.3);
        personality.modify_trait(PersonalityTrait::Compassion, 0.2);
        personality.modify_trait(PersonalityTrait::Aggression, -0.2);
        personality
    }

    pub fn scholar() -> PersonalityMatrix {
        let mut personality = PersonalityMatrix::new();
        personality.modify_trait(PersonalityTrait::Openness, 0.4);
        personality.modify_trait(PersonalityTrait::Curiosity, 0.4);
        personality.modify_trait(PersonalityTrait::Conscientiousness, 0.3);
        personality.modify_trait(PersonalityTrait::Extraversion, -0.2);
        personality
    }

    pub fn trickster() -> PersonalityMatrix {
        let mut personality = PersonalityMatrix::new();
        personality.modify_trait(PersonalityTrait::Openness, 0.3);
        personality.modify_trait(PersonalityTrait::Agreeableness, -0.1);
        personality.modify_trait(PersonalityTrait::Extraversion, 0.3);
        personality.quirks.push(PersonalityQuirk::mischievous());
        personality
    }

    pub fn guardian() -> PersonalityMatrix {
        let mut personality = PersonalityMatrix::new();
        personality.modify_trait(PersonalityTrait::Conscientiousness, 0.4);
        personality.modify_trait(PersonalityTrait::Courage, 0.3);
        personality.modify_trait(PersonalityTrait::Compassion, 0.2);
        personality.values.insert("duty".to_string(), 0.9);
        personality
    }

    pub fn mystic() -> PersonalityMatrix {
        let mut personality = PersonalityMatrix::new();
        personality.modify_trait(PersonalityTrait::Openness, 0.5);
        personality.modify_trait(PersonalityTrait::Neuroticism, 0.2);
        personality.modify_trait(PersonalityTrait::Curiosity, 0.4);
        personality.values.insert("spirituality".to_string(), 0.8);
        personality
    }
}