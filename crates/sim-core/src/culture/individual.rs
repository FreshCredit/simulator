//! Individual Cultural Orientation - CVSCALE Implementation
//!
//! Implements the CVSCALE methodology (Yoo, Donthu & Lenartowicz, 2011) for
//! measuring Hofstede's cultural dimensions at the INDIVIDUAL level.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Key Insight from Taras et al. (2016) Meta-Analysis:
//! - ~80% of cultural value variance is WITHIN countries
//! - ~20% is BETWEEN countries
//! - Country is a poor proxy for individual culture
//!
//! ## CVSCALE Approach:
//! 1. Initialize agents from `N(μ_country`, `σ_within`)
//! 2. `σ_within` >> `σ_between` (80% vs 20% per ICC)
//! 3. Population means approximate Hofstede scores
//!
//! ## Ecological Validity:
//! This replaces the ecological fallacy of assigning nation-level scores directly
//! to individuals. Instead, individuals SAMPLE from a distribution centered on
//! the country mean with substantial within-country variance.
//!
//! References:
//! - Yoo, B., Donthu, N., & Lenartowicz, T. (2011). Measuring Hofstede's five
//!   dimensions of cultural values at the individual level. JBR, 64(6), 584-596.
//! - Taras, V. et al. (2016). Multilevel research: The present and the future.
//!   JIBS, 47(6), 633-651.

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::culture::profiles::HofstedeCountryProfile;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual cultural orientation based on CVSCALE methodology
///
/// Samples from distribution with country mean + individual variation.
/// Per SIM-003: These are SIMULATION constructs, not psychological assessments.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndividualCulturalOrientation {
    /// Power Distance Index (individual level)
    /// Range: [0.0, 100.0]
    /// Extent to which individual accepts unequal power distribution
    pub power_distance: f32,

    /// Individualism (individual level)
    /// Range: [0.0, 100.0]
    /// Degree of independence vs. interdependence preference
    pub individualism: f32,

    /// Masculinity (individual level)
    /// Range: [0.0, 100.0]
    /// Preference for achievement/competition vs. cooperation/care
    pub masculinity: f32,

    /// Uncertainty Avoidance (individual level)
    /// Range: [0.0, 100.0]
    /// Tolerance for ambiguity and unstructured situations
    pub uncertainty_avoidance: f32,

    /// Long-term Orientation (individual level)
    /// Range: [0.0, 100.0]
    /// Planning horizon and connection to past/future
    pub long_term_orientation: f32,

    /// Indulgence (individual level)
    /// Range: [0.0, 100.0]
    /// Control of desires and impulses vs. gratification
    pub indulgence: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl IndividualCulturalOrientation {
    /// Create a new individual cultural orientation from raw values
    ///
    /// All values are clamped to [0.0, 100.0] range.
    #[must_use]
    pub fn new(
        power_distance: f32,
        individualism: f32,
        masculinity: f32,
        uncertainty_avoidance: f32,
        long_term_orientation: f32,
        indulgence: f32,
    ) -> Self {
        Self {
            power_distance: power_distance.clamp(0.0, 100.0),
            individualism: individualism.clamp(0.0, 100.0),
            masculinity: masculinity.clamp(0.0, 100.0),
            uncertainty_avoidance: uncertainty_avoidance.clamp(0.0, 100.0),
            long_term_orientation: long_term_orientation.clamp(0.0, 100.0),
            indulgence: indulgence.clamp(0.0, 100.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Initialize from country profile with individual variation (CVSCALE method)
    ///
    /// # Arguments
    /// * `country_profile` - Hofstede country-level scores (population mean)
    /// * `within_country_variance` - Typically 0.80 (80% of variance is within)
    /// * `rng` - Random number generator for deterministic sampling
    ///
    /// # Methodology
    /// Per Taras et al. (2016): ICC ≈ 0.20, meaning 20% between-country, 80% within-country
    /// We sample from `N(μ_country`, `σ_within`) where `σ_within` corresponds to 80% of total variance.
    pub fn from_country_profile(
        country_profile: &HofstedeCountryProfile,
        within_country_variance: f32,
        rng: &mut StdRng,
    ) -> Self {
        // Total variance = between-country + within-country
        // If ICC = 0.20, then 20% between, 80% within
        let total_var = 100.0; // Arbitrary scale for Hofstede dimensions
        let _between_var = total_var * (1.0 - within_country_variance);
        let within_var = total_var * within_country_variance;

        // Adjust within variance by cultural tightness
        // Tight cultures have lower within-country variance
        let tightness_adjustment = 1.0 - country_profile.tightness * 0.3;
        let adjusted_within_var = within_var * tightness_adjustment;
        let within_sd = adjusted_within_var.sqrt();

        Self {
            power_distance: Self::sample_individual(country_profile.power_distance, within_sd, rng),
            individualism: Self::sample_individual(country_profile.individualism, within_sd, rng),
            masculinity: Self::sample_individual(country_profile.masculinity, within_sd, rng),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            uncertainty_avoidance: Self::sample_individual(
                country_profile.uncertainty_avoidance,
                within_sd,
                rng,
            ),
            long_term_orientation: Self::sample_individual(
                country_profile.long_term_orientation,
                within_sd,
                rng,
            ),
            indulgence: Self::sample_individual(country_profile.indulgence, within_sd, rng),
        }
    }

    /// Sample an individual score from normal distribution
    ///
    /// Uses Box-Muller transform for normal distribution sampling.
    /// Results are clamped to [0.0, 100.0] range.
    fn sample_individual(mean: f32, sd: f32, rng: &mut StdRng) -> f32 {
        // Box-Muller transform for normal distribution
        let u1: f32 = rng.gen();
        let u2: f32 = rng.gen();

        // Avoid log(0)
        let u1 = u1.max(f32::EPSILON);

        let z: f32 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();

        (mean + z * sd).clamp(0.0, 100.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute the individual's cultural "distance" from a country profile
    ///
    /// Uses Euclidean distance across all six dimensions.
    #[must_use]
    pub fn distance_from_profile(&self, profile: &HofstedeCountryProfile) -> f32 {
        let d_pdi = self.power_distance - profile.power_distance;
        let d_idv = self.individualism - profile.individualism;
        let d_mas = self.masculinity - profile.masculinity;
        let d_uai = self.uncertainty_avoidance - profile.uncertainty_avoidance;
        let d_lto = self.long_term_orientation - profile.long_term_orientation;
        let d_ivr = self.indulgence - profile.indulgence;

        ((d_pdi * d_pdi
            + d_idv * d_idv
            + d_mas * d_mas
            + d_uai * d_uai
            + d_lto * d_lto
            + d_ivr * d_ivr)
            / 6.0)
            .sqrt()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the individual's closest country profile from a set of options
    /// Returns a copy of the closest profile (profiles are small structs)
    pub fn closest_profile(&self, profiles: &[HofstedeCountryProfile]) -> HofstedeCountryProfile {
        profiles
            .iter()
            .min_by(|a, b| {
                let dist_a = self.distance_from_profile(a);
                let dist_b = self.distance_from_profile(b);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .unwrap_or_else(crate::culture::profiles::hofstede_usa)
    }

    /// Compute all dimension values as an array [PDI, IDV, MAS, UAI, LTO, IVR]
    #[must_use]
    pub const fn to_array(&self) -> [f32; 6] {
        [
            self.power_distance,
            self.individualism,
            self.masculinity,
            self.uncertainty_avoidance,
            self.long_term_orientation,
            self.indulgence,
        ]
    }

    /// Create from array [PDI, IDV, MAS, UAI, LTO, IVR]
    #[must_use]
    pub fn from_array(arr: [f32; 6]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Default neutral orientation (50 on all dimensions)
    #[must_use]
    pub fn neutral() -> Self {
        Self::new(50.0, 50.0, 50.0, 50.0, 50.0, 50.0)
    }
}

impl Default for IndividualCulturalOrientation {
    fn default() -> Self {
        Self::neutral()
    }
}

/// Generator for creating populations with CVSCALE-based cultural orientations
#[derive(Debug)]
pub struct CulturalOrientationGenerator {
    /// Country profile to use as population mean
    pub country_profile: HofstedeCountryProfile,
    /// Within-country variance proportion (default: 0.80)
    pub within_country_variance: f32,
    /// Intraclass correlation coefficient (default: 0.20)
    pub icc: f32,
}

impl CulturalOrientationGenerator {
    /// Create a new generator for a specific country profile
    #[must_use]
    pub const fn new(country_profile: HofstedeCountryProfile) -> Self {
        Self {
            country_profile,
            within_country_variance: 0.80, // Per Taras et al. (2016)
            icc: 0.20,                     // Per Taras et al. (2016)
        }
    }

    /// Create generator with custom variance parameters
    #[must_use]
    pub const fn with_variance(
        country_profile: HofstedeCountryProfile,
        within_country_variance: f32,
        icc: f32,
    ) -> Self {
        Self {
            country_profile,
            within_country_variance,
            icc,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate a single individual orientation
    pub fn generate(&self, rng: &mut StdRng) -> IndividualCulturalOrientation {
        IndividualCulturalOrientation::from_country_profile(
            &self.country_profile,
            self.within_country_variance,
            rng,
        )
    }

    /// Generate a population of cultural orientations
    pub fn generate_population(
        &self,
        n: usize,
        rng: &mut StdRng,
    ) -> Vec<IndividualCulturalOrientation> {
        (0..n).map(|_| self.generate(rng)).collect()
    }

    /// Generate population and validate that means approximate country profile
    pub fn generate_population_validated(
        &self,
        n: usize,
        rng: &mut StdRng,
    ) -> (Vec<IndividualCulturalOrientation>, PopulationValidation) {
        let population = self.generate_population(n, rng);
        let validation = PopulationValidation::compute(&population, &self.country_profile);
        (population, validation)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation results for generated population
#[derive(Debug, Clone, Copy)]
pub struct PopulationValidation {
    /// Population mean PDI
    pub mean_power_distance: f32,
    /// Population mean IDV
    pub mean_individualism: f32,
    /// Population mean MAS
    pub mean_masculinity: f32,
    /// Population mean UAI
    pub mean_uncertainty_avoidance: f32,
    /// Population mean LTO
    pub mean_long_term_orientation: f32,
    /// Population mean IVR
    pub mean_indulgence: f32,
    /// Within-country variance (should be ~80% of total)
    pub within_country_variance: f32,
    /// ICC estimate (should be ~0.20)
    pub icc_estimate: f32,
    /// Maximum deviation from country mean (should be < 5.0 for n > 100)
    pub max_deviation: f32,
    /// Whether validation passed
    pub passed: bool,
}

impl PopulationValidation {
    /// Compute validation metrics from population
    #[must_use]
    pub fn compute(
        population: &[IndividualCulturalOrientation],
        country_profile: &HofstedeCountryProfile,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ) -> Self {
        let n = (population.len() as f32).max(1.0);

        if population.is_empty() {
            return Self {
                mean_power_distance: 50.0,
                mean_individualism: 50.0,
                mean_masculinity: 50.0,
                mean_uncertainty_avoidance: 50.0,
                mean_long_term_orientation: 50.0,
                mean_indulgence: 50.0,
                within_country_variance: 0.0,
                icc_estimate: 0.0,
                max_deviation: 100.0,
                passed: false,
            };
        }

        // Compute means
        let mean_pdi = population.iter().map(|o| o.power_distance).sum::<f32>() / n;
        let mean_idv = population.iter().map(|o| o.individualism).sum::<f32>() / n;
        let mean_mas = population.iter().map(|o| o.masculinity).sum::<f32>() / n;
        let mean_uai = population
            .iter()
            .map(|o| o.uncertainty_avoidance)
            .sum::<f32>()
            / n;
        let mean_lto = population
            .iter()
            .map(|o| o.long_term_orientation)
            .sum::<f32>()
            / n;
        let mean_ivr = population.iter().map(|o| o.indulgence).sum::<f32>() / n;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compute deviations
        let dev_pdi = (mean_pdi - country_profile.power_distance).abs();
        let dev_idv = (mean_idv - country_profile.individualism).abs();
        let dev_mas = (mean_mas - country_profile.masculinity).abs();
        let dev_uai = (mean_uai - country_profile.uncertainty_avoidance).abs();
        let dev_lto = (mean_lto - country_profile.long_term_orientation).abs();
        let dev_ivr = (mean_ivr - country_profile.indulgence).abs();

        let max_deviation = dev_pdi
            .max(dev_idv)
            .max(dev_mas)
            .max(dev_uai)
            .max(dev_lto)
            .max(dev_ivr);

        // Compute variance
        let var_pdi = population
            .iter()
            .map(|o| (o.power_distance - mean_pdi).powi(2))
            .sum::<f32>()
            / n;

        // Estimate ICC (simplified)
        let within_var = var_pdi;
        let between_var = (mean_pdi - 50.0).powi(2);
        let total_var = within_var + between_var;
        let icc_estimate = if total_var > 0.0 {
            between_var / total_var
        } else {
            0.0
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Validation: population mean should be within 5 points of country mean
        let passed = max_deviation < 5.0;

        Self {
            mean_power_distance: mean_pdi,
            mean_individualism: mean_idv,
            mean_masculinity: mean_mas,
            mean_uncertainty_avoidance: mean_uai,
            mean_long_term_orientation: mean_lto,
            mean_indulgence: mean_ivr,
            within_country_variance: within_var / total_var,
            icc_estimate,
            max_deviation,
            passed,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute population-level Hofstede profile from individual agents
///
/// This is the VALID way to get "national culture" - aggregate up from individuals,
/// not assign down from nations to individuals (which would be ecological fallacy).
///
/// Per Taras et al. (2016): Population means should approximate Hofstede country scores.
#[must_use]
pub fn compute_population_culture(
    orientations: &[IndividualCulturalOrientation],
) -> Option<HofstedeCountryProfile> {
    let n = orientations.len();

    if n == 0 {
        return None;
    }

    let n_f32 = (n as f32).max(1.0);

    let mean_pdi = orientations.iter().map(|o| o.power_distance).sum::<f32>() / n_f32;
    let mean_idv = orientations.iter().map(|o| o.individualism).sum::<f32>() / n_f32;
    let mean_mas = orientations.iter().map(|o| o.masculinity).sum::<f32>() / n_f32;
    let mean_uai = orientations
        .iter()
        .map(|o| o.uncertainty_avoidance)
        .sum::<f32>()
        / n_f32;
    let mean_lto = orientations
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .iter()
        .map(|o| o.long_term_orientation)
        .sum::<f32>()
        / n_f32;
    let mean_ivr = orientations.iter().map(|o| o.indulgence).sum::<f32>() / n_f32;

    // Compute tightness from variance
    let var_pdi = orientations
        .iter()
        .map(|o| (o.power_distance - mean_pdi).powi(2))
        .sum::<f32>()
        / n_f32;
    let tightness = (1.0 - (var_pdi.sqrt() / 50.0)).clamp(0.0, 1.0);

    Some(HofstedeCountryProfile {
        country_code: "SIM".to_string(),
        power_distance: mean_pdi,
        individualism: mean_idv,
        masculinity: mean_mas,
        uncertainty_avoidance: mean_uai,
        long_term_orientation: mean_lto,
        indulgence: mean_ivr,
        tightness,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Helper functions for test profiles
    fn hofstede_usa() -> HofstedeCountryProfile {
        crate::culture::profiles::hofstede_usa()
    }
    fn hofstede_china() -> HofstedeCountryProfile {
        crate::culture::profiles::hofstede_china()
    }
    fn hofstede_japan() -> HofstedeCountryProfile {
        crate::culture::profiles::hofstede_japan()
    }
    fn hofstede_norway() -> HofstedeCountryProfile {
        crate::culture::profiles::hofstede_norway()
    }

    #[test]
    fn test_individual_orientation_creation() {
        let orientation = IndividualCulturalOrientation::new(40.0, 91.0, 62.0, 46.0, 26.0, 68.0);

        assert_eq!(orientation.power_distance, 40.0);
        assert_eq!(orientation.individualism, 91.0);
        assert_eq!(orientation.to_array(), [40.0, 91.0, 62.0, 46.0, 26.0, 68.0]);
    }

    #[test]
    fn test_clamping() {
        // Test that values are clamped to [0, 100]
        let orientation = IndividualCulturalOrientation::new(-10.0, 150.0, 50.0, 50.0, 50.0, 50.0);

        assert_eq!(orientation.power_distance, 0.0);
        assert_eq!(orientation.individualism, 100.0);
    }

    #[test]
    fn test_cvscale_sampling_variance() {
        let mut rng = StdRng::seed_from_u64(42);
        let usa = hofstede_usa();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Generate 1000 agents
        let n = 1000;
        let agents: Vec<_> = (0..n)
            .map(|_| IndividualCulturalOrientation::from_country_profile(&usa, 0.80, &mut rng))
            .collect();

        // Population mean should approximate country mean
        let mean_idv: f32 = agents.iter().map(|a| a.individualism).sum::<f32>() / n as f32;
        let expected_idv = 91.0; // USA individualism
        let deviation = (mean_idv - expected_idv).abs();
        assert!(
            deviation < 5.0,
            "Population mean {mean_idv} should be within 5 points of country mean {expected_idv}, got deviation {deviation}"
        );

        // Within-country variance should be substantial
        let var_idv = agents
            .iter()
            .map(|a| (a.individualism - mean_idv).powi(2))
            .sum::<f32>()
            / n as f32;
        assert!(
            var_idv > 50.0,
            "Within-country variance should be substantial, got {var_idv}"
        );
    }

    #[test]
    fn test_tightness_variance_relationship() {
        let mut rng = StdRng::seed_from_u64(42);

        // Japan is a tight culture
        let japan = hofstede_japan();
        let japan_agents: Vec<_> = (0..500)
            .map(|_| IndividualCulturalOrientation::from_country_profile(&japan, 0.80, &mut rng))
            .collect();

        // Norway is a looser culture
        let norway = hofstede_norway();
        let norway_agents: Vec<_> = (0..500)
            .map(|_| IndividualCulturalOrientation::from_country_profile(&norway, 0.80, &mut rng))
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let japan_mean_pd: f32 = japan_agents.iter().map(|a| a.power_distance).sum::<f32>() / 500.0;
        let norway_mean_pd: f32 =
            norway_agents.iter().map(|a| a.power_distance).sum::<f32>() / 500.0;
        let japan_var = japan_agents
            .iter()
            .map(|a| (a.power_distance - japan_mean_pd).powi(2))
            .sum::<f32>()
            / 500.0;
        let norway_var = norway_agents
            .iter()
            .map(|a| (a.power_distance - norway_mean_pd).powi(2))
            .sum::<f32>()
            / 500.0;

        // Norway (looser) should have higher variance than Japan (tighter)
        // Note: Due to random sampling, this might occasionally fail
        // We use a seed for determinism
        assert!(
            norway_var >= japan_var * 0.7, // Allow some tolerance
            "Loose culture (Norway) should have comparable or higher variance than tight culture (Japan)"
        );
    }

    #[test]
    fn test_generator_population_validation() {
        let mut rng = StdRng::seed_from_u64(42);
        let generator = CulturalOrientationGenerator::new(hofstede_usa());

        let (population, validation) = generator.generate_population_validated(500, &mut rng);

        assert_eq!(population.len(), 500);
        assert!(
            validation.max_deviation < 5.0,
            "Population mean should be within 5 points of country mean, got deviation {}",
            validation.max_deviation
        );
        assert!(validation.passed);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_distance_from_profile() {
        let orientation = IndividualCulturalOrientation::new(40.0, 91.0, 62.0, 46.0, 26.0, 68.0);

        // Distance to USA profile should be small
        let distance_usa = orientation.distance_from_profile(&hofstede_usa());

        // Distance to China profile should be larger
        let distance_china = orientation.distance_from_profile(&hofstede_china());

        assert!(
            distance_china > distance_usa,
            "USA-like orientation should be closer to USA profile than China profile"
        );
    }

    #[test]
    fn test_closest_profile() {
        let usa_like = IndividualCulturalOrientation::new(40.0, 91.0, 62.0, 46.0, 26.0, 68.0);

        let china_like = IndividualCulturalOrientation::new(80.0, 20.0, 66.0, 30.0, 87.0, 24.0);

        let profiles = vec![hofstede_usa(), hofstede_china()];

        assert_eq!(usa_like.closest_profile(&profiles).country_code, "USA");
        assert_eq!(china_like.closest_profile(&profiles).country_code, "CHN");
    }

    #[test]
    fn test_population_culture_computation() {
        let orientations = vec![
            IndividualCulturalOrientation::new(40.0, 90.0, 60.0, 45.0, 25.0, 65.0),
            IndividualCulturalOrientation::new(42.0, 92.0, 64.0, 47.0, 27.0, 71.0),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let profile = compute_population_culture(&orientations).unwrap();

        assert!((profile.power_distance - 41.0).abs() < 1.0);
        assert!((profile.individualism - 91.0).abs() < 1.0);
    }

    #[test]
    fn test_cross_country_variance() {
        // Generate populations from different countries
        let mut rng = StdRng::seed_from_u64(42);

        let usa_gen = CulturalOrientationGenerator::new(hofstede_usa());
        let china_gen = CulturalOrientationGenerator::new(hofstede_china());

        let usa_pop = usa_gen.generate_population(100, &mut rng);
        let china_pop = china_gen.generate_population(100, &mut rng);

        // Between-country variance should be detectable
        let usa_mean_idv = usa_pop.iter().map(|o| o.individualism).sum::<f32>() / 100.0;
        let china_mean_idv = china_pop.iter().map(|o| o.individualism).sum::<f32>() / 100.0;

        // USA should have higher individualism than China
        assert!(
            usa_mean_idv > china_mean_idv + 50.0,
            "USA mean IDV ({usa_mean_idv}) should be much higher than China mean IDV ({china_mean_idv})"
        );
    }
}
