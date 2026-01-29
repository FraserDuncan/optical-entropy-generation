//! Optical Entropy Generation CLI
//!
//! Command-line interface for the optical entropy generation system.
//! Captures frames from a camera, extracts entropy, and reseeds a CSPRNG.

use clap::{Parser, Subcommand};
use optical_entropy::{
    analysis::HealthMonitor,
    capture::{Camera, CaptureConfig, MockCamera},
    conditioning::EntropyPool,
    extraction::Extractor,
    reseeding::ReseedableRng,
};
#[cfg(feature = "camera")]
use optical_entropy::capture::FileConfig;
use rand_core::RngCore;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "optical-entropy")]
#[command(about = "Physical entropy source using optical phenomena")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Camera device index (overrides config file)
    #[arg(short, long)]
    device: Option<u32>,

    /// Run continuously until interrupted
    #[arg(long)]
    continuous: bool,

    /// Number of frames to process (ignored if --continuous)
    #[arg(short = 'n', long, default_value = "100")]
    frames: u32,
}

#[derive(Subcommand)]
enum Commands {
    /// List available camera devices
    ListDevices,
    /// Run with mock camera for testing
    Mock {
        /// Number of frames to process
        #[arg(short = 'n', long, default_value = "20")]
        frames: u32,
    },
    /// Generate random bytes to stdout
    Generate {
        /// Number of bytes to generate
        #[arg(short = 'n', long, default_value = "32")]
        bytes: usize,
        /// Output as hex instead of raw bytes
        #[arg(long)]
        hex: bool,
    },
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::ListDevices) => list_devices(),
        Some(Commands::Mock { frames }) => run_mock(frames),
        Some(Commands::Generate { bytes, hex }) => {
            generate_random(&cli, bytes, hex);
        }
        None => run_capture(&cli),
    }
}

fn list_devices() {
    #[cfg(feature = "camera")]
    {
        use optical_entropy::capture::NokhwaCamera;
        match NokhwaCamera::list_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    println!("No camera devices found.");
                    println!("\nTroubleshooting:");
                    println!("  - Ensure camera is connected");
                    println!("  - Check permissions (add user to 'video' group on Linux)");
                    println!("  - Try: sudo usermod -aG video $USER");
                } else {
                    println!("Available cameras:");
                    for dev in devices {
                        println!("  [{}] {} - {}", dev.index, dev.name, dev.description);
                    }
                    println!("\nUsage: optical-entropy --device <index>");
                }
            }
            Err(e) => {
                eprintln!("Error listing devices: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(feature = "camera"))]
    {
        eprintln!("Camera support not compiled. Rebuild with:");
        eprintln!("  cargo build --features camera");
        std::process::exit(1);
    }
}

fn run_mock(frame_count: u32) {
    info!("Optical Entropy Generator v{}", optical_entropy::VERSION);
    info!("Running with mock camera (testing mode)");

    let config = CaptureConfig::default();
    let mut camera = MockCamera::new();

    if let Err(e) = camera.open(&config) {
        eprintln!("Failed to open mock camera: {}", e);
        std::process::exit(1);
    }

    run_pipeline(&mut camera, frame_count, false);
}

fn run_capture(#[allow(unused)] cli: &Cli) {
    info!("Optical Entropy Generator v{}", optical_entropy::VERSION);

    #[cfg(feature = "camera")]
    {
        use optical_entropy::capture::NokhwaCamera;

        // Load configuration
        let file_config = cli.config.as_ref().map(|path| {
            FileConfig::from_file(path).unwrap_or_else(|e| {
                eprintln!("Failed to load config file: {}", e);
                std::process::exit(1);
            })
        });

        let mut capture_config = file_config
            .as_ref()
            .map(|c| c.capture.clone())
            .unwrap_or_default();

        // CLI overrides
        if let Some(device_id) = cli.device {
            capture_config.device_id = device_id;
        }

        let frame_count = if cli.continuous {
            u32::MAX
        } else {
            cli.frames
        };

        info!("Opening camera device {}...", capture_config.device_id);
        let mut camera = NokhwaCamera::new();

        if let Err(e) = camera.open(&capture_config) {
            eprintln!("Failed to open camera: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  - Run 'optical-entropy list-devices' to see available cameras");
            eprintln!("  - Check camera permissions");
            eprintln!("  - Ensure no other application is using the camera");
            std::process::exit(1);
        }

        run_pipeline(&mut camera, frame_count, cli.continuous);
    }

    #[cfg(not(feature = "camera"))]
    {
        eprintln!("Camera support not compiled. Options:");
        eprintln!("  1. Rebuild with camera support:");
        eprintln!("     cargo build --release --features camera");
        eprintln!("  2. Use mock mode for testing:");
        eprintln!("     optical-entropy mock");
        std::process::exit(1);
    }
}

fn generate_random(#[allow(unused)] cli: &Cli, byte_count: usize, hex_output: bool) {
    // Silently initialize RNG and generate output
    let mut rng = ReseedableRng::from_os_entropy();

    // If we have camera support and a device, try to reseed from it first
    #[cfg(feature = "camera")]
    if cli.device.is_some() || cli.config.is_some() {
        // Quick reseed from camera
        let capture_config = cli
            .config
            .as_ref()
            .and_then(|p| FileConfig::from_file(p).ok())
            .map(|c| c.capture)
            .unwrap_or_default();

        use optical_entropy::capture::NokhwaCamera;
        let mut camera = NokhwaCamera::new();
        if camera.open(&capture_config).is_ok() {
            let mut extractor = Extractor::new();
            let mut pool = EntropyPool::default();
            let mut health = HealthMonitor::default();

            // Collect enough entropy
            for _ in 0..50 {
                if let Ok(frame) = camera.capture() {
                    if let Some(bits) = extractor.process(&frame) {
                        let metrics = health.analyze(&bits);
                        if metrics.is_healthy {
                            pool.add(&bits);
                        }
                    }
                }
                if pool.is_ready() && health.allow_reseed() {
                    break;
                }
            }

            if let Some(seed) = pool.extract() {
                let _ = rng.reseed(&seed);
            }
        }
    }

    let mut output = vec![0u8; byte_count];
    rng.fill_bytes(&mut output);

    if hex_output {
        println!("{}", output.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    } else {
        use std::io::Write;
        std::io::stdout().write_all(&output).unwrap();
    }
}

fn run_pipeline<C: Camera>(camera: &mut C, frame_count: u32, continuous: bool) {
    let mut extractor = Extractor::new();
    let mut pool = EntropyPool::default();
    let mut health = HealthMonitor::default();
    let mut rng = ReseedableRng::from_os_entropy();

    info!("Processing frames...");

    let mut healthy_count = 0u64;
    let mut unhealthy_count = 0u64;
    let mut total_reseeds = 0u64;

    // Set up Ctrl+C handler for continuous mode
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    if continuous {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, std::sync::atomic::Ordering::SeqCst);
        })
        .ok();
    }

    let mut i = 0u32;
    while (continuous && running.load(std::sync::atomic::Ordering::SeqCst))
        || (!continuous && i < frame_count)
    {
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

                // Attempt reseeding when pool is ready
                if health.allow_reseed() && pool.is_ready() {
                    if let Some(seed) = pool.extract() {
                        match rng.reseed(&seed) {
                            Ok(()) => {
                                total_reseeds += 1;
                                info!(
                                    "CSPRNG reseeded (#{}, entropy: {} bits)",
                                    total_reseeds,
                                    seed.entropy_estimate()
                                );
                            }
                            Err(e) => {
                                warn!("Reseed failed: {}", e);
                            }
                        }
                    }
                }
            } else {
                unhealthy_count += 1;
                if let Some(ref violation) = metrics.last_violation {
                    if unhealthy_count % 100 == 1 {
                        warn!("Quality violation: {}", violation);
                    }
                }
            }
        }

        i = i.saturating_add(1);

        // Periodic status update
        if i % 1000 == 0 && continuous {
            info!(
                "Status: {} frames, {} healthy, {} unhealthy, {} reseeds",
                i, healthy_count, unhealthy_count, total_reseeds
            );
        }
    }

    info!(
        "Finished: {} frames processed, {} healthy, {} unhealthy",
        healthy_count + unhealthy_count,
        healthy_count,
        unhealthy_count
    );
    info!("Total reseeds: {}", total_reseeds);

    // Generate sample output
    info!("Sample random output:");
    let mut output = [0u8; 32];
    rng.fill_bytes(&mut output);
    println!(
        "{}",
        output.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    );
}
