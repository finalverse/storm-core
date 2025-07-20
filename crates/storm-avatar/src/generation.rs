// File: storm-core/crates/storm-avatar/src/generation.rs
// Description: Procedural avatar generation system
// Creates unique avatars based on parameters and AI assistance

use crate::*;
use storm_math::Vec3;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Avatar generator with configurable parameters
pub struct AvatarGenerator {
    rng: StdRng,
    seed: u64,
}

impl AvatarGenerator {
    /// Create a new generator with a specific seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Generate a new avatar with the given parameters
    pub fn generate_avatar(&mut self, params: GenerationParams) -> Result<GeneratedAvatar> {
        let base = self.generate_base_avatar(&params)?;
        let customization = self.generate_customization(&params, &base)?;
        let resonance = self.generate_resonance(&params)?;

        Ok(GeneratedAvatar {
            base,
            customization,
            resonance,
            ai_enhancements: None,
        })
    }

    /// Generate the base avatar information
    fn generate_base_avatar(&mut self, params: &GenerationParams) -> Result<AvatarBase> {
        let id = Uuid::new_v4();
        let archetype = self.generate_archetype(params)?;
        let name = params.name.clone().unwrap_or_else(|| self.generate_name(&archetype));

        Ok(AvatarBase {
            id,
            name,
            archetype,
            level: 1,
            experience: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_modified: 0,
        })
    }

    /// Generate an archetype based on preferences
    fn generate_archetype(&mut self, params: &GenerationParams) -> Result<AvatarArchetype> {
        match &params.archetype_preference {
            Some(ArchetypePreference::Human) => {
                Ok(AvatarArchetype::Human {
                    age_group: params.age_preference.unwrap_or_else(|| self.random_age_group()),
                    body_type: params.body_type_preference.unwrap_or_else(|| self.random_body_type()),
                    heritage: params.heritage_preference.unwrap_or_else(|| self.random_heritage()),
                })
            }
            Some(ArchetypePreference::Echo(echo_type)) => {
                Ok(AvatarArchetype::Echo {
                    echo_type: *echo_type,
                    energy_pattern: self.random_energy_pattern(),
                    manifestation: self.random_manifestation(),
                })
            }
            Some(ArchetypePreference::Hybrid) => {
                // Create cloned params to avoid borrow issues
                let human_params = GenerationParams {
                    archetype_preference: Some(ArchetypePreference::Human),
                    ..params.clone()
                };
                let echo_params = GenerationParams {
                    archetype_preference: Some(ArchetypePreference::Echo(EchoType::Hope)),
                    ..params.clone()
                };

                let primary = Box::new(self.generate_archetype(&human_params)?);
                let secondary = Box::new(self.generate_archetype(&echo_params)?);
                let blend_factor = self.rng.gen_range(0.2..0.8);

                Ok(AvatarArchetype::Hybrid {
                    primary,
                    secondary,
                    blend_factor,
                })
            }
            Some(ArchetypePreference::Custom) => {
                let mut traits = HashMap::new();

                // Generate random traits
                traits.insert("strength".to_string(), self.rng.gen_range(-0.5..0.5));
                traits.insert("agility".to_string(), self.rng.gen_range(-0.5..0.5));
                traits.insert("intelligence".to_string(), self.rng.gen_range(-0.5..0.5));
                traits.insert("charisma".to_string(), self.rng.gen_range(-0.5..0.5));

                Ok(AvatarArchetype::Custom {
                    name: "Unique Being".to_string(),
                    traits,
                    appearance_modifiers: vec![],
                })
            }
            None => {
                // Random archetype - create params for each case
                match self.rng.gen_range(0..4) {
                    0 => {
                        let human_params = GenerationParams {
                            archetype_preference: Some(ArchetypePreference::Human),
                            ..params.clone()
                        };
                        self.generate_archetype(&human_params)
                    },
                    1 => {
                        let echo_params = GenerationParams {
                            archetype_preference: Some(ArchetypePreference::Echo(self.random_echo_type())),
                            ..params.clone()
                        };
                        self.generate_archetype(&echo_params)
                    },
                    2 => {
                        let hybrid_params = GenerationParams {
                            archetype_preference: Some(ArchetypePreference::Hybrid),
                            ..params.clone()
                        };
                        self.generate_archetype(&hybrid_params)
                    },
                    _ => {
                        let custom_params = GenerationParams {
                            archetype_preference: Some(ArchetypePreference::Custom),
                            ..params.clone()
                        };
                        self.generate_archetype(&custom_params)
                    },
                }
            }
        }
    }

    /// Generate customization data
    fn generate_customization(&mut self, params: &GenerationParams, base: &AvatarBase) -> Result<CustomizationData> {
        let mut customization = CustomizationData::default();

        // Apply randomization based on archetype
        self.apply_archetype_customization(&mut customization, &base.archetype)?;

        // Apply user preferences if any
        if let Some(ref prefs) = params.customization_hints {
            self.apply_customization_hints(&mut customization, prefs)?;
        }

        Ok(customization)
    }

    /// Generate resonance data
    fn generate_resonance(&mut self, params: &GenerationParams) -> Result<SongResonance> {
        if let Some(ref personality) = params.personality_aspects {
            Ok(SongResonance::from_personality(personality))
        } else {
            // Generate random personality and create resonance
            let personality = self.generate_random_personality();
            Ok(SongResonance::from_personality(&personality))
        }
    }

    /// Generate a name based on archetype
    fn generate_name(&mut self, archetype: &AvatarArchetype) -> String {
        let name_parts = match archetype {
            AvatarArchetype::Human { heritage, .. } => {
                match heritage {
                    Heritage::Terran => &TERRAN_NAMES,
                    Heritage::Celestial => &CELESTIAL_NAMES,
                    Heritage::Aquatic => &AQUATIC_NAMES,
                    Heritage::Synthetic => &SYNTHETIC_NAMES,
                    Heritage::Ethereal => &ETHEREAL_NAMES,
                    Heritage::Ancient => &ANCIENT_NAMES,
                }
            }
            AvatarArchetype::Echo { echo_type, .. } => {
                match echo_type {
                    EchoType::Hope => &HOPE_NAMES,
                    EchoType::Light => &LIGHT_NAMES,
                    EchoType::Shadow => &SHADOW_NAMES,
                    EchoType::Dream => &DREAM_NAMES,
                    EchoType::Wisdom => &WISDOM_NAMES,
                    _ => &ECHO_NAMES,
                }
            }
            AvatarArchetype::Hybrid { .. } => &HYBRID_NAMES,
            AvatarArchetype::Custom { .. } => &CUSTOM_NAMES,
        };

        let first = name_parts.firsts[self.rng.gen_range(0..name_parts.firsts.len())];
        let last = name_parts.lasts[self.rng.gen_range(0..name_parts.lasts.len())];
        format!("{} {}", first, last)
    }

    /// Apply archetype-specific customization
    fn apply_archetype_customization(&mut self, customization: &mut CustomizationData, archetype: &AvatarArchetype) -> Result<()> {
        match archetype {
            AvatarArchetype::Human { age_group, body_type, heritage } => {
                self.apply_human_customization(customization, *age_group, *body_type, *heritage)?;
            }
            AvatarArchetype::Echo { echo_type, energy_pattern, manifestation } => {
                self.apply_echo_customization(customization, *echo_type, *energy_pattern, *manifestation)?;
            }
            AvatarArchetype::Hybrid { primary, secondary, blend_factor } => {
                let mut primary_custom = CustomizationData::default();
                let mut secondary_custom = CustomizationData::default();

                self.apply_archetype_customization(&mut primary_custom, primary)?;
                self.apply_archetype_customization(&mut secondary_custom, secondary)?;

                *customization = self.blend_customizations(&primary_custom, &secondary_custom, *blend_factor);
            }
            AvatarArchetype::Custom { traits, appearance_modifiers, .. } => {
                customization.appearance_modifiers = appearance_modifiers.clone();
                self.apply_trait_based_customization(customization, traits)?;
            }
        }
        Ok(())
    }

    /// Apply human-specific customization
    fn apply_human_customization(&mut self, customization: &mut CustomizationData, age_group: AgeGroup, body_type: BodyType, heritage: Heritage) -> Result<()> {
        // Age group modifications
        match age_group {
            AgeGroup::Child => {
                if let Some(height_morph) = customization.morph_targets.get_mut("height") {
                    height_morph.value = -0.3;
                }
                if let Some(face_morph) = customization.morph_targets.get_mut("face_width") {
                    face_morph.value = 0.2;
                }
            }
            AgeGroup::Teen => {
                if let Some(height_morph) = customization.morph_targets.get_mut("height") {
                    height_morph.value = -0.1;
                }
            }
            AgeGroup::Elder => {
                if let Some(height_morph) = customization.morph_targets.get_mut("height") {
                    height_morph.value = -0.1;
                }
                // Add age lines, etc.
            }
            _ => {}
        }

        // Body type modifications
        match body_type {
            BodyType::Petite => {
                if let Some(height_morph) = customization.morph_targets.get_mut("height") {
                    height_morph.value -= 0.2;
                }
                if let Some(weight_morph) = customization.morph_targets.get_mut("body_weight") {
                    weight_morph.value = -0.3;
                }
            }
            BodyType::Muscular => {
                if let Some(muscle_morph) = customization.morph_targets.get_mut("muscle_definition") {
                    muscle_morph.value = 0.8;
                }
                if let Some(weight_morph) = customization.morph_targets.get_mut("body_weight") {
                    weight_morph.value = 0.2;
                }
            }
            BodyType::Heavy => {
                if let Some(weight_morph) = customization.morph_targets.get_mut("body_weight") {
                    weight_morph.value = 0.6;
                }
            }
            _ => {}
        }

        // Heritage-based appearance
        self.apply_heritage_appearance(customization, heritage)?;

        Ok(())
    }

    /// Apply echo-specific customization
    fn apply_echo_customization(&mut self, customization: &mut CustomizationData, echo_type: EchoType, energy_pattern: EnergyPattern, manifestation: Manifestation) -> Result<()> {
        // Add particle effects based on echo type
        let particle_effect = ParticleEffect {
            name: format!("{:?}_aura", echo_type),
            effect_type: match echo_type {
                EchoType::Light => ParticleType::Glow,
                EchoType::Shadow => ParticleType::Smoke,
                EchoType::Hope => ParticleType::Sparkle,
                EchoType::Dream => ParticleType::Ethereal,
                EchoType::Wisdom => ParticleType::Light,
                _ => ParticleType::Energy,
            },
            intensity: 0.7,
            color: self.get_echo_color(echo_type),
            pattern: energy_pattern,
        };
        customization.particle_effects.push(particle_effect);

        // Add manifestation-based modifications
        match manifestation {
            Manifestation::Ethereal => {
                customization.texture_layers.push(TextureLayer {
                    name: "ethereal_overlay".to_string(),
                    blend_mode: BlendMode::Overlay,
                    opacity: 0.3,
                    color_tint: [1.0, 1.0, 1.0, 0.7],
                    texture_path: "textures/ethereal.png".to_string(),
                });
            }
            Manifestation::Energy => {
                // Add energy glow effects
                customization.texture_layers.push(TextureLayer {
                    name: "energy_glow".to_string(),
                    blend_mode: BlendMode::Add,
                    opacity: 0.8,
                    color_tint: self.get_echo_color(echo_type),
                    texture_path: "textures/energy_glow.png".to_string(),
                });
            }
            _ => {}
        }

        Ok(())
    }

    /// Get color associated with echo type
    fn get_echo_color(&self, echo_type: EchoType) -> [f32; 4] {
        match echo_type {
            EchoType::Hope => [1.0, 0.9, 0.3, 1.0],      // Golden
            EchoType::Light => [1.0, 1.0, 0.9, 1.0],     // Bright white
            EchoType::Shadow => [0.2, 0.1, 0.3, 1.0],    // Dark purple
            EchoType::Love => [1.0, 0.3, 0.5, 1.0],      // Pink
            EchoType::Fear => [0.5, 0.1, 0.1, 1.0],      // Dark red
            EchoType::Joy => [1.0, 0.8, 0.2, 1.0],       // Bright yellow
            EchoType::Sorrow => [0.3, 0.4, 0.7, 1.0],    // Blue
            EchoType::Logic => [0.5, 0.8, 1.0, 1.0],     // Light blue
            EchoType::Dream => [0.8, 0.5, 1.0, 1.0],     // Purple
            EchoType::Memory => [0.7, 0.7, 0.8, 1.0],    // Silver
            EchoType::Order => [0.9, 0.9, 0.9, 1.0],     // White
            EchoType::Chaos => [1.0, 0.2, 0.8, 1.0],     // Magenta
            EchoType::Wisdom => [0.9, 0.8, 0.5, 1.0],    // Golden wisdom
        }
    }

    /// Apply heritage-based appearance
    fn apply_heritage_appearance(&mut self, customization: &mut CustomizationData, heritage: Heritage) -> Result<()> {
        match heritage {
            Heritage::Celestial => {
                // Add starlike patterns
                customization.texture_layers.push(TextureLayer {
                    name: "star_patterns".to_string(),
                    blend_mode: BlendMode::Multiply,
                    opacity: 0.4,
                    color_tint: [0.8, 0.9, 1.0, 1.0],
                    texture_path: "textures/celestial_markings.png".to_string(),
                });
            }
            Heritage::Aquatic => {
                // Add water-like effects
                customization.texture_layers.push(TextureLayer {
                    name: "water_shimmer".to_string(),
                    blend_mode: BlendMode::Overlay,
                    opacity: 0.6,
                    color_tint: [0.3, 0.7, 1.0, 1.0],
                    texture_path: "textures/water_shimmer.png".to_string(),
                });
            }
            Heritage::Synthetic => {
                // Add technological elements
                customization.texture_layers.push(TextureLayer {
                    name: "circuit_patterns".to_string(),
                    blend_mode: BlendMode::Screen,
                    opacity: 0.3,
                    color_tint: [0.0, 1.0, 0.5, 1.0],
                    texture_path: "textures/circuits.png".to_string(),
                });
            }
            _ => {}
        }
        Ok(())
    }

    /// Generate random personality aspects
    fn generate_random_personality(&mut self) -> Vec<PersonalityAspect> {
        let all_aspects = [
            PersonalityAspect::Empathetic,
            PersonalityAspect::Analytical,
            PersonalityAspect::Creative,
            PersonalityAspect::Protective,
            PersonalityAspect::Adventurous,
            PersonalityAspect::Contemplative,
            PersonalityAspect::Passionate,
            PersonalityAspect::Harmonious,
        ];

        let count = self.rng.gen_range(2..5);
        let mut personality = Vec::new();

        for _ in 0..count {
            let aspect = all_aspects[self.rng.gen_range(0..all_aspects.len())];
            if !personality.contains(&aspect) {
                personality.push(aspect);
            }
        }

        personality
    }

    /// Random generation helpers
    fn random_age_group(&mut self) -> AgeGroup {
        match self.rng.gen_range(0..5) {
            0 => AgeGroup::Child,
            1 => AgeGroup::Teen,
            2 => AgeGroup::YoungAdult,
            3 => AgeGroup::Adult,
            _ => AgeGroup::Elder,
        }
    }

    fn random_body_type(&mut self) -> BodyType {
        match self.rng.gen_range(0..6) {
            0 => BodyType::Petite,
            1 => BodyType::Slim,
            2 => BodyType::Average,
            3 => BodyType::Athletic,
            4 => BodyType::Muscular,
            _ => BodyType::Heavy,
        }
    }

    fn random_heritage(&mut self) -> Heritage {
        match self.rng.gen_range(0..6) {
            0 => Heritage::Terran,
            1 => Heritage::Celestial,
            2 => Heritage::Aquatic,
            3 => Heritage::Synthetic,
            4 => Heritage::Ethereal,
            _ => Heritage::Ancient,
        }
    }

    fn random_echo_type(&mut self) -> EchoType {
        match self.rng.gen_range(0..13) {
            0 => EchoType::Hope,
            1 => EchoType::Logic,
            2 => EchoType::Memory,
            3 => EchoType::Dream,
            4 => EchoType::Shadow,
            5 => EchoType::Light,
            6 => EchoType::Chaos,
            7 => EchoType::Order,
            8 => EchoType::Love,
            9 => EchoType::Fear,
            10 => EchoType::Joy,
            11 => EchoType::Sorrow,
            _ => EchoType::Wisdom,
        }
    }

    fn random_energy_pattern(&mut self) -> EnergyPattern {
        match self.rng.gen_range(0..6) {
            0 => EnergyPattern::Flowing,
            1 => EnergyPattern::Pulsing,
            2 => EnergyPattern::Crystalline,
            3 => EnergyPattern::Chaotic,
            4 => EnergyPattern::Harmonic,
            _ => EnergyPattern::Static,
        }
    }

    fn random_manifestation(&mut self) -> Manifestation {
        match self.rng.gen_range(0..6) {
            0 => Manifestation::Physical,
            1 => Manifestation::Ethereal,
            2 => Manifestation::Energy,
            3 => Manifestation::Geometric,
            4 => Manifestation::Organic,
            _ => Manifestation::Mechanical,
        }
    }

    /// Blend two customization sets
    fn blend_customizations(&self, primary: &CustomizationData, secondary: &CustomizationData, blend_factor: f32) -> CustomizationData {
        let mut result = primary.clone();

        // Blend morph targets
        for (name, secondary_morph) in &secondary.morph_targets {
            if let Some(primary_morph) = result.morph_targets.get_mut(name) {
                primary_morph.value = primary_morph.value * (1.0 - blend_factor) + secondary_morph.value * blend_factor;
            }
        }

        // Combine texture layers with opacity blending
        for secondary_layer in &secondary.texture_layers {
            let mut blended_layer = secondary_layer.clone();
            blended_layer.opacity *= blend_factor;
            result.texture_layers.push(blended_layer);
        }

        // Combine particle effects
        for secondary_effect in &secondary.particle_effects {
            let mut blended_effect = secondary_effect.clone();
            blended_effect.intensity *= blend_factor;
            result.particle_effects.push(blended_effect);
        }

        result
    }

    /// Apply customization hints from user preferences
    fn apply_customization_hints(&self, customization: &mut CustomizationData, hints: &CustomizationHints) -> Result<()> {
        if let Some(preferred_colors) = &hints.preferred_colors {
            // Apply color preferences to texture layers
            for layer in &mut customization.texture_layers {
                if layer.name.contains("base") {
                    layer.color_tint = *preferred_colors.first().unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
                }
            }
        }

        if let Some(style) = &hints.style_preference {
            match style {
                StylePreference::Minimal => {
                    // Remove excessive effects
                    customization.particle_effects.retain(|effect| effect.intensity < 0.5);
                    customization.texture_layers.retain(|layer| layer.opacity > 0.3);
                }
                StylePreference::Elaborate => {
                    // Enhance effects
                    for effect in &mut customization.particle_effects {
                        effect.intensity = (effect.intensity * 1.2).min(1.0);
                    }
                }
                StylePreference::Natural => {
                    // Remove synthetic elements
                    customization.texture_layers.retain(|layer| !layer.name.contains("circuit"));
                }
                StylePreference::Futuristic => {
                    // Add tech elements
                    customization.texture_layers.push(TextureLayer {
                        name: "tech_overlay".to_string(),
                        blend_mode: BlendMode::Screen,
                        opacity: 0.3,
                        color_tint: [0.0, 1.0, 1.0, 1.0],
                        texture_path: "textures/tech.png".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Apply trait-based customization for custom archetypes
    fn apply_trait_based_customization(&self, customization: &mut CustomizationData, traits: &HashMap<String, f32>) -> Result<()> {
        for (trait_name, value) in traits {
            match trait_name.as_str() {
                "glow" => {
                    if *value > 0.0 {
                        customization.particle_effects.push(ParticleEffect {
                            name: "custom_glow".to_string(),
                            effect_type: ParticleType::Glow,
                            intensity: *value,
                            color: [1.0, 1.0, 1.0, 1.0],
                            pattern: EnergyPattern::Static,
                        });
                    }
                }
                "transparency" => {
                    if *value > 0.0 {
                        customization.texture_layers.push(TextureLayer {
                            name: "transparency_overlay".to_string(),
                            blend_mode: BlendMode::Overlay,
                            opacity: 1.0 - value,
                            color_tint: [1.0, 1.0, 1.0, 1.0 - value],
                            texture_path: "textures/transparent.png".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Parameters for avatar generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub name: Option<String>,
    pub archetype_preference: Option<ArchetypePreference>,
    pub age_preference: Option<AgeGroup>,
    pub body_type_preference: Option<BodyType>,
    pub heritage_preference: Option<Heritage>,
    pub personality_aspects: Option<Vec<PersonalityAspect>>,
    pub customization_hints: Option<CustomizationHints>,
    pub generation_seed: Option<u64>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            name: None,
            archetype_preference: None,
            age_preference: None,
            body_type_preference: None,
            heritage_preference: None,
            personality_aspects: None,
            customization_hints: None,
            generation_seed: None,
        }
    }
}

/// Archetype preferences for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchetypePreference {
    Human,
    Echo(EchoType),
    Hybrid,
    Custom,
}

/// Customization hints from user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizationHints {
    pub preferred_colors: Option<Vec<[f32; 4]>>,
    pub style_preference: Option<StylePreference>,
    pub intensity_preference: Option<f32>, // 0.0 = subtle, 1.0 = dramatic
}

/// Style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StylePreference {
    Minimal,
    Elaborate,
    Natural,
    Futuristic,
}

/// Generated avatar result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAvatar {
    pub base: AvatarBase,
    pub customization: CustomizationData,
    pub resonance: SongResonance,
    pub ai_enhancements: Option<AIEnhancements>,
}

/// AI-generated enhancements (placeholder for future AI integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEnhancements {
    pub personality_analysis: String,
    pub suggested_improvements: Vec<String>,
    pub compatibility_matches: Vec<EchoType>,
}

/// Name sets for different heritage types
struct NameSet {
    firsts: &'static [&'static str],
    lasts: &'static [&'static str],
}

const TERRAN_NAMES: NameSet = NameSet {
    firsts: &["Alex", "Jordan", "Morgan", "Casey", "Taylor", "Riley", "Avery", "Blake"],
    lasts: &["Chen", "Smith", "Garcia", "Johnson", "Williams", "Brown", "Jones", "Miller"],
};

const CELESTIAL_NAMES: NameSet = NameSet {
    firsts: &["Stella", "Orion", "Luna", "Nova", "Vega", "Lyra", "Altair", "Sirius"],
    lasts: &["Starweaver", "Voidwalker", "Lightbringer", "Cosmos", "Nebula", "Stellar"],
};

const AQUATIC_NAMES: NameSet = NameSet {
    firsts: &["Coral", "Pearl", "Marina", "Reef", "Tide", "Current", "Depth", "Wave"],
    lasts: &["Deepwater", "Tidal", "Oceanic", "Seafoam", "Pelagic", "Abyssal"],
};

const SYNTHETIC_NAMES: NameSet = NameSet {
    firsts: &["Nexus", "Circuit", "Binary", "Data", "Code", "Logic", "Neural", "Quantum"],
    lasts: &["Protocol", "Interface", "System", "Network", "Matrix", "Core"],
};

const ETHEREAL_NAMES: NameSet = NameSet {
    firsts: &["Whisper", "Echo", "Shade", "Mist", "Spirit", "Wraith", "Phantom", "Essence"],
    lasts: &["Ethereal", "Spectral", "Ghostly", "Wisp", "Vapor", "Shimmer"],
};

const ANCIENT_NAMES: NameSet = NameSet {
    firsts: &["Sage", "Elder", "Ancient", "Mystic", "Oracle", "Keeper", "Guardian", "Wisdom"],
    lasts: &["Oldstone", "Timeless", "Eternal", "Ageless", "Primordial", "Forgotten"],
};

const HOPE_NAMES: NameSet = NameSet {
    firsts: &["Dawn", "Aurora", "Beacon", "Ray", "Bright", "Shine", "Gleam", "Radiant"],
    lasts: &["Hopeful", "Bright", "Dawn", "Light", "Beacon", "Promise"],
};

const LIGHT_NAMES: NameSet = NameSet {
    firsts: &["Lux", "Bright", "Radiant", "Gleam", "Shine", "Glow", "Beam", "Flash"],
    lasts: &["Luminous", "Brilliant", "Radiant", "Bright", "Shining", "Glowing"],
};

const SHADOW_NAMES: NameSet = NameSet {
    firsts: &["Shade", "Dark", "Shadow", "Dim", "Dusk", "Twilight", "Night", "Gloom"],
    lasts: &["Shadow", "Dark", "Shade", "Twilight", "Dusk", "Night"],
};

const DREAM_NAMES: NameSet = NameSet {
    firsts: &["Dream", "Vision", "Fantasy", "Mirage", "Illusion", "Reverie", "Imagine", "Wonder"],
    lasts: &["Dreamer", "Visionary", "Mystic", "Ethereal", "Fantastic", "Surreal"],
};

const WISDOM_NAMES: NameSet = NameSet {
    firsts: &["Sage", "Wise", "Scholar", "Learned", "Enlightened", "Knowing", "Insight", "Truth"],
    lasts: &["Wisdom", "Sage", "Scholar", "Learned", "Enlightened", "Knowing"],
};

const ECHO_NAMES: NameSet = NameSet {
    firsts: &["Echo", "Resonance", "Harmony", "Chord", "Note", "Melody", "Rhythm", "Beat"],
    lasts: &["Echo", "Resonant", "Harmonic", "Melodic", "Rhythmic", "Sonic"],
};

const HYBRID_NAMES: NameSet = NameSet {
    firsts: &["Dual", "Twin", "Balance", "Merge", "Blend", "Unity", "Fusion", "Bridge"],
    lasts: &["Hybrid", "Merged", "Blended", "United", "Fused", "Bridged"],
};

const CUSTOM_NAMES: NameSet = NameSet {
    firsts: &["Unique", "Original", "Special", "Distinct", "Individual", "Singular", "Rare", "Exceptional"],
    lasts: &["Custom", "Unique", "Original", "Distinct", "Special", "Individual"],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_generator_creation() {
        let generator = AvatarGenerator::new(12345);
        assert_eq!(generator.seed, 12345);
    }

    #[test]
    fn test_basic_avatar_generation() {
        let mut generator = AvatarGenerator::new(12345);
        let params = GenerationParams::default();

        let result = generator.generate_avatar(params);
        assert!(result.is_ok());

        let avatar = result.unwrap();
        assert!(!avatar.base.name.is_empty());
        assert_eq!(avatar.base.level, 1);
    }

    #[test]
    fn test_human_archetype_generation() {
        let mut generator = AvatarGenerator::new(12345);
        let params = GenerationParams {
            archetype_preference: Some(ArchetypePreference::Human),
            ..Default::default()
        };

        let avatar = generator.generate_avatar(params).unwrap();
        assert!(matches!(avatar.base.archetype, AvatarArchetype::Human { .. }));
    }

    #[test]
    fn test_echo_archetype_generation() {
        let mut generator = AvatarGenerator::new(12345);
        let params = GenerationParams {
            archetype_preference: Some(ArchetypePreference::Echo(EchoType::Hope)),
            ..Default::default()
        };

        let avatar = generator.generate_avatar(params).unwrap();
        if let AvatarArchetype::Echo { echo_type, .. } = avatar.base.archetype {
            assert_eq!(echo_type, EchoType::Hope);
        } else {
            panic!("Expected Echo archetype");
        }
    }

    #[test]
    fn test_name_generation() {
        let mut generator = AvatarGenerator::new(12345);
        let archetype = AvatarArchetype::Human {
            age_group: AgeGroup::Adult,
            body_type: BodyType::Average,
            heritage: Heritage::Terran,
        };

        let name = generator.generate_name(&archetype);
        assert!(!name.is_empty());
        assert!(name.contains(' ')); // Should have first and last name
    }

    #[test]
    fn test_customization_hints() {
        let mut generator = AvatarGenerator::new(12345);
        let params = GenerationParams {
            customization_hints: Some(CustomizationHints {
                preferred_colors: Some(vec![[1.0, 0.0, 0.0, 1.0]]), // Red
                style_preference: Some(StylePreference::Minimal),
                intensity_preference: Some(0.3),
            }),
            ..Default::default()
        };

        let avatar = generator.generate_avatar(params).unwrap();
        // Should have applied customization hints
        assert!(!avatar.customization.texture_layers.is_empty() || !avatar.customization.particle_effects.is_empty());
    }

    #[test]
    fn test_deterministic_generation() {
        let seed = 42;
        let params = GenerationParams::default();

        let mut generator1 = AvatarGenerator::new(seed);
        let avatar1 = generator1.generate_avatar(params.clone()).unwrap();

        let mut generator2 = AvatarGenerator::new(seed);
        let avatar2 = generator2.generate_avatar(params).unwrap();

        // Should generate the same avatar with the same seed
        assert_eq!(avatar1.base.archetype, avatar2.base.archetype);
        assert_eq!(avatar1.base.name, avatar2.base.name);
    }

    #[test]
    fn test_wisdom_echo_generation() {
        let mut generator = AvatarGenerator::new(12345);
        let params = GenerationParams {
            archetype_preference: Some(ArchetypePreference::Echo(EchoType::Wisdom)),
            ..Default::default()
        };

        let avatar = generator.generate_avatar(params).unwrap();
        if let AvatarArchetype::Echo { echo_type, .. } = avatar.base.archetype {
            assert_eq!(echo_type, EchoType::Wisdom);
        } else {
            panic!("Expected Wisdom Echo archetype");
        }

        // Should have wisdom-colored particle effects
        assert!(!avatar.customization.particle_effects.is_empty());
    }

    #[test]
    fn test_personality_aspects() {
        let mut generator = AvatarGenerator::new(12345);
        let params = GenerationParams {
            personality_aspects: Some(vec![
                PersonalityAspect::Contemplative,
                PersonalityAspect::Analytical,
            ]),
            ..Default::default()
        };

        let avatar = generator.generate_avatar(params).unwrap();
        // Should likely have Logic or Memory as primary echo due to analytical/contemplative traits
        assert!(matches!(
            avatar.resonance.primary_aspect,
            EchoType::Logic | EchoType::Memory | EchoType::Wisdom
        ));
    }
}