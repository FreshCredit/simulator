//! Simulation-aware prompt templates for Gemini AI
//!
//! Provides structured prompts for different types of simulation analysis.
//! All prompts maintain SIM-003 terminology ("simulation agent" not "AI agent").

use super::gemini::{NotableEvent, SimulationContext};
use sim_artifacts::manifest::Manifest;
use sim_artifacts::metrics::TickMetrics;
use sim_core::simulation::{GameEvent, SimulationReport};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Builder for simulation analysis prompts
#[derive(Debug)]
pub struct SimulationPromptBuilder;

impl SimulationPromptBuilder {
    /// System instruction for all simulation analysis
    #[must_use]
    pub fn system_instruction() -> String {
        "You are an expert scientific analyst for the Agent Society Simulator, grounded in the 33 published peer-reviewed SIM research papers (docas_revised.md v4.2, GTT, TFP, Whitepaper). \
         You analyze emergent multi-agent social, ecological, neurobiological, and economic phenomena. \
         Use precise scientific terminology: say 'simulation agent' not 'AI agent'. \
         When diagnosing simulation behavior, reference the empirical research invariants:\n\
         - Sim ECOL: Carrying capacity K_eff = K_0 + alpha_GTT*S_t - beta_defect*D_t (K_0=500). Allee threshold N_crit=25 below which populations undergo deterministic extinction. Allostatic yield threshold A_crit=50.0 beyond which chronic stress triggers structural default. Malthusian collapse occurs when metabolic cost > 2.5x resource density.\n\
         - Sim D21 & Sim D16: Meta TRIBE v2 fMRI forward encoding. Predatory visual cloaking (chi=3.65) and fast-talking auditory disclaimers (>9.8 syl/s) suppress Bilateral Anterior Insula (aINS) norm prediction error by 58.1%-60.0% (mean 0.5810 -> 0.2324), generating a 38.9x vulnerability gradient in stressed D1 agents. Ex-post default shocks trigger a 3.37x insular rebound burst.\n\
         - Sim O1-O3: Causal optogenetics. ArchT (566nm yellow) silences Dopamine (lambda^+ -> 0), suppressing trust by 99.97%. ChR2 (470nm blue) excites Cortisol (1.74x predatory uptake).\n\
         - Sim M: 5D TDF Music Vector. Metric hierarchy ratio tau disambiguates scalar tempo (6/8 compound vs 3/4 simple), yielding +136.9% relative mutual information gain.\n\
         - Sim K: Genealogy Kingman coalescent intensity kappa* = 1.62 (twin heritability h^2 = 0.342, accelerating TMRCA by 39.8%).\n\
         - Sim C & Sim P8b: 4:1 loss framing asymmetry (lambda^- = 0.08, lambda^+ = 0.02) and cognitive friction damping.\n\
         - Sim S: Semantic drift rate inversely correlates with trust (r = 0.994), while betrayal triggers an entropy surge (d = 2.79).\n\
         - Sim DMN: Counterfactual planning depth K=5 suppresses deceptive trap losses by 76.9%.\n\
         - Sim CRIT: Neural branching parameter sigma* = 1.0 maximizes dynamic range by +18.1 dB.\n\
         - Sim G7-G9 & Sim P9: Reconciled Master Law Delta S = [lambda^+ max(eps,0) + lambda^- min(eps,0)] r - delta_S(S - S_base) with stereographic phase map theta = 2*atan(eps) achieving dynamic Kuramoto synchronization R = 0.907.\n\
         Be concise, mathematically precise, cite paper findings, and directly reference live frame metrics (tick, alive count, avg trust, default rate, Gini)."
            .to_string()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze a completed simulation run from its report
    #[must_use]
    pub fn analyze_report(report: &SimulationReport) -> String {
        format!(
            "Please analyze this completed Agent Society Simulation:\n\n\
            **Summary Statistics**\n\
            - Total game events processed: {}\n\
            - Total births: {}\n\
            - Total deaths: {}\n\
            - Group formations: {}\n\
            - Recent births: {}\n\
            - Recent deaths: {}\n\n\
            **Recent Game Events** (last 20):\n{}\n\n\
            **Analysis Questions:**\n\
            1. What emergent behaviors do you observe from the event patterns?\n\
            2. How did trust and cooperation evolve?\n\
            3. What economic patterns emerged?\n\
            4. Were there any critical transitions or phase changes?\n\
            5. What would you recommend for future runs?",
            report.summary.total_game_events,
            report.summary.total_births,
            report.summary.total_deaths,
            report.summary.total_group_formations,
            report.births.len(),
            report.deaths.len(),
            format_game_events(&report.recent_game_events)
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze metrics time series data
    #[must_use]
    pub fn analyze_metrics(metrics: &[TickMetrics]) -> String {
        let (Some(first), Some(last)) = (metrics.first(), metrics.last()) else {
            return "No metrics data available for analysis.".to_string();
        };
        let count = metrics.len();

        // Calculate trends
        let avg_cooperation: f64 =
            metrics.iter().map(|m| m.cooperation_rate).sum::<f64>() / count as f64;
        let avg_gini: f64 = metrics.iter().map(|m| m.gini).sum::<f64>() / count as f64;
        let avg_trade: f64 =
            metrics.iter().map(|m| m.trade_volume as f64).sum::<f64>() / count as f64;

        format!(
            "Please analyze this Agent Society Simulation metrics data:\n\n\
            **Metrics Summary**\n\
            - Time span: ticks {} to {} ({} ticks)\n\
            // TAG: surface=simulation owner=research-team rule=SIM-001
            - Initial agents: {}\n\
            - Final agents: {}\n\n\
            **Averages Over Time**\n\
            - Cooperation rate: {:.2}%\n\
            - Gini coefficient: {:.3} (inequality measure)\n\
            - Trade volume: {:.0} transactions/tick\n\
            - Average DOCAS magnitude: {:.2}\n\
            - Branching ratio: {:.2}\n\n\
            **Analysis Questions:**\n\
            1. How did cooperation and trust evolve over time?\n\
            2. What does the Gini trend tell us about wealth distribution?\n\
            3. Were there periods of high instability (high branching ratio)?\n\
            4. What DOCAS patterns emerge?",
            first.tick,
            last.tick,
            count,
            first.agent_count,
            last.agent_count,
            avg_cooperation * 100.0,
            avg_gini,
            avg_trade,
            metrics.iter().map(|m| m.avg_docas_magnitude).sum::<f64>() / count as f64,
            metrics.iter().map(|m| m.branching_ratio).sum::<f64>() / count as f64,
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze from run manifest (configuration and validation)
    #[must_use]
    pub fn analyze_manifest(manifest: &Manifest) -> String {
        format!(
            "Please analyze this Agent Society Simulation configuration and results:\n\n\
            **Run Metadata**\n\
            - Run ID: {}\n\
            - Timestamp: {}\n\
            - Git Commit: {}\n\
            - Status: {:?}\n\n\
            **Configuration**\n\
            - Schema version: {}\n\
            - Metrics file: {}\n\
            - Replay directory: {}\n\n\
            **Validation Results**\n\
            - Passed: {}\n\
            - Issues: {}\n\n\
            **Analysis Questions:**\n\
            1. Was the simulation configuration appropriate for the goals?\n\
            2. What do the validation results indicate?\n\
            3. Were there any anomalies or unexpected behaviors?",
            manifest.run_id,
            manifest.timestamp,
            manifest.git_commit,
            manifest.status,
            manifest.schema_version,
            manifest.metrics_file,
            manifest.replay_dir,
            manifest
                .validation
                .as_ref()
                .map_or("N/A", |v| if v.passed { "Yes" } else { "No" }),
            manifest
                .validation
                .as_ref()
                .and_then(|v| v.pom_checks.first().map(|c| format!(
                    "{}: {} (value={:.2})",
                    c.pattern,
                    if c.observed {
                        "observed"
                    } else {
                        "not observed"
                    },
                    c.value
                )))
                .unwrap_or_else(|| "No validation data".to_string())
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Answer a specific question about live simulation context
    #[must_use]
    pub fn answer_question(context: &SimulationContext, question: &str) -> String {
        format!(
            "**Current Simulation Context**\n\
            - Tick: {}\n\
            - Alive agents: {}\n\
            - Average energy: {:.2}\n\
            - Groups formed: {}\n\
            - DOCAS dimensions: [Trust: {:.2}, Risk: {:.2}, Innovation: {:.2}, Altruism: {:.2}, Aggression: {:.2}]\n\n\
            **Recent Notable Events** (up to 10):\n{}\n\n\
            **User Question:** {}\n\n\
            Please provide a concise, accurate answer based on the simulation context.",
            context.tick,
            context.alive_count,
            context.avg_energy,
            context.group_count,
            context.avg_docas[0],
            context.avg_docas[1],
            context.avg_docas[2],
            context.avg_docas[3],
            context.avg_docas[4],
            format_notable_events(&context.notable_events),
            question
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compare two simulation runs
    #[must_use]
    pub fn compare_runs(run1: &Manifest, run2: &Manifest) -> String {
        format!(
            "Please compare these two Agent Society Simulation runs:\n\n\
            **Run 1: {}**\n\
            - Timestamp: {}\n\
            - Git commit: {}\n\
            - Status: {:?}\n\n\
            **Run 2: {}**\n\
            - Timestamp: {}\n\
            - Git commit: {}\n\
            - Status: {:?}\n\n\
            **Analysis Questions:**\n\
            1. What differences in configuration might explain different outcomes?\n\
            2. Which run showed more stable emergent behavior?\n\
            3. What lessons can be drawn for future experiments?",
            run1.run_id,
            run1.timestamp,
            run1.git_commit,
            run1.status,
            run2.run_id,
            run2.timestamp,
            run2.git_commit,
            run2.status
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate insights about emergent behaviors
    #[must_use]
    pub fn emergent_behaviors(
        context: &SimulationContext,
        recent_events: &[NotableEvent],
    ) -> String {
        format!(
            "**Simulation State**\n\
            - Current tick: {}\n\
            - Active agents: {}\n\
            - Groups: {}\n\
            - Energy distribution: {:.2} average\n\n\
            **Recent Activity** (last 20 events):\n{}\n\n\
            **Task:** Identify any emergent behaviors, patterns, or collective phenomena \
            that have developed in this simulation. Focus on:\n\
            - Cooperation vs competition patterns\n\
            - Group dynamics and formation\n\
            - Resource distribution strategies\n\
            - Adaptation to environmental pressures",
            context.tick,
            context.alive_count,
            context.group_count,
            context.avg_energy,
            format_notable_events(recent_events)
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate trading/economic analysis
    #[must_use]
    pub fn economic_analysis(
        trade_volume: u64,
        avg_docas: [f32; 5],
        recent_trades: &[String],
    ) -> String {
        format!(
            "**Economic Metrics**\n\
            - Total trade volume: {} transactions\n\
            - DOCAS profile: Trust={:.2}, Risk={:.2}, Innovation={:.2}, Altruism={:.2}, Aggression={:.2}\n\n\
            **Recent Trading Activity**:\n{}\n\n\
            **Task:** Analyze the economic behavior in this simulation:\n\
            1. What trading patterns emerge from the DOCAS profile?\n\
            2. How does trust correlate with trade volume?\n\
            3. What market dynamics are observable?",
            trade_volume,
            avg_docas[0], avg_docas[1], avg_docas[2], avg_docas[3], avg_docas[4],
            if recent_trades.is_empty() {
                "No recent trades".to_string()
            } else {
                recent_trades.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
            }
        )
    }
}

// Helper functions for formatting

fn format_game_events(events: &[GameEvent]) -> String {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    if events.is_empty() {
        "No events recorded".to_string()
    } else {
        events
            .iter()
            .rev()
            .take(20)
            .map(|e| format!("- {e:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_notable_events(events: &[NotableEvent]) -> String {
    if events.is_empty() {
        "No notable events".to_string()
    } else {
        events
            .iter()
            .take(10)
            .map(|e| {
                format!(
                    "- [{}] Agent {}: {}",
                    e.event_type,
                    e.agent_id.as_deref().unwrap_or("N/A"),
                    e.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Utility to truncate long text
pub fn truncate(text: impl AsRef<str>, max_len: usize) -> String {
    let text = text.as_ref();
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}... (truncated)", &text[..max_len])
    }
}

/// Estimate token count (rough approximation: 1 token ≈ 4 chars)
pub fn estimate_tokens(text: impl AsRef<str>) -> usize {
    text.as_ref().len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello... (truncated)");
    }

    #[test]
    fn test_format_notable_events_empty() {
        let events: Vec<NotableEvent> = vec![];
        assert_eq!(format_notable_events(&events), "No notable events");
    }
}
