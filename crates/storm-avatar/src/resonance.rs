// File: storm-core/crates/storm-avatar/src/resonance.rs
// Description: Song resonance system for avatar-echo interaction
// Handles the mystical connection between avatars and echoes through harmonic frequencies

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use storm_ecs::prelude::*;
use crate::{EchoType, AvatarError, Result};

/// Core resonance data structure representing avatar-echo harmony
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SongResonance {
    pub harmony_level: f32, // 0.0 to 1.0, how in tune the avatar is
    pub primary_aspect: EchoType,
    pub secondary_aspects: Vec<(EchoType, f32)>, // Secondary echoes and their influence
    pub silence_resistance: f32, // Resistance to silence/disconnection
    pub resonance_history: ResonanceHistory,
}

impl Component for SongResonance {
    fn type_name() -> &'static str {
        "SongResonance"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for SongResonance {
    fn default() -> Self {
        Self {
            harmony_level: 0.5,
            primary_aspect: EchoType::Hope,
            secondary_aspects: vec![],
            silence_resistance: 0.3,
            resonance_history: ResonanceHistory::default(),
        }
    }
}

/// Historical record of resonance changes and events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResonanceHistory {
    pub events: Vec<ResonanceEvent>,
    pub peak_harmony: f32,
    pub lowest_harmony: f32,
    pub total_resonance_time: f32, // Time spent in active resonance
    pub echo_encounters: Vec<(EchoType, u32)>, // Changed from HashMap to Vec for PartialEq
}

impl Default for ResonanceHistory {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            peak_harmony: 0.0,
            lowest_harmony: 1.0,
            total_resonance_time: 0.0,
            echo_encounters: Vec::new(),
        }
    }
}

impl ResonanceHistory {
    /// Helper method to safely access echo encounters like a HashMap
    pub fn get_echo_encounters(&self, echo_type: &EchoType) -> u32 {
        self.echo_encounters
            .iter()
            .find(|(echo, _)| echo == echo_type)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }

    /// Helper method to increment echo encounters
    pub fn increment_echo_encounter(&mut self, echo_type: EchoType) {
        if let Some((_, count)) = self.echo_encounters
            .iter_mut()
            .find(|(echo, _)| *echo == echo_type) {
            *count += 1;
        } else {
            self.echo_encounters.push((echo_type, 1));
        }
    }
}

/// Individual resonance event recording
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResonanceEvent {
    pub timestamp: u64,
    pub event_type: ResonanceEventType,
    pub echo_involved: Option<EchoType>,
    pub harmony_before: f32,
    pub harmony_after: f32,
    pub location: Option<String>, // Where the event occurred
    pub description: String,
}

/// Types of resonance events that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResonanceEventType {
    /// First connection with an echo type
    FirstContact,
    /// Harmony level increased
    HarmonyGain,
    /// Harmony level decreased
    HarmonyLoss,
    /// Discovered a new aspect/secondary echo
    AspectDiscovery,
    /// Reached perfect harmony (1.0)
    PerfectHarmony,
    /// Lost connection (harmony dropped to 0)
    Silence,
    /// Recovered from silence
    Awakening,
    /// Conflicting echoes caused dissonance
    Dissonance,
    /// Resolved dissonance through balance
    Resolution,
}

/// Personality aspects that influence resonance patterns
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PersonalityAspect {
    Empathetic,    // Strong emotional connections
    Analytical,    // Logic-based resonance
    Creative,      // Artistic/imaginative resonance
    Protective,    // Guardian-like tendencies
    Adventurous,   // Seeks new experiences
    Contemplative, // Deep thinking and wisdom
    Passionate,    // Intense emotional responses
    Harmonious,    // Natural balance seeker
}

impl SongResonance {
    /// Create a new resonance with a specific primary echo
    pub fn new(primary_aspect: EchoType) -> Self {
        Self {
            harmony_level: 0.3, // Start with basic connection
            primary_aspect,
            secondary_aspects: vec![],
            silence_resistance: 0.2,
            resonance_history: ResonanceHistory::default(),
        }
    }

    /// Create resonance from personality aspects
    pub fn from_personality(aspects: &[PersonalityAspect]) -> Self {
        let primary_aspect = Self::determine_primary_echo(aspects);
        let mut resonance = Self::new(primary_aspect);

        // Set harmony level based on personality compatibility
        resonance.harmony_level = Self::calculate_initial_harmony(aspects, primary_aspect);

        // Add secondary aspects based on personality
        resonance.secondary_aspects = Self::determine_secondary_echoes(aspects, primary_aspect);

        resonance
    }

    /// Determine primary echo type from personality aspects
    fn determine_primary_echo(aspects: &[PersonalityAspect]) -> EchoType {
        let mut scores = HashMap::new();

        for aspect in aspects {
            let echo_influences = match aspect {
                PersonalityAspect::Empathetic => vec![(EchoType::Love, 3), (EchoType::Hope, 2)],
                PersonalityAspect::Analytical => vec![(EchoType::Logic, 3), (EchoType::Order, 2)],
                PersonalityAspect::Creative => vec![(EchoType::Dream, 3), (EchoType::Chaos, 1)],
                PersonalityAspect::Protective => vec![(EchoType::Hope, 2), (EchoType::Light, 2)],
                PersonalityAspect::Adventurous => vec![(EchoType::Joy, 2), (EchoType::Chaos, 1)],
                PersonalityAspect::Contemplative => vec![(EchoType::Memory, 3), (EchoType::Sorrow, 1)],
                PersonalityAspect::Passionate => vec![(EchoType::Love, 2), (EchoType::Fear, 1)],
                PersonalityAspect::Harmonious => vec![(EchoType::Order, 2), (EchoType::Hope, 1)],
            };

            for (echo, weight) in echo_influences {
                *scores.entry(echo).or_insert(0) += weight;
            }
        }

        // Return the echo with the highest score, default to Hope
        scores.into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(echo, _)| echo)
            .unwrap_or(EchoType::Hope)
    }

    /// Calculate initial harmony level based on personality compatibility
    fn calculate_initial_harmony(aspects: &[PersonalityAspect], primary_echo: EchoType) -> f32 {
        let base_harmony = 0.3;
        let compatibility_bonus = aspects.iter()
            .map(|aspect| Self::aspect_echo_compatibility(*aspect, primary_echo))
            .sum::<f32>() / aspects.len().max(1) as f32;

        (base_harmony + compatibility_bonus * 0.4).clamp(0.1, 0.8)
    }

    /// Determine secondary echo influences
    fn determine_secondary_echoes(aspects: &[PersonalityAspect], primary: EchoType) -> Vec<(EchoType, f32)> {
        let mut secondary = Vec::new();

        // Add complementary echoes based on primary
        let complementary = match primary {
            EchoType::Hope => vec![(EchoType::Memory, 0.3), (EchoType::Light, 0.2)],
            EchoType::Logic => vec![(EchoType::Order, 0.4), (EchoType::Memory, 0.2)],
            EchoType::Love => vec![(EchoType::Joy, 0.3), (EchoType::Hope, 0.2)],
            EchoType::Dream => vec![(EchoType::Memory, 0.3), (EchoType::Chaos, 0.1)],
            EchoType::Light => vec![(EchoType::Hope, 0.3), (EchoType::Order, 0.2)],
            EchoType::Shadow => vec![(EchoType::Memory, 0.2), (EchoType::Fear, 0.1)],
            EchoType::Joy => vec![(EchoType::Love, 0.3), (EchoType::Hope, 0.2)],
            EchoType::Sorrow => vec![(EchoType::Memory, 0.4)],
            EchoType::Order => vec![(EchoType::Logic, 0.3), (EchoType::Light, 0.2)],
            EchoType::Chaos => vec![(EchoType::Dream, 0.2), (EchoType::Fear, 0.1)],
            EchoType::Fear => vec![(EchoType::Shadow, 0.3), (EchoType::Chaos, 0.1)],
            EchoType::Memory => vec![(EchoType::Sorrow, 0.2), (EchoType::Logic, 0.2)],
            EchoType::Wisdom => vec![(EchoType::Logic, 0.3), (EchoType::Memory, 0.3)],
            // Complementary to Logic and Memory for Wisdom, aligning with
            // intelligence bonuses and Time/Space affinities
        };

        secondary.extend(complementary);
        secondary
    }

    /// Calculate compatibility between personality aspect and echo type
    fn aspect_echo_compatibility(aspect: PersonalityAspect, echo: EchoType) -> f32 {
        match (aspect, echo) {
            (PersonalityAspect::Empathetic, EchoType::Love) => 0.9,
            (PersonalityAspect::Empathetic, EchoType::Hope) => 0.7,
            (PersonalityAspect::Analytical, EchoType::Logic) => 0.9,
            (PersonalityAspect::Analytical, EchoType::Order) => 0.7,
            (PersonalityAspect::Creative, EchoType::Dream) => 0.9,
            (PersonalityAspect::Creative, EchoType::Chaos) => 0.6,
            (PersonalityAspect::Protective, EchoType::Light) => 0.8,
            (PersonalityAspect::Protective, EchoType::Hope) => 0.7,
            (PersonalityAspect::Adventurous, EchoType::Joy) => 0.8,
            (PersonalityAspect::Contemplative, EchoType::Memory) => 0.9,
            (PersonalityAspect::Contemplative, EchoType::Sorrow) => 0.6,
            (PersonalityAspect::Passionate, EchoType::Love) => 0.8,
            (PersonalityAspect::Harmonious, EchoType::Order) => 0.8,
            (PersonalityAspect::Harmonious, EchoType::Hope) => 0.7,
            _ => 0.3, // Default compatibility
        }
    }

    /// Update harmony level and record the change
    pub fn update_harmony(&mut self, new_level: f32, reason: ResonanceEventType, location: Option<String>) {
        let old_level = self.harmony_level;
        self.harmony_level = new_level.clamp(0.0, 1.0);

        // Update history peaks
        if self.harmony_level > self.resonance_history.peak_harmony {
            self.resonance_history.peak_harmony = self.harmony_level;
        }
        if self.harmony_level < self.resonance_history.lowest_harmony {
            self.resonance_history.lowest_harmony = self.harmony_level;
        }

        // Record the event
        let event = ResonanceEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_type: reason,
            echo_involved: Some(self.primary_aspect),
            harmony_before: old_level,
            harmony_after: self.harmony_level,
            location,
            description: format!("Harmony changed from {:.2} to {:.2}", old_level, self.harmony_level),
        };

        self.resonance_history.events.push(event);
    }

    /// Add or strengthen a secondary echo aspect
    pub fn add_secondary_aspect(&mut self, echo_type: EchoType, strength: f32) -> Result<()> {
        if echo_type == self.primary_aspect {
            return Err(AvatarError::InvalidData("Cannot add primary echo as secondary".to_string()));
        }

        // Check if already exists
        for (existing_echo, existing_strength) in &mut self.secondary_aspects {
            if *existing_echo == echo_type {
                *existing_strength = (*existing_strength + strength).min(1.0);
                return Ok(());
            }
        }

        // Add new secondary aspect
        self.secondary_aspects.push((echo_type, strength.clamp(0.0, 1.0)));

        // Record the discovery
        let event = ResonanceEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_type: ResonanceEventType::AspectDiscovery,
            echo_involved: Some(echo_type),
            harmony_before: self.harmony_level,
            harmony_after: self.harmony_level,
            location: None,
            description: format!("Discovered connection to {:?} echo", echo_type),
        };

        self.resonance_history.events.push(event);

        Ok(())
    }

    /// Check if avatar can resonate with a specific echo type
    pub fn can_resonate_with(&self, echo_type: &EchoType) -> bool {
        if *echo_type == self.primary_aspect {
            return true;
        }

        // Check secondary aspects
        self.secondary_aspects.iter()
            .any(|(secondary_echo, strength)| secondary_echo == echo_type && *strength > 0.1)
    }

    /// Get the strength of resonance with a specific echo type
    pub fn resonance_strength(&self, echo_type: &EchoType) -> f32 {
        if *echo_type == self.primary_aspect {
            return self.harmony_level;
        }

        // Check secondary aspects
        self.secondary_aspects.iter()
            .find(|(secondary_echo, _)| secondary_echo == echo_type)
            .map(|(_, strength)| *strength * self.harmony_level)
            .unwrap_or(0.0)
    }

    /// Calculate resistance to silence/disconnection
    pub fn calculate_silence_resistance(&self) -> f32 {
        let base_resistance = self.silence_resistance;
        let harmony_bonus = self.harmony_level * 0.3;
        let experience_bonus = (self.resonance_history.events.len() as f32 * 0.01).min(0.2);

        (base_resistance + harmony_bonus + experience_bonus).clamp(0.0, 1.0)
    }

    /// Process interaction with another echo type
    pub fn process_echo_interaction(&mut self, echo_type: EchoType, intensity: f32, location: Option<String>) -> Result<()> {
        // Record encounter using the helper method
        self.resonance_history.increment_echo_encounter(echo_type);

        if echo_type == self.primary_aspect {
            // Interacting with primary echo - strengthen harmony
            let harmony_gain = intensity * 0.1;
            let new_harmony = (self.harmony_level + harmony_gain).min(1.0);
            self.update_harmony(new_harmony, ResonanceEventType::HarmonyGain, location);
        } else if self.can_resonate_with(&echo_type) {
            // Interacting with secondary echo - minor harmony gain
            let harmony_gain = intensity * 0.05;
            let new_harmony = (self.harmony_level + harmony_gain).min(1.0);
            self.update_harmony(new_harmony, ResonanceEventType::HarmonyGain, location);
        } else {
            // New echo type - potential discovery or dissonance
            if intensity > 0.7 && self.harmony_level > 0.5 {
                // Strong interaction with high harmony - discover new aspect
                self.add_secondary_aspect(echo_type, intensity * 0.3)?;
            } else if Self::are_echoes_conflicting(&self.primary_aspect, &echo_type) {
                // Conflicting echoes cause dissonance
                let harmony_loss = intensity * 0.15;
                let new_harmony = (self.harmony_level - harmony_loss).max(0.0);
                self.update_harmony(new_harmony, ResonanceEventType::Dissonance, location);
            }
        }

        Ok(())
    }

    /// Check if two echo types are fundamentally conflicting
    fn are_echoes_conflicting(echo1: &EchoType, echo2: &EchoType) -> bool {
        matches!(
            (echo1, echo2),
            (EchoType::Hope, EchoType::Sorrow) | (EchoType::Sorrow, EchoType::Hope) |
            (EchoType::Light, EchoType::Shadow) | (EchoType::Shadow, EchoType::Light) |
            (EchoType::Order, EchoType::Chaos) | (EchoType::Chaos, EchoType::Order) |
            (EchoType::Love, EchoType::Fear) | (EchoType::Fear, EchoType::Love)
        )
    }

    /// Get a summary of the avatar's resonance state
    pub fn get_summary(&self) -> ResonanceSummary {
        ResonanceSummary {
            primary_echo: self.primary_aspect,
            harmony_level: self.harmony_level,
            secondary_count: self.secondary_aspects.len(),
            total_encounters: self.resonance_history.echo_encounters.iter().map(|(_, count)| *count).sum(),
            peak_harmony: self.resonance_history.peak_harmony,
            silence_resistance: self.calculate_silence_resistance(),
            can_achieve_perfect_harmony: self.harmony_level > 0.8 && !self.secondary_aspects.is_empty(),
        }
    }
}

/// Summary structure for easy access to resonance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceSummary {
    pub primary_echo: EchoType,
    pub harmony_level: f32,
    pub secondary_count: usize,
    pub total_encounters: u32,
    pub peak_harmony: f32,
    pub silence_resistance: f32,
    pub can_achieve_perfect_harmony: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resonance_creation() {
        let resonance = SongResonance::new(EchoType::Hope);
        assert_eq!(resonance.primary_aspect, EchoType::Hope);
        assert_eq!(resonance.harmony_level, 0.3);
    }

    #[test]
    fn test_personality_based_resonance() {
        let aspects = vec![PersonalityAspect::Empathetic, PersonalityAspect::Protective];
        let resonance = SongResonance::from_personality(&aspects);

        // Should likely be Hope or Love based on empathetic/protective traits
        assert!(matches!(resonance.primary_aspect, EchoType::Hope | EchoType::Love));
        assert!(resonance.harmony_level > 0.3);
    }

    #[test]
    fn test_harmony_update() {
        let mut resonance = SongResonance::new(EchoType::Logic);
        let initial_events = resonance.resonance_history.events.len();

        resonance.update_harmony(0.8, ResonanceEventType::HarmonyGain, None);

        assert_eq!(resonance.harmony_level, 0.8);
        assert_eq!(resonance.resonance_history.events.len(), initial_events + 1);
        assert_eq!(resonance.resonance_history.peak_harmony, 0.8);
    }

    #[test]
    fn test_secondary_aspect_addition() {
        let mut resonance = SongResonance::new(EchoType::Hope);

        assert!(resonance.add_secondary_aspect(EchoType::Memory, 0.4).is_ok());
        assert!(resonance.can_resonate_with(&EchoType::Memory));
        assert_eq!(resonance.resonance_strength(&EchoType::Memory), 0.4 * resonance.harmony_level);

        // Should fail to add primary as secondary
        assert!(resonance.add_secondary_aspect(EchoType::Hope, 0.5).is_err());
    }

    #[test]
    fn test_echo_interaction() {
        let mut resonance = SongResonance::new(EchoType::Light);
        let initial_harmony = resonance.harmony_level;

        // Interact with primary echo - should increase harmony
        resonance.process_echo_interaction(EchoType::Light, 0.8, None).unwrap();
        assert!(resonance.harmony_level > initial_harmony);

        // Interact with conflicting echo - should decrease harmony
        let harmony_before = resonance.harmony_level;
        resonance.process_echo_interaction(EchoType::Shadow, 0.7, None).unwrap();
        assert!(resonance.harmony_level < harmony_before);
    }

    #[test]
    fn test_conflicting_echoes() {
        assert!(SongResonance::are_echoes_conflicting(&EchoType::Hope, &EchoType::Sorrow));
        assert!(SongResonance::are_echoes_conflicting(&EchoType::Light, &EchoType::Shadow));
        assert!(!SongResonance::are_echoes_conflicting(&EchoType::Hope, &EchoType::Love));
    }

    #[test]
    fn test_silence_resistance() {
        let mut resonance = SongResonance::new(EchoType::Order);
        resonance.harmony_level = 0.8;

        // Add some experience
        for _ in 0..10 {
            resonance.resonance_history.events.push(ResonanceEvent {
                timestamp: 0,
                event_type: ResonanceEventType::HarmonyGain,
                echo_involved: Some(EchoType::Order),
                harmony_before: 0.5,
                harmony_after: 0.6,
                location: None,
                description: "Test event".to_string(),
            });
        }

        let resistance = resonance.calculate_silence_resistance();
        assert!(resistance > resonance.silence_resistance);
    }

    #[test]
    fn test_resonance_summary() {
        let mut resonance = SongResonance::new(EchoType::Dream);
        resonance.harmony_level = 0.9;
        resonance.add_secondary_aspect(EchoType::Memory, 0.3).unwrap();
        resonance.add_secondary_aspect(EchoType::Hope, 0.2).unwrap();

        let summary = resonance.get_summary();
        assert_eq!(summary.primary_echo, EchoType::Dream);
        assert_eq!(summary.harmony_level, 0.9);
        assert_eq!(summary.secondary_count, 2);
        assert!(summary.can_achieve_perfect_harmony);
    }

    #[test]
    fn test_echo_encounters() {
        let mut resonance = SongResonance::new(EchoType::Hope);

        // Test encounter tracking
        resonance.process_echo_interaction(EchoType::Light, 0.5, None).unwrap();
        resonance.process_echo_interaction(EchoType::Light, 0.3, None).unwrap();

        assert_eq!(resonance.resonance_history.get_echo_encounters(&EchoType::Light), 2);
        assert_eq!(resonance.resonance_history.get_echo_encounters(&EchoType::Shadow), 0);
    }
}