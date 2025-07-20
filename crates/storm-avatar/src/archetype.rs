// File: storm-core/crates/storm-avatar/src/archetype.rs
// Description: Avatar archetype definitions and traits
// Defines the fundamental avatar types and their characteristics

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AvatarArchetype {
    Human {
        age_group: AgeGroup,
        body_type: BodyType,
        heritage: Heritage,
    },
    Echo {
        echo_type: EchoType,
        energy_pattern: EnergyPattern,
        manifestation: Manifestation,
    },
    Hybrid {
        primary: Box<AvatarArchetype>,
        secondary: Box<AvatarArchetype>,
        blend_factor: f32, // 0.0 = all primary, 1.0 = all secondary
    },
    Custom {
        name: String,
        traits: HashMap<String, f32>,
        appearance_modifiers: Vec<AppearanceModifier>,
    },
}

impl Default for AvatarArchetype {
    fn default() -> Self {
        Self::Human {
            age_group: AgeGroup::Adult,
            body_type: BodyType::Average,
            heritage: Heritage::Terran,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgeGroup {
    Child,      // 5-12
    Teen,       // 13-17
    YoungAdult, // 18-25
    Adult,      // 26-55
    Elder,      // 56+
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BodyType {
    Petite,
    Slim,
    Average,
    Athletic,
    Muscular,
    Heavy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Heritage {
    Terran,      // Earth-based human lineage
    Celestial,   // Star-born heritage
    Aquatic,     // Ocean-world heritage
    Synthetic,   // Artificial/technological heritage
    Ethereal,    // Energy-based heritage
    Ancient,     // Old-world heritage
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EchoType {
    Hope,
    Logic,
    Memory,
    Dream,
    Shadow,
    Light,
    Chaos,
    Order,
    Love,
    Fear,
    Joy,
    Sorrow,
    Wisdom, // Added the missing Wisdom variant
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnergyPattern {
    Flowing,     // Smooth, continuous energy flow
    Pulsing,     // Rhythmic energy pulses
    Crystalline, // Structured, geometric patterns
    Chaotic,     // Unpredictable, wild energy
    Harmonic,    // Musical, wave-like patterns
    Static,      // Stable, unchanging energy
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Manifestation {
    Physical,    // Solid, tangible form
    Ethereal,    // Semi-transparent, ghostly
    Energy,      // Pure energy, light-based
    Geometric,   // Mathematical, abstract forms
    Organic,     // Nature-based, flowing forms
    Mechanical,  // Tech-based, structured forms
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceModifier {
    pub name: String,
    pub effect_type: ModifierType,
    pub intensity: f32,
    pub affected_regions: Vec<BodyRegion>,
    pub parameters: HashMap<String, f32>,
}

// Manual PartialEq implementation to handle HashMap comparison properly
impl PartialEq for AppearanceModifier {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.effect_type == other.effect_type
            && (self.intensity - other.intensity).abs() < f32::EPSILON
            && self.affected_regions == other.affected_regions
            && self.params_eq(other)
    }
}

impl AppearanceModifier {
    fn params_eq(&self, other: &Self) -> bool {
        if self.parameters.len() != other.parameters.len() {
            return false;
        }

        for (key, value) in &self.parameters {
            match other.parameters.get(key) {
                Some(other_value) => {
                    if (value - other_value).abs() > f32::EPSILON {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModifierType {
    Scale,
    Color,
    Texture,
    Glow,
    Transparency,
    Particle,
    Animation
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BodyRegion {
    Head,
    Face,
    Eyes,
    Hair,
    Torso,
    Arms,
    Hands,
    Legs,
    Feet,
    Wings,
    Tail,
    Aura,
}

/// Archetype statistics and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeStats {
    pub strength: f32,
    pub agility: f32,
    pub intelligence: f32,
    pub charisma: f32,
    pub wisdom: f32,
    pub constitution: f32,
    pub echo_resonance: f32,
    pub manifestation_power: f32,
}

// Custom PartialEq for ArchetypeStats to handle float comparison
impl PartialEq for ArchetypeStats {
    fn eq(&self, other: &Self) -> bool {
        (self.strength - other.strength).abs() < f32::EPSILON
            && (self.agility - other.agility).abs() < f32::EPSILON
            && (self.intelligence - other.intelligence).abs() < f32::EPSILON
            && (self.charisma - other.charisma).abs() < f32::EPSILON
            && (self.wisdom - other.wisdom).abs() < f32::EPSILON
            && (self.constitution - other.constitution).abs() < f32::EPSILON
            && (self.echo_resonance - other.echo_resonance).abs() < f32::EPSILON
            && (self.manifestation_power - other.manifestation_power).abs() < f32::EPSILON
    }
}

impl Default for ArchetypeStats {
    fn default() -> Self {
        Self {
            strength: 0.5,
            agility: 0.5,
            intelligence: 0.5,
            charisma: 0.5,
            wisdom: 0.5,
            constitution: 0.5,
            echo_resonance: 0.0,
            manifestation_power: 0.0,
        }
    }
}

impl ArchetypeStats {
    /// Blend two stat sets with a factor (0.0 = all first, 1.0 = all second)
    pub fn blend(&self, other: &Self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        let inv_factor = 1.0 - factor;

        Self {
            strength: self.strength * inv_factor + other.strength * factor,
            agility: self.agility * inv_factor + other.agility * factor,
            intelligence: self.intelligence * inv_factor + other.intelligence * factor,
            charisma: self.charisma * inv_factor + other.charisma * factor,
            wisdom: self.wisdom * inv_factor + other.wisdom * factor,
            constitution: self.constitution * inv_factor + other.constitution * factor,
            echo_resonance: self.echo_resonance * inv_factor + other.echo_resonance * factor,
            manifestation_power: self.manifestation_power * inv_factor + other.manifestation_power * factor,
        }
    }
}

/// Archetype capabilities and restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchetypeCapabilities {
    pub can_fly: bool,
    pub can_phase: bool,
    pub can_shapeshift: bool,
    pub elemental_affinities: Vec<ElementType>,
    pub special_abilities: Vec<String>,
    pub restrictions: Vec<String>,
    pub echo_compatibility: Vec<EchoType>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ElementType {
    Fire,
    Water,
    Earth,
    Air,
    Light,
    Shadow,
    Time,
    Space,
    Life,
    Death,
}

impl AvatarArchetype {
    /// Get the base statistics for this archetype
    pub fn base_stats(&self) -> ArchetypeStats {
        match self {
            AvatarArchetype::Human { age_group, body_type, heritage: _ } => {
                let mut stats = ArchetypeStats::default();

                // Age group modifiers
                match age_group {
                    AgeGroup::Child => {
                        stats.agility += 0.2;
                        stats.strength -= 0.1;
                        stats.constitution -= 0.1;
                    }
                    AgeGroup::Teen => {
                        stats.agility += 0.1;
                        stats.charisma += 0.1;
                    }
                    AgeGroup::YoungAdult => {
                        stats.strength += 0.1;
                        stats.constitution += 0.1;
                    }
                    AgeGroup::Adult => {
                        stats.intelligence += 0.1;
                        stats.wisdom += 0.1;
                    }
                    AgeGroup::Elder => {
                        stats.wisdom += 0.3;
                        stats.intelligence += 0.2;
                        stats.strength -= 0.1;
                        stats.agility -= 0.1;
                    }
                }

                // Body type modifiers
                match body_type {
                    BodyType::Petite => {
                        stats.agility += 0.2;
                        stats.strength -= 0.1;
                    }
                    BodyType::Slim => {
                        stats.agility += 0.1;
                    }
                    BodyType::Average => {
                        // No modifiers - balanced
                    }
                    BodyType::Athletic => {
                        stats.strength += 0.1;
                        stats.agility += 0.1;
                        stats.constitution += 0.1;
                    }
                    BodyType::Muscular => {
                        stats.strength += 0.2;
                        stats.constitution += 0.1;
                        stats.agility -= 0.1;
                    }
                    BodyType::Heavy => {
                        stats.constitution += 0.2;
                        stats.strength += 0.1;
                        stats.agility -= 0.2;
                    }
                }

                stats
            }

            AvatarArchetype::Echo { echo_type, energy_pattern: _, manifestation: _ } => {
                let mut stats = ArchetypeStats::default();
                stats.echo_resonance = 1.0;
                stats.manifestation_power = 0.8;

                // Echo type specific bonuses
                match echo_type {
                    EchoType::Hope => {
                        stats.charisma += 0.3;
                        stats.wisdom += 0.2;
                    }
                    EchoType::Logic => {
                        stats.intelligence += 0.4;
                        stats.wisdom += 0.1;
                    }
                    EchoType::Memory => {
                        stats.intelligence += 0.2;
                        stats.wisdom += 0.3;
                    }
                    EchoType::Dream => {
                        stats.charisma += 0.2;
                        stats.manifestation_power += 0.2;
                    }
                    EchoType::Shadow => {
                        stats.agility += 0.3;
                        stats.intelligence += 0.1;
                    }
                    EchoType::Light => {
                        stats.charisma += 0.2;
                        stats.constitution += 0.2;
                    }
                    EchoType::Chaos => {
                        stats.manifestation_power += 0.3;
                        stats.strength += 0.1;
                    }
                    EchoType::Order => {
                        stats.intelligence += 0.2;
                        stats.constitution += 0.2;
                    }
                    EchoType::Love => {
                        stats.charisma += 0.4;
                    }
                    EchoType::Fear => {
                        stats.manifestation_power += 0.2;
                        stats.agility += 0.2;
                    }
                    EchoType::Joy => {
                        stats.charisma += 0.2;
                        stats.agility += 0.2;
                    }
                    EchoType::Sorrow => {
                        stats.wisdom += 0.3;
                        stats.intelligence += 0.1;
                    }
                    EchoType::Wisdom => {
                        stats.wisdom += 0.4;
                        stats.intelligence += 0.3;
                    }
                }

                stats
            }

            AvatarArchetype::Hybrid { primary, secondary, blend_factor } => {
                let primary_stats = primary.base_stats();
                let secondary_stats = secondary.base_stats();
                primary_stats.blend(&secondary_stats, *blend_factor)
            }

            AvatarArchetype::Custom { traits, .. } => {
                let mut stats = ArchetypeStats::default();

                // Apply custom trait modifiers
                for (trait_name, value) in traits {
                    match trait_name.as_str() {
                        "strength" => stats.strength += value,
                        "agility" => stats.agility += value,
                        "intelligence" => stats.intelligence += value,
                        "charisma" => stats.charisma += value,
                        "wisdom" => stats.wisdom += value,
                        "constitution" => stats.constitution += value,
                        "echo_resonance" => stats.echo_resonance += value,
                        "manifestation_power" => stats.manifestation_power += value,
                        _ => {} // Unknown trait, ignore
                    }
                }

                // Clamp all values to valid ranges
                stats.strength = stats.strength.clamp(0.0, 2.0);
                stats.agility = stats.agility.clamp(0.0, 2.0);
                stats.intelligence = stats.intelligence.clamp(0.0, 2.0);
                stats.charisma = stats.charisma.clamp(0.0, 2.0);
                stats.wisdom = stats.wisdom.clamp(0.0, 2.0);
                stats.constitution = stats.constitution.clamp(0.0, 2.0);
                stats.echo_resonance = stats.echo_resonance.clamp(0.0, 1.0);
                stats.manifestation_power = stats.manifestation_power.clamp(0.0, 1.0);

                stats
            }
        }
    }

    /// Get the capabilities for this archetype
    pub fn capabilities(&self) -> ArchetypeCapabilities {
        match self {
            AvatarArchetype::Human { .. } => {
                ArchetypeCapabilities {
                    can_fly: false,
                    can_phase: false,
                    can_shapeshift: false,
                    elemental_affinities: vec![],
                    special_abilities: vec![
                        "Tool Use".to_string(),
                        "Language".to_string(),
                        "Learning".to_string(),
                    ],
                    restrictions: vec![],
                    echo_compatibility: vec![
                        EchoType::Hope,
                        EchoType::Logic,
                        EchoType::Memory,
                        EchoType::Love,
                        EchoType::Joy,
                        EchoType::Sorrow,
                        EchoType::Wisdom,
                    ],
                }
            }

            AvatarArchetype::Echo { echo_type, manifestation, .. } => {
                let mut capabilities = ArchetypeCapabilities {
                    can_fly: true,
                    can_phase: true,
                    can_shapeshift: matches!(manifestation, Manifestation::Energy | Manifestation::Ethereal),
                    elemental_affinities: vec![],
                    special_abilities: vec![
                        "Echo Resonance".to_string(),
                        "Manifestation".to_string(),
                        "Energy Manipulation".to_string(),
                    ],
                    restrictions: vec![
                        "Requires Echo Energy".to_string(),
                    ],
                    echo_compatibility: vec![*echo_type],
                };

                // Add echo-specific affinities
                match echo_type {
                    EchoType::Hope | EchoType::Light => {
                        capabilities.elemental_affinities.push(ElementType::Light);
                    }
                    EchoType::Shadow | EchoType::Fear => {
                        capabilities.elemental_affinities.push(ElementType::Shadow);
                    }
                    EchoType::Logic | EchoType::Order => {
                        capabilities.elemental_affinities.push(ElementType::Time);
                    }
                    EchoType::Dream | EchoType::Memory => {
                        capabilities.elemental_affinities.push(ElementType::Space);
                    }
                    EchoType::Love | EchoType::Joy => {
                        capabilities.elemental_affinities.push(ElementType::Life);
                    }
                    EchoType::Sorrow => {
                        capabilities.elemental_affinities.push(ElementType::Death);
                    }
                    EchoType::Chaos => {
                        capabilities.elemental_affinities.extend_from_slice(&[
                            ElementType::Fire,
                            ElementType::Air,
                        ]);
                    }
                    EchoType::Wisdom => {
                        capabilities.elemental_affinities.extend_from_slice(&[
                            ElementType::Time,
                            ElementType::Space,
                        ]);
                    }
                }

                capabilities
            }

            AvatarArchetype::Hybrid { primary, secondary, .. } => {
                let primary_caps = primary.capabilities();
                let secondary_caps = secondary.capabilities();

                // Merge capabilities
                let mut merged = primary_caps;
                merged.can_fly |= secondary_caps.can_fly;
                merged.can_phase |= secondary_caps.can_phase;
                merged.can_shapeshift |= secondary_caps.can_shapeshift;

                // Combine unique elements
                for affinity in secondary_caps.elemental_affinities {
                    if !merged.elemental_affinities.contains(&affinity) {
                        merged.elemental_affinities.push(affinity);
                    }
                }

                // Combine unique abilities
                for ability in secondary_caps.special_abilities {
                    if !merged.special_abilities.contains(&ability) {
                        merged.special_abilities.push(ability);
                    }
                }

                // Combine restrictions
                merged.restrictions.extend(secondary_caps.restrictions);

                // Combine echo compatibility
                for echo in secondary_caps.echo_compatibility {
                    if !merged.echo_compatibility.contains(&echo) {
                        merged.echo_compatibility.push(echo);
                    }
                }

                merged
            }

            AvatarArchetype::Custom { traits, .. } => {
                // Basic capabilities with custom trait influences
                ArchetypeCapabilities {
                    can_fly: traits.get("flight").copied().unwrap_or(0.0) > 0.5,
                    can_phase: traits.get("phasing").copied().unwrap_or(0.0) > 0.5,
                    can_shapeshift: traits.get("shapeshifting").copied().unwrap_or(0.0) > 0.5,
                    elemental_affinities: vec![], // Would be determined by trait analysis
                    special_abilities: vec!["Custom Abilities".to_string()],
                    restrictions: vec![],
                    echo_compatibility: vec![], // All compatible by default
                }
            }
        }
    }

    /// Check if this archetype is compatible with a specific echo type
    pub fn is_compatible_with(&self, echo_type: &EchoType) -> bool {
        match self {
            AvatarArchetype::Echo { echo_type: self_echo, .. } => {
                self_echo == echo_type || self.is_complementary_echo(echo_type)
            }
            _ => {
                let capabilities = self.capabilities();
                capabilities.echo_compatibility.contains(echo_type)
            }
        }
    }

    /// Check if an echo type is complementary to this archetype's echo
    fn is_complementary_echo(&self, other: &EchoType) -> bool {
        if let AvatarArchetype::Echo { echo_type, .. } = self {
            match (echo_type, other) {
                (EchoType::Hope, EchoType::Sorrow) | (EchoType::Sorrow, EchoType::Hope) => true,
                (EchoType::Light, EchoType::Shadow) | (EchoType::Shadow, EchoType::Light) => true,
                (EchoType::Order, EchoType::Chaos) | (EchoType::Chaos, EchoType::Order) => true,
                (EchoType::Love, EchoType::Fear) | (EchoType::Fear, EchoType::Love) => true,
                (EchoType::Joy, EchoType::Sorrow) | (EchoType::Sorrow, EchoType::Joy) => true,
                (EchoType::Logic, EchoType::Dream) | (EchoType::Dream, EchoType::Logic) => true,
                (EchoType::Wisdom, EchoType::Chaos) | (EchoType::Chaos, EchoType::Wisdom) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get a human-readable description of this archetype
    pub fn description(&self) -> String {
        match self {
            AvatarArchetype::Human { age_group, body_type, heritage } => {
                format!("{:?} {:?} with {:?} heritage", age_group, body_type, heritage)
            }
            AvatarArchetype::Echo { echo_type, energy_pattern, manifestation } => {
                format!("{:?} Echo with {:?} energy and {:?} manifestation",
                        echo_type, energy_pattern, manifestation)
            }
            AvatarArchetype::Hybrid { primary, secondary, blend_factor } => {
                format!("Hybrid: {}% {} + {}% {}",
                        ((1.0 - blend_factor) * 100.0) as u8,
                        primary.description(),
                        (*blend_factor * 100.0) as u8,
                        secondary.description())
            }
            AvatarArchetype::Custom { name, traits, .. } => {
                format!("Custom: {} (with {} traits)", name, traits.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_archetype() {
        let archetype = AvatarArchetype::default();
        assert!(matches!(archetype, AvatarArchetype::Human { .. }));
    }

    #[test]
    fn test_archetype_stats() {
        let human = AvatarArchetype::Human {
            age_group: AgeGroup::Adult,
            body_type: BodyType::Athletic,
            heritage: Heritage::Terran,
        };

        let stats = human.base_stats();
        assert!(stats.strength > 0.5); // Athletic bonus
        assert!(stats.constitution > 0.5); // Athletic bonus
    }

    #[test]
    fn test_echo_compatibility() {
        let hope_echo = AvatarArchetype::Echo {
            echo_type: EchoType::Hope,
            energy_pattern: EnergyPattern::Flowing,
            manifestation: Manifestation::Ethereal,
        };

        assert!(hope_echo.is_compatible_with(&EchoType::Hope));
        assert!(hope_echo.is_compatible_with(&EchoType::Sorrow)); // Complementary
    }

    #[test]
    fn test_stat_blending() {
        let stats1 = ArchetypeStats {
            strength: 1.0,
            agility: 0.0,
            ..Default::default()
        };
        let stats2 = ArchetypeStats {
            strength: 0.0,
            agility: 1.0,
            ..Default::default()
        };

        let blended = stats1.blend(&stats2, 0.5);
        assert_eq!(blended.strength, 0.5);
        assert_eq!(blended.agility, 0.5);
    }

    #[test]
    fn test_hybrid_archetype() {
        let human = AvatarArchetype::Human {
            age_group: AgeGroup::Adult,
            body_type: BodyType::Average,
            heritage: Heritage::Terran,
        };

        let echo = AvatarArchetype::Echo {
            echo_type: EchoType::Hope,
            energy_pattern: EnergyPattern::Flowing,
            manifestation: Manifestation::Ethereal,
        };

        let hybrid = AvatarArchetype::Hybrid {
            primary: Box::new(human),
            secondary: Box::new(echo),
            blend_factor: 0.3,
        };

        let stats = hybrid.base_stats();
        assert!(stats.echo_resonance > 0.0); // Should have some echo resonance
        assert!(stats.echo_resonance < 1.0); // But not full echo power
    }

    #[test]
    fn test_capabilities() {
        let echo = AvatarArchetype::Echo {
            echo_type: EchoType::Light,
            energy_pattern: EnergyPattern::Flowing,
            manifestation: Manifestation::Energy,
        };

        let caps = echo.capabilities();
        assert!(caps.can_fly);
        assert!(caps.can_phase);
        assert!(caps.elemental_affinities.contains(&ElementType::Light));
    }

    #[test]
    fn test_wisdom_echo() {
        let wisdom_echo = AvatarArchetype::Echo {
            echo_type: EchoType::Wisdom,
            energy_pattern: EnergyPattern::Harmonic,
            manifestation: Manifestation::Ethereal,
        };

        let stats = wisdom_echo.base_stats();
        assert!(stats.wisdom > 0.5);
        assert!(stats.intelligence > 0.5);

        let caps = wisdom_echo.capabilities();
        assert!(caps.elemental_affinities.contains(&ElementType::Time));
        assert!(caps.elemental_affinities.contains(&ElementType::Space));
    }

    #[test]
    fn test_appearance_modifier_equality() {
        let mut params1 = HashMap::new();
        params1.insert("scale".to_string(), 1.5);
        params1.insert("brightness".to_string(), 0.8);

        let mut params2 = HashMap::new();
        params2.insert("scale".to_string(), 1.5);
        params2.insert("brightness".to_string(), 0.8);

        let modifier1 = AppearanceModifier {
            name: "Test Modifier".to_string(),
            effect_type: ModifierType::Scale,
            intensity: 0.75,
            affected_regions: vec![BodyRegion::Head, BodyRegion::Arms],
            parameters: params1,
        };

        let modifier2 = AppearanceModifier {
            name: "Test Modifier".to_string(),
            effect_type: ModifierType::Scale,
            intensity: 0.75,
            affected_regions: vec![BodyRegion::Head, BodyRegion::Arms],
            parameters: params2,
        };

        assert_eq!(modifier1, modifier2);
    }

    #[test]
    fn test_archetype_stats_equality() {
        let stats1 = ArchetypeStats {
            strength: 0.75,
            agility: 0.5,
            intelligence: 0.9,
            charisma: 0.6,
            wisdom: 0.8,
            constitution: 0.7,
            echo_resonance: 0.3,
            manifestation_power: 0.4,
        };

        let stats2 = ArchetypeStats {
            strength: 0.75,
            agility: 0.5,
            intelligence: 0.9,
            charisma: 0.6,
            wisdom: 0.8,
            constitution: 0.7,
            echo_resonance: 0.3,
            manifestation_power: 0.4,
        };

        assert_eq!(stats1, stats2);
    }
}