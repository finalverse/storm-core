// File: examples/opensim-client/src/main.rs
// Example OpenSim client using StormCore
// Demonstrates connecting to an OpenSimulator grid

use storm_core::{StormCore, StormConfig, init_logging, core::WorldConfig, core::ProtocolType};
use tracing::{info, error};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    init_logging();
    info!("Starting OpenSim client example");

    // Create engine configuration
    let mut config = StormConfig::default();
    config.enable_rendering = false; // Headless for this example
    config.enable_audio = false;
    config.debug_mode = true;

    // Create the engine
    info!("Initializing StormCore engine...");
    let engine = StormCore::new(config).await?;

    // Configure world connection
    let world_config = WorldConfig::new_opensim(
        "OSGrid",
        "http://login.osgrid.org",
        "TestUser",
        "password123",
    );

    // Connect to the world
    info!("Connecting to OpenSim world: {}", world_config.name);
    match engine.connect_to_world(&world_config).await {
        Ok(_) => info!("Successfully connected to world!"),
        Err(e) => {
            error!("Failed to connect to world: {}", e);
            return Err(e.into());
        }
    }

    // Main update loop
    info!("Starting main update loop...");
    let mut frame_count = 0;
    let target_fps = 60.0;
    let frame_duration = Duration::from_secs_f32(1.0 / target_fps);

    loop {
        let frame_start = std::time::Instant::now();

        // Update the engine
        if let Err(e) = engine.update(1.0 / target_fps).await {
            error!("Engine update failed: {}", e);
            break;
        }

        frame_count += 1;
        if frame_count % 60 == 0 {
            info!("Running... Frame: {}", frame_count);
        }

        // Exit after 10 seconds for demo
        if frame_count > 600 {
            info!("Demo complete, shutting down...");
            break;
        }

        // Frame rate limiting
        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            tokio::time::sleep(frame_duration - frame_time).await;
        }
    }

    // Shutdown gracefully
    info!("Shutting down engine...");
    engine.shutdown().await?;
    info!("Shutdown complete");

    Ok(())
}