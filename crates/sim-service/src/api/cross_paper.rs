//! Cross-Paper Meta-Analysis API (P1-SIM-5)
//!
//! REST endpoints for aggregating results across all 12 research papers.

use axum::{extract::Query, http::StatusCode, Json};
use serde::Deserialize;

use crate::meta_analysis::{MetaAggregator, PaperComparisonRequest, ResearchPaper};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get aggregated analysis of all papers
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_cross_paper_analysis(
    Query(params): Query<AnalysisQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let aggregator = MetaAggregator::new();
    let analysis = aggregator.aggregate_all();

    // Filter by papers if specified
    let papers = if let Some(ref paper_codes) = params.papers {
        let filtered: std::collections::HashMap<_, _> = analysis
            .papers
            .into_iter()
            .filter(|(paper, _)| paper_codes.contains(&paper.short_code().to_string()))
            .collect();
        filtered
    } else {
        analysis.papers
    };

    Ok(Json(serde_json::json!({
        "papers": papers,
        "cross_cutting_themes": analysis.cross_cutting_themes,
        "overall_synthesis": analysis.overall_synthesis,
        "generated_at": analysis.generated_at,
    })))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get summary statistics
pub async fn get_summary_statistics() -> Json<serde_json::Value> {
    let aggregator = MetaAggregator::new();
    let analysis = aggregator.aggregate_all();

    let total_experiments: usize = analysis.papers.values().map(|p| p.experiments.len()).sum();
    let completed_experiments: usize = analysis
        .papers
        .values()
        .flat_map(|p| &p.experiments)
        .filter(|e| matches!(e.status, crate::meta_analysis::ExperimentStatus::Completed))
        .count();

    let all_papers = ResearchPaper::all();
    let papers_with_coverage: Vec<_> = all_papers
        .iter()
        .filter_map(|paper| {
            analysis.papers.get(paper).map(|results| {
                serde_json::json!({
                    "paper": paper.short_code(),
                    "name": paper.name(),
                    "coverage": results.coverage_percentage,
                    "experiments_count": results.experiments.len(),
                    "completed_count": results.experiments.iter()
                        .filter(|e| matches!(e.status, crate::meta_analysis::ExperimentStatus::Completed))
                        .count(),
                })
            })
        })
        .collect();

    Json(serde_json::json!({
        "total_papers": analysis.papers.len(),
        "total_experiments": total_experiments,
        "completed_experiments": completed_experiments,
        "completion_rate": if total_experiments > 0 {
            (completed_experiments as f64 / total_experiments as f64) * 100.0
        } else { 0.0 },
        "cross_cutting_themes": analysis.cross_cutting_themes.len(),
        "research_gaps": analysis.overall_synthesis.research_gaps.len(),
        "papers": papers_with_coverage,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get cross-cutting themes
pub async fn get_cross_cutting_themes() -> Json<serde_json::Value> {
    let aggregator = MetaAggregator::new();
    let analysis = aggregator.aggregate_all();

    Json(serde_json::json!({
        "themes": analysis.cross_cutting_themes,
        "theme_count": analysis.cross_cutting_themes.len(),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get research gaps
pub async fn get_research_gaps() -> Json<serde_json::Value> {
    let aggregator = MetaAggregator::new();
    let analysis = aggregator.aggregate_all();

    Json(serde_json::json!({
        "gaps": analysis.overall_synthesis.research_gaps,
        "gap_count": analysis.overall_synthesis.research_gaps.len(),
        "recommendations": analysis.overall_synthesis.recommendations,
    }))
}

/// Compare two papers
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn compare_papers(
    Json(request): Json<PaperComparisonRequest>,
) -> Json<serde_json::Value> {
    let aggregator = MetaAggregator::new();
    let comparison = aggregator.compare_papers(request.paper1, request.paper2);

    Json(serde_json::json!({
        "paper1": comparison.paper1.short_code(),
        "paper1_name": comparison.paper1.name(),
        "paper2": comparison.paper2.short_code(),
        "paper2_name": comparison.paper2.name(),
        "correlations": comparison.correlations,
        "common_themes": comparison.common_themes,
        "divergent_findings": comparison.divergent_findings,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get publication readiness for a paper
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_publication_readiness(
    Query(params): Query<ReadinessQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let paper = parse_paper_code(&params.paper).ok_or(StatusCode::BAD_REQUEST)?;

    let aggregator = MetaAggregator::new();
    let readiness = aggregator.assess_publication_readiness(paper);

    Ok(Json(serde_json::json!({
        "paper": readiness.paper.short_code(),
        "paper_name": readiness.paper.name(),
        "overall_score": readiness.overall_score,
        "readiness_percentage": readiness.overall_score,
        "criteria": readiness.criteria,
        "blockers": readiness.blockers,
        "recommendations": readiness.recommendations,
        "ready_for_publication": readiness.overall_score >= 80.0 && readiness.blockers.is_empty(),
    })))
}

/// Get all papers publication readiness
pub async fn get_all_readiness() -> Json<serde_json::Value> {
    let aggregator = MetaAggregator::new();
    let papers = ResearchPaper::all();

    let readiness_list: Vec<_> = papers
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .iter()
        .map(|&paper| {
            let readiness = aggregator.assess_publication_readiness(paper);
            serde_json::json!({
                "paper": paper.short_code(),
                "paper_name": paper.name(),
                "overall_score": readiness.overall_score,
                "ready": readiness.overall_score >= 80.0 && readiness.blockers.is_empty(),
                "blockers_count": readiness.blockers.len(),
            })
        })
        .collect();

    let ready_count = readiness_list
        .iter()
        .filter(|r| {
            r.get("ready")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
        .count();

    Json(serde_json::json!({
        "papers": readiness_list,
        "ready_count": ready_count,
        "total_count": papers.len(),
        "ready_percentage": (ready_count as f64 / papers.len() as f64) * 100.0,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// List all papers
pub async fn list_papers() -> Json<serde_json::Value> {
    let papers = ResearchPaper::all();
    let paper_list: Vec<_> = papers
        .iter()
        .map(|p| {
            serde_json::json!({
                "code": p.short_code(),
                "name": p.name(),
                "id": format!("{p:?}").to_lowercase(),
            })
        })
        .collect();

    Json(serde_json::json!({
        "papers": paper_list,
        "count": paper_list.len(),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get synthesis report
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_synthesis() -> Json<serde_json::Value> {
    let aggregator = MetaAggregator::new();
    let analysis = aggregator.aggregate_all();

    Json(serde_json::json!({
        "synthesis": analysis.overall_synthesis,
        "generated_at": analysis.generated_at,
    }))
}

/// Get paper details
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_paper_details(
    Query(params): Query<PaperQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let paper = parse_paper_code(&params.paper).ok_or(StatusCode::BAD_REQUEST)?;

    let aggregator = MetaAggregator::new();
    let results = aggregator.aggregate_paper(paper);

    Ok(Json(serde_json::json!({
            "paper": paper.short_code(),
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            "name": paper.name(),
            "experiments": results.experiments,
            "key_findings": results.key_findings,
            "effect_sizes": results.effect_sizes,
            "validation_status": results.validation_status,
            "coverage_percentage": results.coverage_percentage,
        })))
}

// Helper functions

fn parse_paper_code(code: &str) -> Option<ResearchPaper> {
    match code.to_uppercase().as_str() {
        "GTT" => Some(ResearchPaper::Gtt),
        "DOCAS" => Some(ResearchPaper::Docas),
        "TFP" => Some(ResearchPaper::Tfp),
        "EMOT" => Some(ResearchPaper::Emotion),
        "CULT" => Some(ResearchPaper::Culture),
        "ECON" => Some(ResearchPaper::Economics),
        "LING" => Some(ResearchPaper::Linguistics),
        "PHIL" => Some(ResearchPaper::Philosophy),
        "GAME" => Some(ResearchPaper::GameTheory),
        "PDE4B" => Some(ResearchPaper::Pde4b),
        "MEAN" => Some(ResearchPaper::Meaning),
        "MUSIC" => Some(ResearchPaper::Music),
        _ => None,
    }
}

// Query parameters

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `AnalysisQuery` data
pub struct AnalysisQuery {
    /// papers
    pub papers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
/// `ReadinessQuery` data
pub struct ReadinessQuery {
    /// paper
    pub paper: String,
}

#[derive(Debug, Deserialize)]
/// `PaperQuery` data
pub struct PaperQuery {
    /// paper
    pub paper: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paper_code() {
        assert_eq!(parse_paper_code("gtt"), Some(ResearchPaper::Gtt));
        assert_eq!(parse_paper_code("docas"), Some(ResearchPaper::Docas));
        assert_eq!(parse_paper_code("tfp"), Some(ResearchPaper::Tfp));
        assert_eq!(parse_paper_code("unknown"), None);
    }
}
