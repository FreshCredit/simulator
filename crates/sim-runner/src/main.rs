//! sim-runner CLI: Batch execution harness for Agent Society Simulator
//!
//! # Usage
//!
//! ```bash
//! # Run with default config
//! sim-runner run
//!
//! # Run with config file
//! sim-runner run --config experiment.toml
//!
//! # Run with inline parameters
//! sim-runner run --agents 1000 --ticks 2000 --stage 5 --replications 10
//!
//! # Parameter sweep
//! sim-runner sweep --parameter agent_count --min 100 --max 1000 --steps 10
//!
//! # Morris screening
//! sim-runner morris-screen --trajectories 10 --levels 4
//!
//! # Run specific experiment
//! sim-runner experiment lambda-universality --output ./results
//!
//! # List all experiments
//! sim-runner experiment --list
//! ```

use clap::{Parser, Subcommand, ValueEnum};
use sim_report::{ReportConfig, ReportGenerator, SimulationMetadata};
use sim_runner::{
    batch::BatchRunner,
    config::RunConfig,
    experiment::{ExperimentConfig, ExperimentRunner, ExperimentType},
    hypothesis::GttHypothesisRunner,
    sweep::{MorrisScreener, SweepRunner},
    visualization::generate_experiment_pngs,
};
use std::path::PathBuf;

/// CLI-friendly wrapper for `ExperimentType`
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExperimentCliType {
    /// GTT-P0-1: Lambda universality validation
    LambdaUniversality,
    /// GTT-P0-6: Stability law validation
    StabilityLaw,
    /// GTT-P0-5: Cross-domain correlation
    CrossDomainCorrelation,
    /// DOCAS-P0-2: Twin heritability
    // TAG: surface=simulation owner=research-team rule=SIM-001
    TwinHeritability,
    /// DOCAS-P0-3: Criticality sigma
    CriticalitySigma,
    /// PDE4B-P0-1: PDE4B-lambda correlation
    Pde4bLambdaCorrelation,
    /// PDE4B-P0-2: PDE4B pharmacology
    Pde4bPharmacology,
    /// EMOT-P0-1: Emotion mapping
    EmotionMapping,
    /// EMOT-P0-3: Emotion pharmacology
    EmotionPharmacology,
    /// LING-P0-1: Semantic network (trust centrality)
    SemanticNetwork,
    /// LING-P0-2: Semantic drift
    SemanticDrift,
    /// PHIL-P0-2: Philosophical oscillation
    PhilosophicalOscillation,
    /// CULT-P0-1: Cultural quadrants
    CulturalQuadrants,
    /// CULT-P0-3: Hofstede prediction
    HofstedePrediction,
    /// ECON-P0-1: GDP prediction
    GdpPrediction,
    /// ECON-P0-2: Crisis threshold
    CrisisThreshold,
    /// GAME-P0-1: Trust-weighted prisoner's dilemma
    TrustWeightedPD,
    /// Game Theory: Trust-weighted ultimatum
    TrustWeightedUltimatum,
    /// GAME-P0-3: Evolutionary dynamics
    EvolutionaryDynamics,
    /// TFP-P0-3: Psi-trust AUC
    PsiTrustAUC,
    /// TFP: Entropy reduction
    EntropyReduction,
    /// TFP-P0-4: Cost model
    CostModel,
    /// MEAN-P0-3: Percolation threshold
    PercolationThreshold,
    /// MEAN-P0-2: Civilizational collapse
    CivilizationalCollapse,
    /// MUSIC-P0-2: Spectral analysis
    SpectralCoherence,
    /// MUSIC-P0-3: Music-trust correlation
    MusicTrustCorrelation,
    /// DOCAS v4.0: Sim O1 Dopamine Lesion (ArchT 566nm)
    #[clap(name = "sim-o1", alias = "sim_o1")]
    SimO1DopamineLesion,
    /// DOCAS v4.0: Sim O2 Cortisol Threat Excitation (ChR2 470nm)
    #[clap(name = "sim-o2", alias = "sim_o2")]
    SimO2InducedVulnerability,
    /// TFP v4.0: Sim O3 Kuramoto O/S Mutant Fragmentation
    #[clap(name = "sim-o3", alias = "sim_o3")]
    SimO3KuramotoFragmentation,
    /// DOCAS v4.0: Sim D16 Dual-Branch Visual Cloaking
    #[clap(name = "sim-d16", alias = "sim_d16")]
    SimD16VisualCloaking,
    /// DOCAS v4.0: Sim D17 Atmospheric Gating & Weather-Shield
    #[clap(name = "sim-d17", alias = "sim_d17")]
    SimD17WeatherGating,
    /// DOCAS v4.0: Sim D18 Baylor Unconscious Narrative Gating
    #[clap(name = "sim-d18", alias = "sim_d18")]
    SimD18BaylorUnconsciousGating,
}

impl ExperimentCliType {
    const fn to_experiment_type(self) -> ExperimentType {
        match self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::LambdaUniversality => ExperimentType::LambdaUniversality,
            Self::StabilityLaw => ExperimentType::StabilityLaw,
            Self::CrossDomainCorrelation => ExperimentType::CrossDomainCorrelation,
            Self::TwinHeritability => ExperimentType::TwinHeritability {
                n_mz: 1000,
                n_dz: 1000,
            },
            Self::CriticalitySigma => ExperimentType::CriticalitySigma,
            Self::Pde4bLambdaCorrelation => ExperimentType::Pde4bLambdaCorrelation,
            Self::Pde4bPharmacology => ExperimentType::Pde4bPharmacology,
            Self::EmotionMapping => ExperimentType::EmotionMapping,
            Self::EmotionPharmacology => ExperimentType::EmotionPharmacology,
            Self::SemanticNetwork => ExperimentType::SemanticNetwork,
            Self::SemanticDrift => ExperimentType::SemanticDrift,
            Self::PhilosophicalOscillation => ExperimentType::PhilosophicalOscillation,
            Self::CulturalQuadrants => ExperimentType::CulturalQuadrants,
            Self::HofstedePrediction => ExperimentType::HofstedePrediction,
            Self::GdpPrediction => ExperimentType::GdpPrediction,
            Self::CrisisThreshold => ExperimentType::CrisisThreshold,
            Self::TrustWeightedPD => ExperimentType::TrustWeightedPD,
            Self::TrustWeightedUltimatum => ExperimentType::TrustWeightedUltimatum,
            Self::EvolutionaryDynamics => ExperimentType::EvolutionaryDynamics,
            Self::PsiTrustAUC => ExperimentType::PsiTrustAUC,
            Self::EntropyReduction => ExperimentType::EntropyReduction,
            Self::CostModel => ExperimentType::CostModel,
            Self::PercolationThreshold => ExperimentType::PercolationThreshold,
            Self::CivilizationalCollapse => ExperimentType::CivilizationalCollapse,
            Self::SpectralCoherence => ExperimentType::SpectralCoherence,
            Self::MusicTrustCorrelation => ExperimentType::MusicTrustCorrelation,
            Self::SimO1DopamineLesion => ExperimentType::SimO1DopamineLesion,
            Self::SimO2InducedVulnerability => ExperimentType::SimO2InducedVulnerability,
            Self::SimO3KuramotoFragmentation => ExperimentType::SimO3KuramotoFragmentation,
            Self::SimD16VisualCloaking => ExperimentType::SimD16VisualCloaking,
            Self::SimD17WeatherGating => ExperimentType::SimD17WeatherGating,
            Self::SimD18BaylorUnconsciousGating => ExperimentType::SimD18BaylorUnconsciousGating,
        }
    }
}

#[derive(Parser)]
#[command(name = "sim-runner")]
#[command(about = "Batch execution harness for Agent Society Simulator")]
#[command(version = sim_runner::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Subcommand)]
enum Commands {
    /// Run batch simulation
    Run {
        /// Configuration file (TOML or JSON)
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Number of agents
        #[arg(short, long, default_value_t = 100)]
        agents: u32,

        /// Maximum ticks
        #[arg(short, long, default_value_t = 1000)]
        ticks: u64,

        /// Stage (0-5)
        #[arg(short, long, default_value_t = 0)]
        stage: u8,

        /// Number of replications
        #[arg(short, long, default_value_t = 1)]
        replications: u32,

        /// Master random seed
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Run sequentially (disable parallelism)
        #[arg(long)]
        sequential: bool,
    },

    /// Run parameter sweep
    Sweep {
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        /// Parameter to sweep
        #[arg(short, long)]
        parameter: String,

        /// Minimum value
        #[arg(long)]
        min: f64,

        /// Maximum value
        #[arg(long)]
        max: f64,

        /// Number of steps
        #[arg(long, default_value_t = 10)]
        steps: usize,

        /// Replications per point
        #[arg(short, long, default_value_t = 3)]
        replications: u32,

        /// Use logarithmic spacing
        #[arg(long)]
        log_scale: bool,
    },

    /// Run Morris parameter screening
    MorrisScreen {
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Number of trajectories (r)
        #[arg(short, long, default_value_t = 10)]
        trajectories: usize,

        /// Number of levels (p)
        #[arg(short, long, default_value_t = 4)]
        levels: usize,

        /// Replications per evaluation
        #[arg(short, long, default_value_t = 1)]
        replications: u32,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        /// Output file for results (JSON)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run GTT hypothesis test (Protocol ON vs OFF)
    Hypothesis {
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Number of runs per condition
        #[arg(short, long, default_value_t = 20)]
        runs: u32,

        /// Master random seed
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// Quick test mode (reduced parameters)
        #[arg(long)]
        quick: bool,

        /// Output file for results (JSON)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run a specific experiment from the 26-experiment suite
    Experiment {
        /// Experiment type to run
        #[arg(value_enum)]
        experiment: Option<ExperimentCliType>,

        /// Output directory for results
        #[arg(short, long, default_value = "./experiments")]
        output: PathBuf,

        /// Override default agent count
        #[arg(long)]
        agents: Option<usize>,

        // TAG: surface=simulation owner=research-team rule=SIM-001
        /// Override default tick count
        #[arg(long)]
        ticks: Option<u64>,

        /// Override default replications
        #[arg(short, long)]
        replications: Option<u32>,

        /// Master random seed
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// List all available experiments
        #[arg(long)]
        list: bool,
    },

    /// Generate PNG visualizations from experiment results
    Visualize {
        /// Path to manifest.json file
        #[arg(short, long)]
        manifest: PathBuf,

        /// Output directory for PNG files (defaults to manifest directory)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },

    /// Validate the 8 "ready" experiments against acceptance criteria
    Validate {
        /// Output directory for results
        #[arg(short, long, default_value = "./validation")]
        output: PathBuf,

        /// Quick mode: reduced replications for testing
        #[arg(long)]
        quick: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let result = match cli.command {
        Commands::Run {
            config,
            agents,
            ticks,
            stage,
            replications,
            seed,
            output,
            sequential,
        } => run_batch(
            config,
            agents,
            ticks,
            stage,
            replications,
            seed,
            output,
            sequential,
        ),

        Commands::Sweep {
            config,
            parameter,
            min,
            max,
            steps,
            replications,
            log_scale,
        } => run_sweep(config, &parameter, min, max, steps, replications, log_scale),

        Commands::MorrisScreen {
            config,
            trajectories,
            levels,
            replications,
            output,
        } => run_morris(config, trajectories, levels, replications, output),

        Commands::Hypothesis {
            config,
            runs,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            seed,
            quick,
            output,
        } => run_hypothesis(config, runs, seed, quick, output),

        Commands::Experiment {
            experiment,
            output,
            agents,
            ticks,
            replications,
            seed,
            list,
        } => {
            if list {
                list_experiments();
                Ok(())
            } else if let Some(exp) = experiment {
                run_single_experiment(exp, output, agents, ticks, replications, seed)
            } else {
                eprintln!("Error: Must specify --list or an experiment type");
                eprintln!("Use --list to see all available experiments");
                std::process::exit(1);
            }
        }

        Commands::Visualize {
            manifest,
            output_dir,
        } => run_visualize(manifest, output_dir),

        Commands::Validate { output, quick } => run_validate(output, quick),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_batch(
    // TAG: surface=simulation owner=research-team rule=SIM-001
    config_path: Option<PathBuf>,
    agents: u32,
    ticks: u64,
    stage: u8,
    replications: u32,
    seed: u64,
    output: Option<PathBuf>,
    sequential: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-runner: Batch Execution Harness                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Load or create config
    let mut config = if let Some(path) = config_path {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "toml" => RunConfig::from_toml_file(&path)?,
            "json" => RunConfig::from_json_file(&path)?,
            _ => return Err(format!("Unknown config format: {ext}").into()),
        }
    } else {
        let mut c = RunConfig::default();
        c.simulation.agent_count = agents;
        c.simulation.max_ticks = ticks;
        c.simulation.stage = stage;
        c.simulation.seed = seed;
        c
    };

    config.batch.replications = replications;
    config.batch.parallel = !sequential;

    if let Some(out) = output {
        config.output.dir = out.to_string_lossy().to_string();
    }

    let agent_count = config.simulation.agent_count;
    let max_ticks = config.simulation.max_ticks;
    let stage = config.simulation.stage;
    let sim_seed = config.simulation.seed;
    let parallel = config.batch.parallel;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let output_dir = &config.output.dir;

    println!("Configuration:");
    println!("  Agents:       {agent_count}");
    println!("  Ticks:        {max_ticks}");
    println!("  Stage:        {stage}");
    println!("  Seed:         {sim_seed}");
    println!("  Replications: {replications}");
    println!("  Parallel:     {parallel}");
    if !output_dir.is_empty() {
        println!("  Output:       {output_dir}");
    }
    println!();

    let runner = BatchRunner::new(config);
    let result = runner.run(replications)?;

    result.print_summary();

    println!("✓ Batch execution complete. SIM-004: Deterministic (seed={seed})");

    Ok(())
}

fn run_sweep(
    config_path: Option<PathBuf>,
    parameter: &str,
    min: f64,
    max: f64,
    steps: usize,
    replications: u32,
    log_scale: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-runner: Parameter Sweep                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let mut config = if let Some(path) = config_path {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "toml" => RunConfig::from_toml_file(&path)?,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            "json" => RunConfig::from_json_file(&path)?,
            _ => return Err(format!("Unknown config format: {ext}").into()),
        }
    } else {
        RunConfig::default()
    };

    // Disable output for sweep runs
    config.output.dir = String::new();

    let scale = if log_scale { "logarithmic" } else { "linear" };
    println!("Sweep Configuration:");
    println!("  Parameter:    {parameter}");
    println!("  Range:        {min} to {max}");
    println!("  Steps:        {steps}");
    println!("  Scale:        {scale}");
    println!("  Replications: {replications}");
    println!();

    let values = if log_scale {
        SweepRunner::log_values(min, max, steps)
    } else {
        SweepRunner::linear_values(min, max, steps)
    };

    let runner = SweepRunner::new(config);
    let result = runner.run_sweep(parameter, &values, replications)?;

    result.print_summary();

    let sensitivity = result.sensitivity();
    let gini_range = sensitivity.gini_range;
    let br_range = sensitivity.branching_ratio_range;
    println!("Sensitivity:");
    println!("  Gini range:      {gini_range:.4}");
    println!("  BR range:        {br_range:.4}");
    println!();

    println!("✓ Sweep complete.");

    Ok(())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn run_morris(
    config_path: Option<PathBuf>,
    trajectories: usize,
    levels: usize,
    replications: u32,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-runner: Morris Screening (§4.4.1)              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let mut config = if let Some(path) = config_path {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "toml" => RunConfig::from_toml_file(&path)?,
            "json" => RunConfig::from_json_file(&path)?,
            _ => return Err(format!("Unknown config format: {ext}").into()),
        }
    } else {
        let mut c = RunConfig::default();
        c.simulation.agent_count = 100;
        c.simulation.max_ticks = 200;
        c
    };

    // Disable output for Morris runs
    config.output.dir = String::new();

    println!("Morris Configuration:");
    println!("  Trajectories: {trajectories}");
    println!("  Levels:       {levels}");
    println!("  Replications: {replications}");
    println!();

    let screener = MorrisScreener::new(config);
    let result = screener.screen(trajectories, levels, replications)?;

    result.print_summary();

    // Write output if specified
    if let Some(out_path) = output {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let json = serde_json::json!({
            "trajectories": result.trajectories,
            "levels": result.levels,
            "parameters": result.parameters.iter().map(|p| {
                serde_json::json!({
                    "parameter": p.parameter,
                    "mu_star": p.mu_star,
                    "sigma": p.sigma,
                    "is_inactive": p.is_inactive
                })
            }).collect::<Vec<_>>()
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&json)?)?;
        let out_display = out_path.display();
        println!("Results written to: {out_display}");
    }

    let inactive = result.inactive_parameters();
    println!();
    println!("Inactive parameters: {inactive:?}");
    println!();
    println!("✓ Morris screening complete.");

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn run_hypothesis(
    config_path: Option<PathBuf>,
    runs: u32,
    seed: u64,
    quick: bool,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-runner: GTT Hypothesis Test                    ║");
    println!("║     \"Protocol ON increases trust (τ) vs Protocol OFF\"        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let config = if let Some(path) = config_path {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "toml" => RunConfig::from_toml_file(&path)?,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            "json" => RunConfig::from_json_file(&path)?,
            _ => return Err(format!("Unknown config format: {ext}").into()),
        }
    } else {
        let mut c = RunConfig::default();
        if quick {
            c.simulation.agent_count = 50;
            c.simulation.max_ticks = 200;
        }
        c
    };

    let actual_runs = if quick { 5 } else { runs };

    println!("Configuration:");
    println!("  Runs per condition: {actual_runs}");
    println!("  Master seed:        {seed}");
    println!("  Agents:             {}", config.simulation.agent_count);
    println!("  Ticks:              {}", config.simulation.max_ticks);
    println!(
        "  Mode:               {}",
        if quick { "quick" } else { "full" }
    );
    println!();

    let runner = GttHypothesisRunner::new(config.clone());
    let result = runner.run(actual_runs, seed)?;

    // Print summary report
    println!("{}", result.summary_report());

    // Generate complete report using sim-report
    // Convert result to sim-report format via JSON serialization
    let report_result: sim_report::GttHypothesisResult =
        serde_json::from_str(&result.to_json().unwrap_or_default()).unwrap_or_else(|_| {
            sim_report::GttHypothesisResult {
                n_runs: result.n_runs,
                seed: result.seed,
                protocol_on: sim_report::ConditionResults {
                    n: result.protocol_on.n,
                    mean: result.protocol_on.mean,
                    std: result.protocol_on.std,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    values: result.protocol_on.values.clone(),
                },
                protocol_off: sim_report::ConditionResults {
                    n: result.protocol_off.n,
                    mean: result.protocol_off.mean,
                    std: result.protocol_off.std,
                    values: result.protocol_off.values.clone(),
                },
                effect_size: result.effect_size,
                effect_size_ci_lower: result.effect_size_ci_lower,
                effect_size_ci_upper: result.effect_size_ci_upper,
                effect_size_interpretation: result.effect_size_interpretation.clone(),
                t_statistic: result.t_statistic,
                p_value: result.p_value,
                hypothesis_supported: result.hypothesis_supported,
                confidence_level: result.confidence_level,
            }
        });

    let report_config = ReportConfig::default();
    let report_gen = ReportGenerator::new(report_config);

    let metadata = SimulationMetadata {
        simulation_id: format!("gtt-{}-{}", seed, chrono::Utc::now().timestamp()),
        description: "GTT Hypothesis Test: Protocol ON vs OFF".to_string(),
        agent_count: config.simulation.agent_count,
        max_ticks: config.simulation.max_ticks,
        seed,
        stage: config.simulation.stage,
        verification_mode: format!("{:?}", config.simulation.verification_mode),
        ..Default::default()
    };

    let full_report = report_gen.generate_full_report(None, Some(&report_result), metadata);

    // Write outputs
    if let Some(out_path) = &output {
        // Write JSON results
        let json = result.to_json()?;
        std::fs::write(out_path, json)?;
        println!("Results written to: {}", out_path.display());

        // Write Markdown report
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let md_path = out_path.with_extension("md");
        let md_report = full_report.to_markdown();
        std::fs::write(&md_path, md_report)?;
        println!("Report written to: {}", md_path.display());
    }

    // Exit with error code if hypothesis not supported
    if !result.hypothesis_supported {
        eprintln!("⚠️  GTT hypothesis NOT SUPPORTED by data");
        std::process::exit(1);
    }

    println!("✓ GTT hypothesis test complete.");

    Ok(())
}

/// List all available experiments
const EXPERIMENTS: [(&str, &str, &str, &str, &str, &str); 32] = [
    (
        "lambda-universality",
        "GTT-P0-1",
        "Lambda universality validation",
        "10,000",
        "365",
        "10,000",
    ),
    (
        "stability-law",
        "GTT-P0-6",
        "Stability law validation",
        "5,000",
        "365",
        "10,000",
    ),
    (
        "cross-domain-correlation",
        "GTT-P0-5",
        "Cross-domain correlation",
        "10,000",
        "365",
        "10,000",
        // TAG: surface=simulation owner=research-team rule=SIM-001
    ),
    (
        "twin-heritability",
        "DOCAS-P0-2",
        "Twin heritability",
        "2,000 pairs",
        "365",
        "2,000",
    ),
    (
        "criticality-sigma",
        "DOCAS-P0-3",
        "Criticality sigma",
        "5,000",
        "365",
        "5,000",
    ),
    (
        "pde4b-lambda-correlation",
        "PDE4B-P0-1",
        "PDE4B-lambda correlation",
        "10,000",
        "365",
        "5,000",
    ),
    (
        "pde4b-pharmacology",
        "PDE4B-P0-2",
        "PDE4B pharmacology",
        "5,000",
        "365",
        "2,000",
    ),
    (
        "emotion-mapping",
        "EMOT-P0-1",
        "Emotion mapping",
        "5,000",
        "365",
        "5,000",
    ),
    (
        "emotion-pharmacology",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "EMOT-P0-3",
        "Emotion pharmacology",
        "5,000",
        "365",
        "2,000",
    ),
    (
        "semantic-network",
        "LING-P0-1",
        "Semantic network (trust centrality)",
        "5,000",
        "365",
        "5,000",
    ),
    (
        "semantic-drift",
        "LING-P0-2",
        "Semantic drift",
        "5,000",
        "730",
        "5,000",
    ),
    (
        "philosophical-oscillation",
        "PHIL-P0-2",
        "Philosophical oscillation",
        "10,000",
        "3,650",
        "1,000",
    ),
    (
        "cultural-quadrants",
        "CULT-P0-1",
        "Cultural quadrants",
        "10,000",
        "1,825",
        "2,000",
    ),
    (
        "hofstede-prediction",
        "CULT-P0-3",
        "Hofstede prediction",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "10,000",
        "1,825",
        "2,000",
    ),
    (
        "gdp-prediction",
        "ECON-P0-1",
        "GDP prediction",
        "10,000",
        "1,825",
        "5,000",
    ),
    (
        "crisis-threshold",
        "ECON-P0-2",
        "Crisis threshold",
        "10,000",
        "1,825",
        "5,000",
    ),
    (
        "trust-weighted-pd",
        "GAME-P0-1",
        "Trust-weighted prisoner's dilemma",
        "1,000",
        "1,000",
        "10,000",
    ),
    (
        "trust-weighted-ultimatum",
        "-",
        "Trust-weighted ultimatum",
        "1,000",
        "1,000",
        "10,000",
    ),
    (
        "evolutionary-dynamics",
        "GAME-P0-3",
        "Evolutionary dynamics",
        "5,000",
        "1,000",
        "5,000",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ),
    (
        "psi-trust-auc",
        "TFP-P0-3",
        "Psi-trust AUC",
        "10,000",
        "365",
        "10,000",
    ),
    (
        "entropy-reduction",
        "-",
        "Entropy reduction",
        "10,000",
        "365",
        "10,000",
    ),
    (
        "cost-model",
        "TFP-P0-4",
        "Cost model",
        "100,000",
        "365",
        "1,000",
    ),
    (
        "percolation-threshold",
        "MEAN-P0-3",
        "Percolation threshold",
        "10,000",
        "365",
        "5,000",
    ),
    (
        "civilizational-collapse",
        "MEAN-P0-2",
        "Civilizational collapse",
        "10,000",
        "365",
        "5,000",
    ),
    (
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "spectral-coherence",
        "MUSIC-P0-2",
        "Spectral analysis",
        "5,000",
        "365",
        "5,000",
    ),
    (
        "music-trust-correlation",
        "MUSIC-P0-3",
        "Music-trust correlation",
        "5,000",
        "365",
        "5,000",
    ),
    (
        "sim-o1",
        "DOCAS-v4-O1",
        "Sim O1: Dopamine Lesion (ArchT 566nm)",
        "1,000",
        "100",
        "1",
    ),
    (
        "sim-o2",
        "DOCAS-v4-O2",
        "Sim O2: Cortisol Threat Excitation (ChR2 470nm)",
        "1,000",
        "100",
        "1",
    ),
    (
        "sim-o3",
        "TFP-v4-O3",
        "Sim O3: Kuramoto O/S Mutant Fragmentation",
        "100",
        "50",
        "1",
    ),
    (
        "sim-d16",
        "DOCAS-v4-D16",
        "Sim D16: Dual-Branch Visual Cloaking (38.9x Gradient)",
        "1,000",
        "100",
        "1",
    ),
    (
        "sim-d17",
        "DOCAS-v4-D17",
        "Sim D17: Atmospheric Gating & Weather-Shield",
        "1,000",
        "100",
        "1",
    ),
    (
        "sim-d18",
        "DOCAS-v4-D18",
        "Sim D18: Baylor Unconscious Narrative Gating",
        "1,000",
        "100",
        "1",
    ),
];

fn list_experiments() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           Available Experiments (32 Total)                       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "{:<30} {:<12} {:<40} {:>10} {:>8} {:>10}",
        "Experiment", "Gap ID", "Description", "Agents", "Ticks", "Reps"
    );
    println!("{}", "─".repeat(120));
    for (name, gap, desc, agents, ticks, reps) in &EXPERIMENTS {
        println!("{name:<30} {gap:<12} {desc:<40} {agents:>10} {ticks:>8} {reps:>10}");
    }
    println!();
    println!("Usage: sim-runner experiment <EXPERIMENT-NAME> --output ./results");
    println!();
    println!("Options:");
    println!("  --agents N       Override default agent count");
    println!("  --ticks N        Override default tick count");
    println!("  --replications N Override default replication count");
    println!("  --seed N         Set master random seed (default: 42)");
}

/// Run a single experiment
#[allow(clippy::needless_pass_by_value)]
fn run_single_experiment(
    exp_cli: ExperimentCliType,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    output: PathBuf,
    agents: Option<usize>,
    ticks: Option<u64>,
    replications: Option<u32>,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let exp_type = exp_cli.to_experiment_type();
    let exp_name = format!("{exp_type:?}").to_lowercase();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-runner: Experiment Execution                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Experiment: {exp_name}");
    println!("Paper:      {}", exp_type.paper());
    println!("Gaps:       {:?}", exp_type.gap_ids());
    println!();

    let mut config = ExperimentConfig::for_experiment(exp_type);
    if let Some(a) = agents {
        config.agent_count = a;
    }
    if let Some(t) = ticks {
        config.tick_count = t;
    }
    if let Some(r) = replications {
        config.replications = r as usize;
    }
    config.seed_base = seed;
    config.output_dir = output.clone();

    println!("Configuration:");
    println!("  Agents:       {}", config.agent_count);
    println!("  Ticks:        {}", config.tick_count);
    println!("  Replications: {}", config.replications);
    println!("  Seed:         {}", config.seed_base);
    println!("  Output:       {}", output.display());
    println!();

    std::fs::create_dir_all(&output)?;

    println!("Running experiment...");
    let result = ExperimentRunner::run(&config)?;
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let base_path = output.join(result.experiment_id());
    write_experiment_outputs(&base_path, &result)?;
    print_validation_results(&result);

    println!();
    println!("✓ Experiment complete.");

    Ok(())
}

fn write_experiment_outputs(
    base_path: &std::path::Path,
    result: &sim_runner::ExperimentResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let csv_path = base_path.with_extension("csv");
    std::fs::write(&csv_path, result.to_csv()?)?;
    println!("  CSV:   {}", csv_path.display());

    let json_bytes = result.to_json()?;
    let json_path = base_path.with_extension("json");
    std::fs::write(&json_path, &json_bytes)?;
    println!("  JSON:  {}", json_path.display());

    if let Some(parent) = base_path.parent() {
        let manifest_path = parent.join("manifest.json");
        let _ = std::fs::write(&manifest_path, &json_bytes);
    }

    let md_path = base_path.with_extension("md");
    std::fs::write(&md_path, result.to_markdown()?)?;
    println!("  Report: {}", md_path.display());

    println!();
    Ok(())
}

fn print_validation_results(result: &sim_runner::ExperimentResult) {
    if result.acceptance_results().is_empty() {
        println!("⚠️  No acceptance criteria defined for this experiment");
        return;
    }

    let passed = result
        .acceptance_results()
        .iter()
        .filter(|r| r.passed)
        .count();
    let total = result.acceptance_results().len();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    println!("Validation Results: {passed}/{total} criteria passed");

    for c in result.acceptance_results() {
        let status = if c.passed { "✓" } else { "✗" };
        println!(
            "  {} {}: {} (expected [{:.2}, {:.2}], got {:.4})",
            status,
            c.criterion.metric_name,
            if c.passed { "PASS" } else { "FAIL" },
            c.criterion.expected_range.0,
            c.criterion.expected_range.1,
            c.observed_value
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
/// Run visualization command to generate PNG charts
fn run_visualize(
    manifest_path: PathBuf,
    output_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-runner: Visualization                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Determine output directory
    let out_dir = output_dir.map_or_else(
        || {
            manifest_path
                .parent()
                .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
        },
        |dir| dir,
    );

    println!("Manifest: {}", manifest_path.display());
    println!("Output:   {}", out_dir.display());
    println!();

    // Generate PNGs
    let generated_files = generate_experiment_pngs(&manifest_path, &out_dir)?;

    println!("Generated charts:");
    // TAG: surface=simulation owner=research-team rule=SIM-001
    for path in &generated_files {
        println!("  - {}", path.display());
    }
    println!();
    println!(
        "✓ Visualization complete. Generated {} PNG files.",
        generated_files.len()
    );

    Ok(())
}

/// Run validation on the 8 "ready" experiments
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
fn run_validate(output: PathBuf, quick: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Define the 8 ready experiments
    let experiments = [
        (
            ExperimentType::Pde4bLambdaCorrelation,
            "pde4b-lambda-correlation",
            "r > 0.50",
        ),
        (
            ExperimentType::Pde4bPharmacology,
            "pde4b-pharmacology",
            "d > 0.50",
        ),
        (
            ExperimentType::EmotionPharmacology,
            "emotion-pharmacology",
            "R² > 0.40",
        ),
        (
            ExperimentType::PhilosophicalOscillation,
            "philosophical-oscillation",
            "T ∈ [200,300]",
        ),
        (
            ExperimentType::CulturalQuadrants,
            "cultural-quadrants",
            "R² > 0.80",
        ),
        (
            ExperimentType::HofstedePrediction,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            "hofstede-prediction",
            "r > 0.50",
        ),
        (
            ExperimentType::PercolationThreshold,
            "percolation-threshold",
            "τ_c ≈ 0.32",
        ),
        (
            ExperimentType::CivilizationalCollapse,
            "civilizational-collapse",
            "accuracy > 0.70",
        ),
    ];

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Phase 2.1: Experiment Validation (8 Core)                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    std::fs::create_dir_all(&output)?;

    let mut results = Vec::new();
    let mut passed_count = 0;

    for (idx, (exp_type, name, target)) in experiments.iter().enumerate() {
        println!("[{}/8] Running: {}", idx + 1, name);
        println!("      Target: {target}");

        let mut config = ExperimentConfig::for_experiment(exp_type.clone());
        if quick {
            config.replications = config.replications.min(100);
            config.agent_count = config.agent_count.min(1000);
        }
        config.output_dir = output.join(name);

        match ExperimentRunner::run(&config) {
            Ok(result) => {
                let passed = result.passed;
                if passed {
                    passed_count += 1;
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                println!(
                    "      Result: {}",
                    if passed { "✅ PASSED" } else { "❌ FAILED" }
                );
                results.push(((*name).to_string(), passed));
            }
            Err(e) => {
                println!("      Result: ❌ ERROR - {e}");
                results.push(((*name).to_string(), false));
            }
        }
        println!();
    }

    // Generate validation report
    println!(
        "Validation Summary: {}/8 passed ({:.1}%)",
        passed_count,
        (f64::from(passed_count) / 8.0) * 100.0
    );

    // Write JSON report
    let report_path = output.join("validation_report.json");
    let report = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "mode": if quick { "quick" } else { "full" },
        "summary": {
            "total": 8,
            "passed": passed_count,
            "failed": 8 - passed_count,
            "pass_rate": f64::from(passed_count) / 8.0,
        },
        "experiments": results.iter().map(|(name, passed)| {
            serde_json::json!({
                "name": name,
                "passed": passed,
            })
        }).collect::<Vec<_>>(),
    });
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    println!("Report saved to: {}", report_path.display());

    Ok(())
}
