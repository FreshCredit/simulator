//! Router configuration for sim-service
//! ## AUTHENTICATION
//! All simulation research routes require auth middleware or internal token validation.

//!
//! Sets up all HTTP routes and middleware.
//!
//! # Rate Limiting
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! Rate limiting can be applied using the `rate_limit` module:
//!
//! ```rust,ignore
//! use tower_governor::GovernorLayer;
//! use sim_service::rate_limit::{Tier, EndpointLimiters};
//!
//! // Apply tier-based rate limiting to routes
//! let limited_routes = Router::new()
//!     .route("/api/experiments", get(list_experiments))
//!     .layer(GovernorLayer::new(Tier::Consumer.config()));
//!
//! // Apply connection limiting to SSE streams
//! let stream_routes = Router::new()
//!     .route("/api/live", get(live_stream))
//!     .with_state(connection_limiter.clone());
//! ```

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::api::analysis::{get_metrics_insights, get_run_insights, query_artifact, AnalysisState};
use crate::api::batch::{
    cancel_batch, create_batch_store, get_batch_status, list_active_batches, stream_batch_progress,
    submit_batch, BatchState,
};
use crate::api::cross_paper::{
    compare_papers, get_all_readiness, get_cross_cutting_themes, get_cross_paper_analysis,
    get_paper_details, get_publication_readiness, get_research_gaps, get_summary_statistics,
    get_synthesis, list_papers,
};
use crate::api::cross_validation::{
    compare_models,
    fold_statistics,
    learning_curve,
    list_cv_methods,
    list_metrics,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    run_cross_validation,
};
use crate::api::events::{
    cancel_event, cancel_event_post, get_event_stats, get_event_status, get_pending_events,
    inject_immediate, inject_scenario, stream_events, submit_event, EventState,
};
use crate::api::export::{export_batch, export_preview, export_results, list_export_formats};
use crate::api::health::{
    acknowledge_alert, get_alerts, get_dashboard, get_health, get_service_health,
    get_simulation_health, get_simulation_metrics, get_simulations_health, get_thresholds,
    get_time_series, remove_simulation, stream_dashboard, submit_simulation_metrics,
    update_thresholds, HealthState,
};
use crate::api::import::{
    delete_dataset, get_dataset, import_dataset, list_imported_datasets, map_to_experiment,
    upload_dataset, validate_dataset,
};
use crate::api::memory::{
    add_related_memory, clear_memories, consolidate_memories, delete_memory, get_memory_stats,
    query_memories, retrieve_associative, retrieve_by_context, retrieve_similar, store_memory,
    MemoryState,
};
use crate::api::meta::{
    compare_simulation_empirical, cumulative_meta_analysis, generate_forest_plot,
    generate_funnel_plot, leave_one_out_analysis, list_heterogeneity_methods,
    list_models as list_meta_models, run_meta_analysis, subgroup_analysis,
};
use crate::api::neural::{get_agent_brain_state, get_population_neural_stats};
use crate::api::patterns::{
    cancel_pattern, execute_from_template, execute_pattern, export_pattern_results,
    get_pattern_results, get_pattern_status, get_template, list_patterns, list_templates,
    preview_pattern, validate_pattern, PatternState,
};
use crate::api::pde4b::{get_lambda_stats, get_pde4b_config, update_pde4b_config, Pde4bState};
use crate::api::replay::{
    add_bookmark, compare_runs, create_replay, delete_session, get_bookmarks, get_diff, get_frames,
    get_session, get_storage_stats, list_run_sessions, pause_replay, play_replay, remove_bookmark,
    seek_replay, stream_replay, ReplayState,
};
// TAG: surface=simulation owner=research-team rule=SIM-001
use crate::api::reproducibility::{
    analyze_seed_sensitivity, analyze_variance, check_equivalence, check_reproducibility,
    generate_report, recommended_replications, reproducibility_guidelines,
};
use crate::api::research::{
    compare_models as compare_bayesian_models, discover_causal, generate_hypotheses,
    generate_synthesis, get_research_report, high_priority_hypotheses, list_hypotheses,
    search_literature, ResearchState,
};
use crate::api::results::{
    create_version, delete_result, export_result, get_result, get_result_versions, get_stats,
    list_results, store_result, ResultsState,
};
use crate::api::sensitivity::{
    compare_methods, generate_spider_plot, generate_tornado, get_parameter_templates, list_methods,
    run_local_analysis, run_morris_analysis, run_sobol_analysis, SensitivityState,
};
use crate::api::validation::{
    calculate_power, calculate_sample_size, convert_effect_size, generate_power_curve,
    get_effect_size_conventions, get_power_analysis_summary, validate_sample_size, ValidationState,
};
use crate::api::visualization::{
    generate_chart, generate_comparison_chart, generate_dashboard, generate_heatmap,
    generate_histogram, generate_radar_chart, generate_scatter_plot, list_chart_types,
    list_color_schemes,
};
use crate::artifacts::{self, ArtifactStore};
use crate::events::injector::EventInjector;
use crate::executor::ExecutorConfig;
use crate::health::collector::HealthCollector;
use crate::jobs::{self, JobState, JobStore};
use crate::live::{self, LiveSimulationState};
use crate::memory::consolidation::MemoryMaintenance;
use crate::memory::store::MemoryStore;
use crate::persistence::result_store::ResultStore;
use crate::replay::player::PlayerManager;
use crate::replay::storage::ReplayStorage;
use crate::streaming::websocket::{batch_websocket, experiment_websocket};
use crate::sweep::{self, SweepState};

// AI Chat imports
use crate::ai::AiState;
use crate::live::{ai_chat, ai_stream};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create the main router with all routes
pub fn create_router(artifacts_dir: PathBuf) -> Router {
    info!(
        "[Router] Initializing with artifacts_dir: {:?}",
        artifacts_dir
    );
    create_router_with_config(
        artifacts_dir.clone(),
        ExecutorConfig {
            sim_runner_path: PathBuf::from("sim-runner"),
            artifacts_dir,
        },
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create the main router with custom executor config
/// # Panics
///
/// Panics if preconditions are not met.
#[allow(clippy::needless_pass_by_value)] // public entry point; ownership kept for API stability
pub fn create_router_with_config(
    artifacts_dir: PathBuf,
    executor_config: ExecutorConfig,
) -> Router {
    let cors = build_cors_layer();

    // Create shared state
    let artifact_store = Arc::new(ArtifactStore::new(artifacts_dir));
    let job_store = Arc::new(JobStore::new());

    // Create AI state from environment
    let ai_state = Arc::new(AiState::from_env());
    if ai_state.is_available() {
        info!("[Router] Gemini AI integration enabled");
    } else {
        info!("[Router] Gemini AI not configured (set GEMINI_API_KEY to enable)");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Create live simulation state with in-memory broadcast
    let live_state = LiveSimulationState::with_ai_state(ai_state.clone());
    info!("[Router] Using in-memory broadcast for live simulation");

    // Start the continuous simulation loop
    live::spawn_simulation_loop(live_state.clone());
    let live_routes = build_live_routes(&live_state);
    let artifact_routes = build_artifact_routes(&artifact_store);
    let job_routes = build_job_routes(&job_store, &executor_config);
    let preset_routes = build_preset_routes();
    let sweep_routes = build_sweep_routes(&job_store, &executor_config);
    let experiment_routes = build_experiment_routes(&executor_config);
    let pde4b_routes = build_pde4b_routes();
    let neural_routes = build_neural_routes(&live_state);
    let ai_routes = build_ai_routes(&live_state);
    let analysis_routes = build_analysis_routes(&artifact_store, &ai_state);
    let results_routes = build_results_routes();
    let batch_routes = build_batch_routes(&executor_config);
    let health_routes = build_health_routes();
    let memory_routes = build_memory_routes();
    let event_routes = build_event_routes();
    let replay_routes = build_replay_routes();
    let pattern_routes = build_pattern_routes();
    let validation_routes = build_validation_routes();
    let sensitivity_routes = build_sensitivity_routes();
    let cv_routes = build_cv_routes();
    let meta_routes = build_meta_routes();
    let reproducibility_routes = build_reproducibility_routes();
    let visualization_routes = build_visualization_routes();
    let cross_paper_routes = build_cross_paper_routes();
    let export_routes = build_export_routes();
    let import_routes = build_import_routes();
    let research_routes = build_research_routes();
    let docas_routes = build_docas_routes();
    let ws_routes = build_ws_routes();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Combine all routes
    Router::new()
        .route("/health", get(health))
        .route("/docas/viz", get(crate::api::docas::docas_visualizer))
        .nest("/api", live_routes)
        .nest("/api", artifact_routes)
        .nest("/api", job_routes)
        .nest("/api", preset_routes)
        .nest("/api", sweep_routes)
        .nest("/api", experiment_routes)
        .nest("/api", pde4b_routes)
        .nest("/api", neural_routes)
        .nest("/api", ai_routes)
        .nest("/api", analysis_routes)
        .nest("/api", results_routes)
        .nest("/api", batch_routes)
        .nest("/api", health_routes)
        .nest("/api", memory_routes)
        .nest("/api", event_routes)
        .nest("/api", replay_routes)
        .nest("/api", pattern_routes)
        .nest("/api", validation_routes)
        .nest("/api", sensitivity_routes)
        .nest("/api", cv_routes)
        .nest("/api", meta_routes)
        .nest("/api", reproducibility_routes)
        .nest("/api", visualization_routes)
        .nest("/api", cross_paper_routes)
        .nest("/api", export_routes)
        .nest("/api", import_routes)
        .nest("/api", research_routes)
        .nest("/api", docas_routes)
        .merge(ws_routes)
        .layer(cors)
}

/// Builds the CORS layer for browser access (ENT-008 compliance).
fn build_cors_layer() -> CorsLayer {
    // Use ALLOWED_ORIGINS env var or default to safe localhost origins
    let allowed_origins: Vec<String> = std::env::var("ALLOWED_ORIGINS").map_or_else(
        |_| {
            vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
            ]
        },
        |s| s.split(',').map(|o| o.trim().to_string()).collect(),
    );

    // ENT-008: Never use Any for origins - always use explicit whitelist
    let mut origins: Vec<axum::http::HeaderValue> = allowed_origins
        .iter()
        .filter(|o| *o != "*") // Exclude wildcard even if present in env var
        .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok())
        .collect();

    // BUILD-005 MARKER: Empty origins fallback implemented
    // Ensure at least one origin is present (fallback for health checks)
    if origins.is_empty() {
        origins.push(axum::http::HeaderValue::from_static(
            "http://localhost:3000",
        ));
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ])
}

/// Builds the `live_routes` router.
fn build_live_routes(live_state: &LiveSimulationState) -> Router {
    // Live simulation routes (SSE stream + speed control + lifecycle management)
    Router::new()
        .route("/live", get(live::live_stream))
        .route("/live/speed", post(live::set_speed))
        .route("/live/speed", get(live::get_speed))
        .route("/live/pause", post(live::pause))
        .route("/live/resume", post(live::resume))
        .route("/live/stop", post(live::stop))
        .route("/live/restart", post(live::restart))
        .route("/live/intervention", post(live::live_intervention))
        .route("/live/status", get(live::get_status))
        .route("/live/state/export", get(live::export_state))
        .route("/live/state/import", post(live::import_state))
        .route("/live/report", get(live::get_simulation_report))
        .with_state(live_state.clone())
}

/// Builds the `artifact_routes` router.
fn build_artifact_routes(artifact_store: &Arc<ArtifactStore>) -> Router {
    // Artifact routes (read-only, public)
    Router::new()
        .route("/runs", get(artifacts::list_runs))
        .route("/runs/:run_id/manifest", get(artifacts::get_manifest))
        .route("/runs/:run_id/metrics", get(artifacts::get_metrics))
        .route(
            "/runs/:run_id/replay/:tick",
            get(artifacts::get_replay_frame),
        )
        .with_state(artifact_store.clone())
}

/// Builds the `job_routes` router.
fn build_job_routes(job_store: &Arc<JobStore>, executor_config: &ExecutorConfig) -> Router {
    let job_state = JobState {
        store: job_store.clone(),
        executor_config: executor_config.clone(),
    };
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Job routes (submission and status) - deprecated, kept for backward compatibility
    Router::new()
        .route("/jobs", get(jobs::list_jobs).post(jobs::submit_job))
        .route("/jobs/:job_id", get(jobs::get_job_status))
        .with_state(job_state)
}

/// Builds the `preset_routes` router.
fn build_preset_routes() -> Router {
    // Parameter presets route (no state needed)
    Router::new().route("/presets", get(jobs::list_presets))
}

/// Builds the `sweep_routes` router.
fn build_sweep_routes(job_store: &Arc<JobStore>, executor_config: &ExecutorConfig) -> Router {
    // Parameter sweep routes (P2-8: batch runs)
    let sweep_state = SweepState {
        job_store: job_store.clone(),
        executor_config: executor_config.clone(),
    };
    Router::new()
        .route("/sweep", post(sweep::submit_sweep))
        .route("/sweep/:sweep_id/status", get(sweep::get_sweep_status))
        .with_state(sweep_state)
}

/// Builds the `experiment_routes` router.
fn build_experiment_routes(executor_config: &ExecutorConfig) -> Router {
    // Experiment routes (Phase 1: Experiment Catalog API)
    let executor_config_arc = Arc::new(executor_config.clone());
    Router::new()
        .route(
            "/experiments",
            get(crate::api::experiments::list_experiments),
        )
        .route(
            "/experiments/:exp_id",
            get(crate::api::experiments::get_experiment),
        )
        .route(
            "/experiments/:exp_id/schema",
            get(crate::api::experiments::get_experiment_schema),
        )
        .route(
            "/experiments/:exp_id/runs",
            post(crate::api::experiments::launch_experiment),
        )
        .route(
            "/experiment-runs/:run_id",
            get(crate::api::experiments::get_run_status),
        )
        .route(
            "/experiment-runs/:run_id/stream",
            get(crate::api::experiments::stream_progress),
        )
        .with_state(executor_config_arc)
}

/// Builds the `pde4b_routes` router.
fn build_pde4b_routes() -> Router {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // PDE4B configuration routes
    let pde4b_state = Pde4bState::new();
    Router::new()
        .route(
            "/config/pde4b",
            get(get_pde4b_config).post(update_pde4b_config),
        )
        .route("/stats/lambda", get(get_lambda_stats))
        .with_state(pde4b_state)
}

/// Builds the `neural_routes` router.
fn build_neural_routes(live_state: &LiveSimulationState) -> Router {
    // Neural brain visualization routes
    Router::new()
        .route("/neural/agent/:agent_id", get(get_agent_brain_state))
        .route("/neural/population", get(get_population_neural_stats))
        .with_state(live_state.clone())
}

/// Builds the `ai_routes` router.
fn build_ai_routes(live_state: &LiveSimulationState) -> Router {
    // AI Chat routes (P3: AI Chat backend)
    Router::new()
        .route("/ai/stream", get(ai_stream))
        .route("/ai/chat", post(ai_chat))
        .with_state(live_state.clone())
}

/// Builds the `analysis_routes` router.
fn build_analysis_routes(artifact_store: &Arc<ArtifactStore>, ai_state: &Arc<AiState>) -> Router {
    // Artifact analysis routes (Gemini AI integration)
    let analysis_state = AnalysisState::new(artifact_store.clone(), ai_state.clone());
    Router::new()
        .route("/analysis/:run_id/insights", get(get_run_insights))
        .route("/analysis/:run_id/query", post(query_artifact))
        .route(
            "/analysis/:run_id/metrics-insights",
            get(get_metrics_insights),
        )
        .with_state(analysis_state)
}

/// Builds the `results_routes` router.
fn build_results_routes() -> Router {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Results API routes (P1-SIM-2: Result persistence)
    let results_state = ResultsState {
        store: Arc::new(ResultStore::new()),
    };
    Router::new()
        .route("/results", get(list_results).post(store_result))
        .route("/results/:result_id", get(get_result).delete(delete_result))
        .route(
            "/results/:result_id/versions",
            get(get_result_versions).post(create_version),
        )
        .route("/results/export/:result_id", get(export_result))
        .route("/results/stats", get(get_stats))
        .with_state(results_state)
}

/// Builds the `batch_routes` router.
fn build_batch_routes(executor_config: &ExecutorConfig) -> Router {
    // Batch execution routes (P1-SIM-1: Parallel execution)
    let batch_store = create_batch_store();
    let batch_state = BatchState {
        store: batch_store,
        config: Arc::new(executor_config.clone()),
    };
    Router::new()
        .route("/batch", get(list_active_batches).post(submit_batch))
        .route("/batch/:batch_id", get(get_batch_status))
        .route("/batch/:batch_id/progress", get(stream_batch_progress))
        .route("/batch/:batch_id/cancel", post(cancel_batch))
        .with_state(batch_state)
}

/// Builds the `health_routes` router.
fn build_health_routes() -> Router {
    // Health dashboard routes (P1-SIM-3: Health Dashboard)
    let health_collector = Arc::new(HealthCollector::new());
    health_collector.start_collection_loop();

    let health_state = HealthState {
        collector: health_collector,
        service_start: std::time::Instant::now(),
        // TAG: surface=simulation owner=research-team rule=SIM-001
    };
    Router::new()
        .route("/health", get(get_health))
        .route("/health/service", get(get_service_health))
        .route("/health/dashboard", get(get_dashboard))
        .route("/health/dashboard/stream", get(stream_dashboard))
        .route("/health/simulations", get(get_simulations_health))
        .route(
            "/health/simulations/:sim_id",
            get(get_simulation_health).delete(remove_simulation),
        )
        .route(
            "/health/simulations/:sim_id/metrics",
            get(get_simulation_metrics).post(submit_simulation_metrics),
        )
        .route(
            "/health/simulations/:sim_id/timeseries",
            get(get_time_series),
        )
        .route("/health/alerts", get(get_alerts))
        .route("/health/alerts/:alert_id/ack", post(acknowledge_alert))
        .route(
            "/health/thresholds",
            get(get_thresholds).put(update_thresholds),
        )
        .with_state(health_state)
}

/// Builds the `memory_routes` router.
fn build_memory_routes() -> Router {
    // Memory routes (P1-SIM-4: Agent Memory Subsystem)
    let memory_state = MemoryState {
        store: Arc::new(MemoryStore::new()),
        maintenance: Arc::new(tokio::sync::Mutex::new(MemoryMaintenance::new())),
    };
    Router::new()
        .route("/memory", post(store_memory))
        .route(
            "/memory/:agent_id",
            get(query_memories).delete(clear_memories),
        )
        .route("/memory/:agent_id/stats", get(get_memory_stats))
// TAG: surface=simulation owner=research-team rule=SIM-001
        .route("/memory/:agent_id/consolidate", post(consolidate_memories))
        .route(
            "/memory/:agent_id/retrieve/context",
            get(retrieve_by_context),
        )
        .route(
            "/memory/:agent_id/retrieve/similar/:memory_id",
            get(retrieve_similar),
        )
        .route(
            "/memory/:agent_id/retrieve/associative/:memory_id",
            get(retrieve_associative),
        )
        .route("/memory/:agent_id/:memory_id", delete(delete_memory))
        .route(
            "/memory/:agent_id/:memory_id/relate",
            post(add_related_memory),
        )
        .with_state(memory_state)
}

/// Builds the `event_routes` router.
fn build_event_routes() -> Router {
    // Event injection routes (P1-SIM-5: External Event Injection)
    let event_state = EventState {
        injector: Arc::new(EventInjector::new()),
    };
    Router::new()
        // Simulation-scoped routes (must come before /events/:event_id)
        .route("/events/simulation/:simulation_id", post(submit_event))
        .route("/events/simulation/:simulation_id/immediate", post(inject_immediate))
        .route("/events/simulation/:simulation_id/scenario", post(inject_scenario))
        .route("/events/simulation/:simulation_id/pending", get(get_pending_events))
        .route("/events/simulation/:simulation_id/stats", get(get_event_stats))
        .route("/events/simulation/:simulation_id/stream", get(stream_events))
        // Event-specific routes
        .route("/events/:event_id", get(get_event_status).delete(cancel_event))
        .route("/events/:event_id/cancel", post(cancel_event_post))
        .with_state(event_state)
}

/// Builds the `replay_routes` router.
fn build_replay_routes() -> Router {
    // Replay & Diff routes (P1-SIM-6: Simulation Replay & Diff)
    let replay_state = ReplayState {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        storage: Arc::new(ReplayStorage::new()),
        player_manager: Arc::new(PlayerManager::new()),
    };
    Router::new()
        .route("/replay", post(create_replay).get(get_storage_stats))
        .route(
            "/replay/:session_id",
            get(get_session).delete(delete_session),
        )
        .route("/replay/:session_id/frames", get(get_frames))
        .route("/replay/:session_id/play", post(play_replay))
        .route("/replay/:session_id/pause", post(pause_replay))
        .route("/replay/:session_id/seek", post(seek_replay))
        .route(
            "/replay/:session_id/bookmark",
            post(add_bookmark).get(get_bookmarks),
        )
        .route(
            "/replay/:session_id/bookmarks/:bookmark_id",
            delete(remove_bookmark),
        )
        .route("/replay/:session_id/stream", get(stream_replay))
        .route("/replay/runs/:run_id/sessions", get(list_run_sessions))
        .route("/diff", post(compare_runs))
        .route("/diff/:base_id/:compare_id", get(get_diff))
        .with_state(replay_state)
}

/// Builds the `pattern_routes` router.
fn build_pattern_routes() -> Router {
    // Macro Pattern routes (P1-SIM-7: Macro Experiment Patterns)
    let pattern_state = PatternState::new();
    Router::new()
        // Template routes
        .route("/patterns/templates", get(list_templates))
        .route("/patterns/templates/:template_id", get(get_template))
        .route("/patterns/templates/:template_id/execute", post(execute_from_template))
        // Pattern execution routes
        .route("/patterns", get(list_patterns).post(execute_pattern))
        .route("/patterns/validate", post(validate_pattern))
        .route("/patterns/preview", post(preview_pattern))
        .route("/patterns/:pattern_id", get(get_pattern_status).delete(cancel_pattern))
        .route("/patterns/:pattern_id/results", get(get_pattern_results))
        .route("/patterns/:pattern_id/export", post(export_pattern_results))
        .with_state(pattern_state)
}

/// Builds the `validation_routes` router.
fn build_validation_routes() -> Router {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Statistical Validation routes (P1-VAL-1: Power Analysis)
    let validation_state = ValidationState::new();
    Router::new()
        .route("/validation/power", post(calculate_power))
        .route("/validation/sample-size", post(calculate_sample_size))
        .route(
            "/validation/sample-size/validate",
            post(validate_sample_size),
        )
        .route("/validation/power-curve", post(generate_power_curve))
        .route(
            "/validation/effect-sizes/conventions",
            get(get_effect_size_conventions),
        )
        .route(
            "/validation/effect-sizes/convert",
            post(convert_effect_size),
        )
        .route(
            "/validation/analysis-summary",
            post(get_power_analysis_summary),
        )
        .with_state(validation_state)
}

/// Builds the `sensitivity_routes` router.
fn build_sensitivity_routes() -> Router {
    // Sensitivity Analysis routes (P1-VAL-2: Sensitivity Analysis)
    let sensitivity_state = SensitivityState::new();
    Router::new()
        .route("/sensitivity/methods", get(list_methods))
        .route("/sensitivity/templates", get(get_parameter_templates))
        .route("/sensitivity/local", post(run_local_analysis))
        .route("/sensitivity/morris", post(run_morris_analysis))
        .route("/sensitivity/sobol", post(run_sobol_analysis))
        .route("/sensitivity/tornado", post(generate_tornado))
        .route("/sensitivity/spider", post(generate_spider_plot))
        .route("/sensitivity/compare", post(compare_methods))
        .with_state(sensitivity_state)
}

/// Builds the `cv_routes` router.
fn build_cv_routes() -> Router {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Cross-Validation routes (P1-VAL-3: Cross-Validation Framework)
    Router::new()
        .route("/cv/methods", get(list_cv_methods))
        .route("/cv/metrics", get(list_metrics))
        .route("/cv/run", post(run_cross_validation))
        .route("/cv/learning-curve", post(learning_curve))
        .route("/cv/compare", post(compare_models))
        .route("/cv/fold-stats", post(fold_statistics))
}

/// Builds the `meta_routes` router.
fn build_meta_routes() -> Router {
    // Meta-Analysis routes (P1-VAL-4: Effect Size Meta-Analysis)
    Router::new()
        .route("/meta/models", get(list_meta_models))
        .route(
            "/meta/heterogeneity-methods",
            get(list_heterogeneity_methods),
        )
        .route("/meta/analyze", post(run_meta_analysis))
        .route(
            "/meta/compare-simulation",
            post(compare_simulation_empirical),
        )
        .route("/meta/forest-plot", post(generate_forest_plot))
        .route("/meta/funnel-plot", post(generate_funnel_plot))
        .route("/meta/subgroup", post(subgroup_analysis))
        .route("/meta/cumulative", post(cumulative_meta_analysis))
        .route("/meta/leave-one-out", post(leave_one_out_analysis))
}

/// Builds the `reproducibility_routes` router.
fn build_reproducibility_routes() -> Router {
    // Reproducibility Validation routes (P1-VAL-5: Reproducibility Validation)
    Router::new()
        .route("/reproducibility/check", post(check_reproducibility))
        .route(
            "/reproducibility/seed-sensitivity",
            post(analyze_seed_sensitivity),
        )
        .route("/reproducibility/variance", post(analyze_variance))
        .route(
            "/reproducibility/recommended-replications",
            post(recommended_replications),
        )
        .route(
            "/reproducibility/guidelines",
            get(reproducibility_guidelines),
        )
        .route("/reproducibility/report", post(generate_report))
        .route("/reproducibility/equivalence", post(check_equivalence))
}

/// Builds the `visualization_routes` router.
fn build_visualization_routes() -> Router {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Visualization routes (P1-SIM-4: Experiment Visualization)
    Router::new()
        .route("/visualization/chart", get(generate_chart))
        .route("/visualization/chart-types", get(list_chart_types))
        .route("/visualization/color-schemes", get(list_color_schemes))
        .route("/visualization/comparison", post(generate_comparison_chart))
        .route("/visualization/scatter", post(generate_scatter_plot))
        .route("/visualization/histogram", post(generate_histogram))
        .route("/visualization/heatmap", post(generate_heatmap))
        .route("/visualization/radar", post(generate_radar_chart))
        .route("/visualization/dashboard", post(generate_dashboard))
}

/// Builds the `cross_paper_routes` router.
fn build_cross_paper_routes() -> Router {
    // Cross-Paper Meta-Analysis routes (P1-SIM-5: Cross-Paper Meta-Analysis)
    Router::new()
        .route("/cross-paper/analysis", get(get_cross_paper_analysis))
        .route("/cross-paper/summary", get(get_summary_statistics))
        .route("/cross-paper/themes", get(get_cross_cutting_themes))
        .route("/cross-paper/gaps", get(get_research_gaps))
        .route("/cross-paper/compare", post(compare_papers))
        .route("/cross-paper/readiness", get(get_publication_readiness))
        .route("/cross-paper/readiness/all", get(get_all_readiness))
        .route("/cross-paper/papers", get(list_papers))
        .route("/cross-paper/papers/:paper_id", get(get_paper_details))
        .route("/cross-paper/synthesis", get(get_synthesis))
}

/// Builds the `export_routes` router.
fn build_export_routes() -> Router {
    // Export routes (P1-API-4: Result Export)
    Router::new()
        .route("/export/formats", get(list_export_formats))
        .route("/export/results", get(export_results))
        .route("/export/batch", post(export_batch))
        .route("/export/preview", get(export_preview))
}

/// Builds the `import_routes` router.
fn build_import_routes() -> Router {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Import routes (P1-API-5: External Data Import)
    Router::new()
        .route(
            "/import/datasets",
            get(list_imported_datasets).post(import_dataset),
        )
        .route(
            "/import/datasets/:id",
            get(get_dataset).delete(delete_dataset),
        )
        .route("/import/upload", post(upload_dataset))
        .route("/import/validate", post(validate_dataset))
        .route("/import/map", post(map_to_experiment))
}

/// Builds the `research_routes` router.
fn build_research_routes() -> Router {
    // Research routes (P2-RES: Advanced Research Tools)
    let research_state = ResearchState::new();
    Router::new()
        .route("/research/hypotheses", get(list_hypotheses))
        .route(
            "/research/hypotheses/high-priority",
            get(high_priority_hypotheses),
        )
        .route("/research/hypotheses/generate", post(generate_hypotheses))
        .route("/research/literature/search", get(search_literature))
        .route(
            "/research/literature/synthesis/:topic",
            get(generate_synthesis),
        )
        .route("/research/causal/discover", post(discover_causal))
        .route("/research/bayesian/compare", post(compare_bayesian_models))
        .route("/research/report", get(get_research_report))
        .with_state(research_state)
}

/// Builds the `docas_routes` router.
fn build_docas_routes() -> Router {
    let docas_state = Arc::new(crate::api::docas::DocasStreamState::new());
    Router::new()
        .route(
            "/docas/snapshot",
            get(crate::api::docas::get_docas_snapshot),
        )
        .route("/docas/stats", get(crate::api::docas::get_docas_stats))
        .route("/docas/stream", get(crate::api::docas::stream_docas_stats))
        .route(
            "/docas/history/:agent_id",
            get(crate::api::docas::get_agent_docas_history),
        )
        .route("/docas/docas.js", get(crate::api::docas::docas_js))
        .with_state(docas_state)
}

/// Builds the `ws_routes` router.
fn build_ws_routes() -> Router {
    // WebSocket routes (P1-API-2: Real-time streaming)
    Router::new()
        .route("/ws/experiments/:exp_id", get(experiment_websocket))
        .route("/ws/batch/:batch_id", get(batch_websocket))
}

/// Health check endpoint
async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "sim-service",
        "version": crate::VERSION
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tempfile::tempdir;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let dir = tempdir().unwrap();
        let app = create_router(dir.path().to_path_buf());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_runs_empty() {
        let dir = tempdir().unwrap();
        let app = create_router(dir.path().to_path_buf());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/runs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
