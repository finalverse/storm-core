// File: crates/storm-opensim/src/lib.rs
// OpenSim protocol implementation
// Extended LLUDP protocol support for OpenSimulator

pub mod messages;
pub mod serialization;
pub mod login;
pub mod circuit;

pub use messages::*;
pub use serialization::*;
pub use login::*;
pub use circuit::*;

/// OpenSim protocol version information
pub const OPENSIM_PROTOCOL_VERSION: &str = "0.9.2";
pub const LLUDP_PROTOCOL_VERSION: u32 = 1;

/// OpenSim specific message types beyond base LLUDP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenSimMessageType {
    // Extended messaging
    GenericMessage = 245,
    MuteListRequest = 246,
    UpdateMuteListEntry = 247,
    RemoveMuteListEntry = 248,
    CopyInventoryFromNotecard = 250,

    // OpenSim extensions
    OpenSimExtras = 300,
    RegionHandshake = 301,
    EstateOwnerMessage = 302,
}

/// OpenSim region information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionInfo {
    pub region_id: uuid::Uuid,
    pub region_name: String,
    pub region_handle: u64,
    pub sim_ip: std::net::Ipv4Addr,
    pub sim_port: u16,
    pub region_size_x: u32,
    pub region_size_y: u32,
    pub region_flags: u32,
    pub water_height: f32,
    pub agent_limit: u32,
    pub object_bonus_factor: f32,
}

impl Default for RegionInfo {
    fn default() -> Self {
        Self {
            region_id: uuid::Uuid::new_v4(),
            region_name: "Unknown Region".to_string(),
            region_handle: 0,
            sim_ip: std::net::Ipv4Addr::new(127, 0, 0, 1),
            sim_port: 9000,
            region_size_x: 256,
            region_size_y: 256,
            region_flags: 0,
            water_height: 20.0,
            agent_limit: 100,
            object_bonus_factor: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_info_default() {
        let region = RegionInfo::default();
        assert_eq!(region.region_name, "Unknown Region");
        assert_eq!(region.region_size_x, 256);
        assert_eq!(region.water_height, 20.0);
    }
}