//! Country-level Hofstede Cultural Profiles
//!
//! CVSCALE-based individual cultural orientation foundation.
//! Per Taras et al. (2016): ~80% of cultural value variance is WITHIN countries.
//!
//! This module provides country-level Hofstede dimensions from empirical research,
//! which serve as population MEANS for individual-level sampling.
//!
//! References:
//! - Hofstede, G. et al. (2010). Cultures and Organizations. McGraw-Hill.
//! - Yoo, B., Donthu, N., & Lenartowicz, T. (2011). CVSCALE. JBR.
//! - Taras, V. et al. (2016). Meta-analysis of cultural variance. JIBS.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Country-level Hofstede dimensions (from empirical data)
///
/// These are POPULATION-LEVEL means, not individual scores.
/// Individual scores are sampled from `N(μ_country`, `σ_within`) per CVSCALE methodology.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HofstedeCountryProfile {
    /// ISO 3166-1 alpha-3 country code
    /// Using String instead of &'static str to avoid lifetime issues with serde
    pub country_code: String,

    /// Power Distance Index [0, 100]
    /// Extent to which less powerful members expect unequal power distribution
    pub power_distance: f32,

    /// Individualism [0, 100]
    /// Degree of interdependence a society maintains among its members
    pub individualism: f32,

    /// Masculinity [0, 100]
    /// Preference for achievement, heroism, assertiveness vs cooperation
    pub masculinity: f32,

    /// Uncertainty Avoidance [0, 100]
    /// How comfortable a society is with ambiguity
    pub uncertainty_avoidance: f32,

    /// Long-term Orientation [0, 100]
    /// Link with the past while dealing with the present/future
    pub long_term_orientation: f32,

    /// Indulgence [0, 100]
    /// Extent to which people try to control their desires and impulses
    pub indulgence: f32,

    /// Cultural tightness (within-country variance)
    /// Tight cultures: low variance (homogeneous), Loose cultures: high variance (heterogeneous)
    /// Range: [0, 1] where 0 = very loose, 1 = very tight
    /// Per Gelfand et al. (2011): Tightness-looseness theory
    pub tightness: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl HofstedeCountryProfile {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get within-country standard deviation based on tightness
    ///
    /// Tight cultures have lower within-country variance.
    /// Loose cultures have higher within-country variance.
    ///
    /// # Arguments
    /// * `total_variance` - Total variance across all individuals (default: 100.0)
    /// * `icc` - Intraclass correlation coefficient (default: 0.20, meaning 20% between-country)
    ///
    /// # Returns
    /// Standard deviation for individual sampling
    #[must_use]
    pub fn within_country_sd(&self, total_variance: f32, icc: f32) -> f32 {
        // Within-country variance = total_variance * (1 - ICC) * looseness_factor
        // Tightness → lower variance, Looseness → higher variance
        let within_var = total_variance * (1.0 - icc);
        let looseness_factor = 1.0 - self.tightness * 0.5; // Tightness reduces variance
        (within_var * looseness_factor).sqrt()
    }

    /// Create a simulated/simulation profile from computed means
    #[must_use]
    pub fn simulated(code: &str, means: [f32; 6]) -> Self {
        Self {
            country_code: code.to_string(),
            power_distance: means[0],
            individualism: means[1],
            masculinity: means[2],
            uncertainty_avoidance: means[3],
            long_term_orientation: means[4],
            indulgence: means[5],
            tightness: 0.5, // Default medium tightness
        }
    }
}

// ============================================================================
// EMPIRICAL HOFSTEDE PROFILES
// Based on Hofstede's research (1980-2010) and subsequent updates
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// United States profile
/// High individualism (91), low power distance (40), medium masculinity (62)
#[must_use]
pub fn hofstede_usa() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "USA".to_string(),
        power_distance: 40.0,
        individualism: 91.0,
        masculinity: 62.0,
        uncertainty_avoidance: 46.0,
        long_term_orientation: 26.0,
        indulgence: 68.0,
        tightness: 0.60, // Medium (loose individualism but tight institutions)
    }
}

/// China profile
/// Low individualism (20), high power distance (80), very high LTO (87)
#[must_use]
pub fn hofstede_china() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "CHN".to_string(),
        power_distance: 80.0,
        individualism: 20.0,
        masculinity: 66.0,
        uncertainty_avoidance: 30.0,
        long_term_orientation: 87.0,
        indulgence: 24.0,
        tightness: 0.75, // High (cultural norms strongly enforced)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Japan profile
/// High masculinity (95), very high uncertainty avoidance (92), high LTO (88)
#[must_use]
pub fn hofstede_japan() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "JPN".to_string(),
        power_distance: 54.0,
        individualism: 46.0,
        masculinity: 95.0,
        uncertainty_avoidance: 92.0,
        long_term_orientation: 88.0,
        indulgence: 42.0,
        tightness: 0.85, // Very high (strong social norms)
    }
}

/// Germany profile
/// Medium individualism (67), high uncertainty avoidance (65), low LTO (31)
#[must_use]
pub fn hofstede_germany() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "DEU".to_string(),
        power_distance: 35.0,
        individualism: 67.0,
        masculinity: 66.0,
        uncertainty_avoidance: 65.0,
        long_term_orientation: 31.0,
        indulgence: 40.0,
        tightness: 0.70, // High (rule-based society)
    }
}

/// Norway profile
/// Low masculinity (8), high individualism (69), low power distance (31)
#[must_use]
pub fn hofstede_norway() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "NOR".to_string(),
        power_distance: 31.0,
        individualism: 69.0,
        masculinity: 8.0,
        uncertainty_avoidance: 50.0,
        long_term_orientation: 35.0,
        indulgence: 55.0,
        tightness: 0.45, // Lower (consensual, egalitarian)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Brazil profile
/// High power distance (69), high uncertainty avoidance (76), high indulgence (59)
#[must_use]
pub fn hofstede_brazil() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "BRA".to_string(),
        power_distance: 69.0,
        individualism: 38.0,
        masculinity: 49.0,
        uncertainty_avoidance: 76.0,
        long_term_orientation: 44.0,
        indulgence: 59.0,
        tightness: 0.55, // Medium
    }
}

/// Nigeria profile
/// High power distance (80), low LTO (13), very high indulgence (84)
#[must_use]
pub fn hofstede_nigeria() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "NGA".to_string(),
        power_distance: 80.0,
        individualism: 30.0,
        masculinity: 60.0,
        uncertainty_avoidance: 55.0,
        long_term_orientation: 13.0,
        indulgence: 84.0,
        tightness: 0.50, // Medium (diverse ethnic groups)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// India profile
/// High power distance (77), medium individualism (48), high masculinity (56)
#[must_use]
pub fn hofstede_india() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "IND".to_string(),
        power_distance: 77.0,
        individualism: 48.0,
        masculinity: 56.0,
        uncertainty_avoidance: 40.0,
        long_term_orientation: 51.0,
        indulgence: 26.0,
        tightness: 0.65, // Medium-high (hierarchical society)
    }
}

/// Russia profile
/// High power distance (93), medium individualism (39), high LTO (81)
#[must_use]
pub fn hofstede_russia() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "RUS".to_string(),
        power_distance: 93.0,
        individualism: 39.0,
        masculinity: 36.0,
        uncertainty_avoidance: 95.0,
        long_term_orientation: 81.0,
        indulgence: 20.0,
        tightness: 0.75, // High (centralized history)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mexico profile
/// High power distance (81), low individualism (30), high LTO (82)
#[must_use]
pub fn hofstede_mexico() -> HofstedeCountryProfile {
    HofstedeCountryProfile {
        country_code: "MEX".to_string(),
        power_distance: 81.0,
        individualism: 30.0,
        masculinity: 69.0,
        uncertainty_avoidance: 82.0,
        long_term_orientation: 82.0,
        indulgence: 97.0, // Highest indulgence score
        tightness: 0.60,  // Medium-high
    }
}

/// Get profile by country code
/// Returns a new instance since profiles are now generated by functions
#[must_use]
pub fn get_profile_by_code(code: &str) -> Option<HofstedeCountryProfile> {
    match code.to_uppercase().as_str() {
        "USA" | "US" => Some(hofstede_usa()),
        "CHN" | "CN" => Some(hofstede_china()),
        "JPN" | "JP" => Some(hofstede_japan()),
        "DEU" | "DE" => Some(hofstede_germany()),
        "NOR" | "NO" => Some(hofstede_norway()),
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        "BRA" | "BR" => Some(hofstede_brazil()),
        "NGA" | "NG" => Some(hofstede_nigeria()),
        "IND" | "IN" => Some(hofstede_india()),
        "RUS" | "RU" => Some(hofstede_russia()),
        "MEX" | "MX" => Some(hofstede_mexico()),
        _ => None,
    }
}

/// Get all available profiles
/// Returns a vector of profile instances
#[must_use]
pub fn all_profiles() -> Vec<HofstedeCountryProfile> {
    vec![
        hofstede_usa(),
        hofstede_china(),
        hofstede_japan(),
        hofstede_germany(),
        hofstede_norway(),
        hofstede_brazil(),
        hofstede_nigeria(),
        hofstede_india(),
        hofstede_russia(),
        hofstede_mexico(),
    ]
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute population-level aggregate from individual scores
///
/// This is the VALID way to get "national culture" - aggregate up from individuals,
/// not assign down from nations to individuals (which would be ecological fallacy).
#[must_use]
pub fn compute_population_profile(
    country_code: &'static str,
    individual_scores: &[(f32, f32, f32, f32, f32, f32)], // (PDI, IDV, MAS, UAI, LTO, IVR)
) -> HofstedeCountryProfile {
    let n = individual_scores.len() as f32;

    if individual_scores.is_empty() {
        return HofstedeCountryProfile {
            country_code: country_code.to_string(),
            power_distance: 50.0,
            individualism: 50.0,
            masculinity: 50.0,
            uncertainty_avoidance: 50.0,
            long_term_orientation: 50.0,
            indulgence: 50.0,
            tightness: 0.5,
        };
    }

    let sum_pdi: f32 = individual_scores.iter().map(|s| s.0).sum();
    let sum_idv: f32 = individual_scores.iter().map(|s| s.1).sum();
    let sum_mas: f32 = individual_scores.iter().map(|s| s.2).sum();
    let sum_uai: f32 = individual_scores.iter().map(|s| s.3).sum();
    let sum_lto: f32 = individual_scores.iter().map(|s| s.4).sum();
    let sum_ivr: f32 = individual_scores.iter().map(|s| s.5).sum();

    // Compute tightness as inverse of normalized variance
    let variance: f32 = individual_scores
        .iter()
        .map(|s| {
            let mean_score = (s.0 + s.1 + s.2 + s.3 + s.4 + s.5) / 6.0;
            let mean_pop = (sum_pdi + sum_idv + sum_mas + sum_uai + sum_lto + sum_ivr) / (6.0 * n);
            (mean_score - mean_pop).powi(2)
        })
        .sum::<f32>()
        / n;

    // Tightness: low variance = high tightness
    let tightness = (1.0 - (variance.sqrt() / 50.0)).clamp(0.0, 1.0);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let safe_n = n.max(1.0);
    HofstedeCountryProfile {
        country_code: country_code.to_string(),
        power_distance: sum_pdi / safe_n,
        individualism: sum_idv / safe_n,
        masculinity: sum_mas / safe_n,
        uncertainty_avoidance: sum_uai / safe_n,
        long_term_orientation: sum_lto / safe_n,
        indulgence: sum_ivr / safe_n,
        tightness,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_lookup() {
        let usa = get_profile_by_code("USA");
        assert!(usa.is_some());
        assert_eq!(usa.unwrap().country_code, "USA");

        let china = get_profile_by_code("CHN");
        assert!(china.is_some());
        assert_eq!(china.unwrap().country_code, "CHN");
    }

    // Re-export test helpers
    fn hofstede_usa() -> HofstedeCountryProfile {
        super::hofstede_usa()
    }
    fn hofstede_china() -> HofstedeCountryProfile {
        super::hofstede_china()
    }
    fn hofstede_japan() -> HofstedeCountryProfile {
        super::hofstede_japan()
    }
    fn hofstede_norway() -> HofstedeCountryProfile {
        super::hofstede_norway()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_known_cultural_patterns() {
        // USA: High individualism, low power distance
        let usa = hofstede_usa();
        assert!(usa.individualism > 80.0);
        assert!(usa.power_distance < 50.0);

        // China: Low individualism, high power distance
        let china = hofstede_china();
        assert!(china.individualism < 50.0);
        assert!(china.power_distance > 70.0);

        // Japan: Very high masculinity and uncertainty avoidance
        let japan = hofstede_japan();
        assert!(japan.masculinity > 90.0);
        assert!(japan.uncertainty_avoidance > 90.0);

        // Norway: Very low masculinity (feminine culture)
        let norway = hofstede_norway();
        assert!(norway.masculinity < 20.0);
    }

    #[test]
    fn test_tightness_variance_relationship() {
        // Japan is tight (high norms enforcement) → lower within-country SD
        let japan = hofstede_japan();
        let japan_sd = japan.within_country_sd(100.0, 0.20);

        // Norway is looser → higher within-country SD
        let norway = hofstede_norway();
        let norway_sd = norway.within_country_sd(100.0, 0.20);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Norway should have higher within-country SD than Japan
        assert!(
            norway_sd > japan_sd,
            "Loose cultures should have higher within-country variance"
        );
    }

    #[test]
    fn test_population_profile_computation() {
        // Create some individual scores around USA means
        let individuals: Vec<(f32, f32, f32, f32, f32, f32)> = vec![
            (35.0, 86.0, 57.0, 41.0, 21.0, 63.0),
            (40.0, 91.0, 62.0, 46.0, 26.0, 68.0),
            (45.0, 96.0, 67.0, 51.0, 31.0, 73.0),
        ];

        let pop_profile = compute_population_profile("TST", &individuals);

        // Population means should be close to individual means
        assert!((pop_profile.power_distance - 40.0).abs() < 5.0);
        assert!((pop_profile.individualism - 91.0).abs() < 5.0);
    }

    #[test]
    fn test_icc_variance_computation() {
        let profile = hofstede_usa();

        // ICC = 0.20 (20% between-country variance)
        let sd_20 = profile.within_country_sd(100.0, 0.20);

        // ICC = 0.50 (50% between-country variance)
        let sd_50 = profile.within_country_sd(100.0, 0.50);

        // Higher ICC (more between-country variance) = lower within-country SD
        assert!(
            sd_50 < sd_20,
            "Higher ICC should yield lower within-country SD"
        );
    }
}
