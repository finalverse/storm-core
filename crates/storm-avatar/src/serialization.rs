// File: storm-core/crates/storm-avatar/src/serialization.rs
// Description: Avatar serialization and deserialization for save/load and networking
// Handles binary and JSON formats for avatars and components

use crate::*;
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::collections::HashMap;

/// Serializable avatar data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAvatar {
    pub version: u32,
    pub base: AvatarBase,
    pub stats: AvatarStats,
    pub resonance: SongResonance,
    pub customization: CustomizationData,
    pub inventory: Option<AvatarInventory>,
    pub metadata: AvatarMetadata,
}

/// Avatar metadata for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarMetadata {
    pub save_timestamp: u64,
    pub client_version: String,
    pub world_origin: Option<String>,
    pub checksum: Option<String>,
}

impl SerializableAvatar {
    const CURRENT_VERSION: u32 = 1;

    /// Create from individual components
    pub fn new(
        base: AvatarBase,
        stats: AvatarStats,
        resonance: SongResonance,
        customization: CustomizationData,
        inventory: Option<AvatarInventory>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            base,
            stats,
            resonance,
            customization,
            inventory,
            metadata: AvatarMetadata {
                save_timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                client_version: env!("CARGO_PKG_VERSION").to_string(),
                world_origin: None,
                checksum: None,
            },
        }
    }

    /// Serialize to binary format
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))
    }

    /// Deserialize from binary format
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let avatar: Self = bincode::deserialize(data)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))?;

        // Version compatibility check
        if avatar.version > Self::CURRENT_VERSION {
            return Err(AvatarError::InvalidData(
                format!("Unsupported avatar version: {}", avatar.version)
            ));
        }

        Ok(avatar)
    }

    /// Serialize to JSON format
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))
    }

    /// Deserialize from JSON format
    pub fn from_json(json: &str) -> Result<Self> {
        let avatar: Self = serde_json::from_str(json)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))?;

        // Version compatibility check
        if avatar.version > Self::CURRENT_VERSION {
            return Err(AvatarError::InvalidData(
                format!("Unsupported avatar version: {}", avatar.version)
            ));
        }

        Ok(avatar)
    }

    /// Write to a file
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let data = self.to_binary()?;
        writer.write_all(&data)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))?;
        Ok(())
    }

    /// Read from a file
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))?;

        Self::from_binary(&buffer)
    }

    /// Calculate checksum for data integrity
    pub fn calculate_checksum(&self) -> Result<String> {
        let data = self.to_binary()?;
        let hash = calculate_simple_hash(&data);
        Ok(format!("{:x}", hash))
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> Result<bool> {
        if let Some(ref stored_checksum) = self.metadata.checksum {
            let calculated = self.calculate_checksum()?;
            Ok(&calculated == stored_checksum)
        } else {
            Ok(true) // No checksum to verify
        }
    }

    /// Update metadata
    pub fn update_metadata(&mut self, world_origin: Option<String>) -> Result<()> {
        self.metadata.save_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.metadata.world_origin = world_origin;
        self.metadata.checksum = Some(self.calculate_checksum()?);
        Ok(())
    }

    /// Migrate from older version if needed
    pub fn migrate_if_needed(mut self) -> Result<Self> {
        match self.version {
            0 => {
                // Migrate from version 0 to 1
                // Add any missing fields with defaults
                self.version = 1;
            }
            1 => {
                // Current version, no migration needed
            }
            _ => {
                return Err(AvatarError::InvalidData(
                    format!("Unknown avatar version: {}", self.version)
                ));
            }
        }

        Ok(self)
    }
}

/// Avatar preset for quick character creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarPreset {
    pub name: String,
    pub description: String,
    pub archetype: AvatarArchetype,
    pub customization: CustomizationData,
    pub resonance: SongResonance,
    pub tags: Vec<String>,
    pub difficulty: PresetDifficulty,
}

/// Difficulty level for presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl AvatarPreset {
    /// Apply this preset to create a new avatar
    pub fn apply_to_avatar(&self, name: Option<String>) -> GeneratedAvatar {
        let base = AvatarBase {
            id: uuid::Uuid::new_v4(),
            name: name.unwrap_or_else(|| format!("{} Clone", self.name)),
            archetype: self.archetype.clone(),
            level: 1,
            experience: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_modified: 0,
        };

        GeneratedAvatar {
            base,
            customization: self.customization.clone(),
            resonance: self.resonance.clone(),
            ai_enhancements: None,
        }
    }

    /// Create a preset from an existing avatar
    pub fn from_avatar(
        avatar: &SerializableAvatar,
        name: String,
        description: String,
        tags: Vec<String>,
        difficulty: PresetDifficulty,
    ) -> Self {
        Self {
            name,
            description,
            archetype: avatar.base.archetype.clone(),
            customization: avatar.customization.clone(),
            resonance: avatar.resonance.clone(),
            tags,
            difficulty,
        }
    }
}

/// Preset library for managing avatar presets
pub struct AvatarPresetLibrary {
    presets: HashMap<String, AvatarPreset>,
}

impl AvatarPresetLibrary {
    pub fn new() -> Self {
        Self {
            presets: Self::create_default_presets(),
        }
    }

    /// Add a preset to the library
    pub fn add_preset(&mut self, preset: AvatarPreset) {
        self.presets.insert(preset.name.clone(), preset);
    }

    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&AvatarPreset> {
        self.presets.get(name)
    }

    /// Get all presets
    pub fn get_all_presets(&self) -> &HashMap<String, AvatarPreset> {
        &self.presets
    }

    /// Get presets by tags
    pub fn get_presets_by_tags(&self, tags: &[String]) -> Vec<&AvatarPreset> {
        self.presets.values()
            .filter(|preset| tags.iter().any(|tag| preset.tags.contains(tag)))
            .collect()
    }

    /// Get presets by difficulty
    pub fn get_presets_by_difficulty(&self, difficulty: PresetDifficulty) -> Vec<&AvatarPreset> {
        self.presets.values()
            .filter(|preset| std::mem::discriminant(&preset.difficulty) == std::mem::discriminant(&difficulty))
            .collect()
    }

    /// Save library to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.presets)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))
    }

    /// Load library from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        let presets: HashMap<String, AvatarPreset> = serde_json::from_str(json)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))?;

        Ok(Self { presets })
    }

    /// Create default presets
    fn create_default_presets() -> HashMap<String, AvatarPreset> {
        let mut presets = HashMap::new();

        // Hope Bearer preset
        presets.insert("Hope Bearer".to_string(), AvatarPreset {
            name: "Hope Bearer".to_string(),
            description: "An avatar infused with the essence of hope and light".to_string(),
            archetype: AvatarArchetype::Echo {
                echo_type: EchoType::Hope,
                energy_pattern: EnergyPattern::Flowing,
                manifestation: Manifestation::Ethereal,
            },
            customization: CustomizationData::default(),
            resonance: SongResonance {
                harmony_level: 0.8,
                primary_aspect: EchoType::Hope,
                secondary_aspects: vec![(EchoType::Light, 0.3)],
                silence_resistance: 0.6,
                resonance_history: ResonanceHistory::default(),
            },
            tags: vec!["echo".to_string(), "hope".to_string(), "beginner".to_string()],
            difficulty: PresetDifficulty::Beginner,
        });

        // Logic Seeker preset
        presets.insert("Logic Seeker".to_string(), AvatarPreset {
            name: "Logic Seeker".to_string(),
            description: "A methodical avatar that resonates with logic and order".to_string(),
            archetype: AvatarArchetype::Echo {
                echo_type: EchoType::Logic,
                energy_pattern: EnergyPattern::Crystalline,
                manifestation: Manifestation::Geometric,
            },
            customization: CustomizationData::default(),
            resonance: SongResonance {
                harmony_level: 0.7,
                primary_aspect: EchoType::Logic,
                secondary_aspects: vec![(EchoType::Order, 0.4)],
                silence_resistance: 0.5,
                resonance_history: ResonanceHistory::default(),
            },
            tags: vec!["echo".to_string(), "logic".to_string(), "intermediate".to_string()],
            difficulty: PresetDifficulty::Intermediate,
        });

        // Human Explorer preset
        presets.insert("Human Explorer".to_string(), AvatarPreset {
            name: "Human Explorer".to_string(),
            description: "A curious human ready to discover the world of echoes".to_string(),
            archetype: AvatarArchetype::Human {
                age_group: AgeGroup::YoungAdult,
                body_type: BodyType::Athletic,
                heritage: Heritage::Terran,
            },
            customization: CustomizationData::default(),
            resonance: SongResonance {
                harmony_level: 0.5,
                primary_aspect: EchoType::Hope,
                secondary_aspects: vec![],
                silence_resistance: 0.3,
                resonance_history: ResonanceHistory::default(),
            },
            tags: vec!["human".to_string(), "explorer".to_string(), "beginner".to_string()],
            difficulty: PresetDifficulty::Beginner,
        });

        // Hybrid Mystic preset
        presets.insert("Hybrid Mystic".to_string(), AvatarPreset {
            name: "Hybrid Mystic".to_string(),
            description: "A complex being balancing human intuition with echo wisdom".to_string(),
            archetype: AvatarArchetype::Hybrid {
                primary: Box::new(AvatarArchetype::Human {
                    age_group: AgeGroup::Adult,
                    body_type: BodyType::Average,
                    heritage: Heritage::Ancient,
                }),
                secondary: Box::new(AvatarArchetype::Echo {
                    echo_type: EchoType::Memory,
                    energy_pattern: EnergyPattern::Harmonic,
                    manifestation: Manifestation::Ethereal,
                }),
                blend_factor: 0.6,
            },
            customization: CustomizationData::default(),
            resonance: SongResonance {
                harmony_level: 0.6,
                primary_aspect: EchoType::Memory,
                secondary_aspects: vec![(EchoType::Hope, 0.2), (EchoType::Sorrow, 0.1)],
                silence_resistance: 0.7,
                resonance_history: ResonanceHistory::default(),
            },
            tags: vec!["hybrid".to_string(), "mystic".to_string(), "advanced".to_string()],
            difficulty: PresetDifficulty::Advanced,
        });

        presets
    }
}

impl Default for AvatarPresetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Network packet format for avatar data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarNetworkPacket {
    pub packet_type: AvatarPacketType,
    pub avatar_id: uuid::Uuid,
    pub data: AvatarNetworkData,
    pub timestamp: u64,
}

/// Types of avatar network packets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvatarPacketType {
    FullAvatar,
    AvatarUpdate,
    ResonanceUpdate,
    CustomizationUpdate,
    StatsUpdate,
}

/// Avatar data for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvatarNetworkData {
    Full(SerializableAvatar),
    Base(AvatarBase),
    Stats(AvatarStats),
    Resonance(SongResonance),
    Customization(CustomizationData),
}

impl AvatarNetworkPacket {
    /// Create a full avatar packet
    pub fn full_avatar(avatar: SerializableAvatar) -> Self {
        Self {
            packet_type: AvatarPacketType::FullAvatar,
            avatar_id: avatar.base.id,
            data: AvatarNetworkData::Full(avatar),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create an update packet
    pub fn stats_update(avatar_id: uuid::Uuid, stats: AvatarStats) -> Self {
        Self {
            packet_type: AvatarPacketType::StatsUpdate,
            avatar_id,
            data: AvatarNetworkData::Stats(stats),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Serialize for network transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))
    }

    /// Deserialize from network data
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AvatarError::InvalidData(e.to_string()))
    }
}

/// Simple hash function for checksums
fn calculate_simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for byte in data {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_serialization() {
        let avatar = SerializableAvatar::new(
            AvatarBase::default(),
            AvatarStats::default(),
            SongResonance::default(),
            CustomizationData::default(),
            None,
        );

        // Test binary serialization
        let binary_data = avatar.to_binary().unwrap();
        let deserialized = SerializableAvatar::from_binary(&binary_data).unwrap();
        assert_eq!(avatar.base.id, deserialized.base.id);

        // Test JSON serialization
        let json_data = avatar.to_json().unwrap();
        let deserialized_json = SerializableAvatar::from_json(&json_data).unwrap();
        assert_eq!(avatar.base.id, deserialized_json.base.id);
    }

    #[test]
    fn test_checksum_calculation() {
        let mut avatar = SerializableAvatar::new(
            AvatarBase::default(),
            AvatarStats::default(),
            SongResonance::default(),
            CustomizationData::default(),
            None,
        );

        avatar.update_metadata(None).unwrap();
        assert!(avatar.verify_checksum().unwrap());

        // Modify avatar and verify checksum fails
        avatar.base.name = "Modified".to_string();
        assert!(!avatar.verify_checksum().unwrap());
    }

    #[test]
    fn test_preset_library() {
        let library = AvatarPresetLibrary::new();

        assert!(library.get_preset("Hope Bearer").is_some());
        assert!(library.get_preset("Logic Seeker").is_some());
        assert!(library.get_preset("Nonexistent").is_none());

        let hope_presets = library.get_presets_by_tags(&["hope".to_string()]);
        assert!(!hope_presets.is_empty());

        let beginner_presets = library.get_presets_by_difficulty(PresetDifficulty::Beginner);
        assert!(!beginner_presets.is_empty());
    }

    #[test]
    fn test_preset_application() {
        let library = AvatarPresetLibrary::new();
        let preset = library.get_preset("Hope Bearer").unwrap();

        let avatar = preset.apply_to_avatar(Some("Test Character".to_string()));
        assert_eq!(avatar.base.name, "Test Character");

        if let AvatarArchetype::Echo { echo_type, .. } = avatar.base.archetype {
            assert_eq!(echo_type, EchoType::Hope);
        } else {
            panic!("Expected Echo archetype");
        }
    }

    #[test]
    fn test_network_packets() {
        let avatar = SerializableAvatar::new(
            AvatarBase::default(),
            AvatarStats::default(),
            SongResonance::default(),
            CustomizationData::default(),
            None,
        );

        let packet = AvatarNetworkPacket::full_avatar(avatar.clone());
        let bytes = packet.to_bytes().unwrap();
        let deserialized = AvatarNetworkPacket::from_bytes(&bytes).unwrap();

        assert_eq!(packet.avatar_id, deserialized.avatar_id);
        assert!(matches!(deserialized.packet_type, AvatarPacketType::FullAvatar));
    }

    #[test]
    fn test_version_migration() {
        let mut avatar = SerializableAvatar::new(
            AvatarBase::default(),
            AvatarStats::default(),
            SongResonance::default(),
            CustomizationData::default(),
            None,
        );

        // Test current version
        assert_eq!(avatar.version, 1);
        let migrated = avatar.clone().migrate_if_needed().unwrap();
        assert_eq!(migrated.version, 1);

        // Test version 0 migration
        avatar.version = 0;
        let migrated = avatar.migrate_if_needed().unwrap();
        assert_eq!(migrated.version, 1);
    }
}