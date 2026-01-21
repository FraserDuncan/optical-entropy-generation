//! Optical Entropy Generation CLI
//!
//! Command-line interface for testing and demonstrating the optical
//! entropy generation system.

use optical_entropy::{
    analysis::HealthMonitor,
    capture::{Camera, CaptureConfig, MockCamera},
    conditioning::EntropyPool,
    extraction::Extractor,
    reseeding::ReseedableRng,
};
use rand_core::RngCore;
use tracing::{info, warn};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Optical Entropy Generator v{}", optical_entropy::VERSION);
    info!("This is a demonstration using mock camera input");

    // Initialize components
    let config = CaptureConfig::default();
    let mut camera = MockCamera::new();

    if let Err(e) = camera.open(&config) {
        eprintln!("Failed to open camera: {}", e);
        std::process::exit(1);
    }

    let mut extractor = Extractor::new();
    let mut pool = EntropyPool::default();
    let mut health = HealthMonitor::default();
    let mut rng = ReseedableRng::from_os_entropy();

    info!("Processing frames...");

    // Process frames
    let frame_count = 20;
    let mut healthy_count = 0;
    let mut unhealthy_count = 0;

    for i in 0..frame_count {
        let frame = match camera.capture() {
            Ok(f) => f,
            Err(e) => {
                warn!("Frame capture failed: {}", e);
                continue;
            }
        };

        if let Some(bits) = extractor.process(&frame) {
            let metrics = health.analyze(&bits);

            if metrics.is_healthy {
                healthy_count += 1;
                pool.add(&bits);
            } else {
                unhealthy_count += 1;
                if let Some(ref violation) = metrics.last_violation {
                    warn!("Frame {}: quality violation: {}", i, violation);
                }
            }
        }
    }

    info!(
        "Processed {} frames: {} healthy, {} unhealthy",
        frame_count, healthy_count, unhealthy_count
    );

    // Attempt reseeding
    if health.allow_reseed() && pool.is_ready() {
        if let Some(seed) = pool.extract() {
            match rng.reseed(&seed) {
                Ok(()) => {
                    info!(
                        "CSPRNG reseeded successfully (entropy estimate: {} bits)",
                        seed.entropy_estimate()
                    );
                }
                Err(e) => {
                    warn!("Reseed failed: {}", e);
                }
            }
        }
    } else {
        warn!("Reseeding not performed: health={}, pool_ready={}",
            health.allow_reseed(), pool.is_ready());
    }

    // Generate some random output to demonstrate
    info!("Generating sample random output:");
    let mut output = [0u8; 32];
    rng.fill_bytes(&mut output);

    println!(
        "Random bytes: {}",
        output
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    );

    info!("Done. Reseed count: {}", rng.reseed_count());
}
