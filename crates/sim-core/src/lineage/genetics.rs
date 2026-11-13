use serde::{Deserialize, Serialize};

// ============================================================================
// WRIGHT-FISHER POPULATION GENETICS MODEL
// Standard model for genetic drift and allele frequency dynamics
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Represents an allele (variant of a gene/trait)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Allele {
    /// Dominant allele (e.g., A)
    Dominant,
    /// Recessive allele (e.g., a)
    Recessive,
}

/// Genotype for a single locus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Genotype {
    /// Homozygous dominant (AA)
    HomozygousDominant,
    /// Heterozygous (Aa)
    Heterozygous,
    /// Homozygous recessive (aa)
    HomozygousRecessive,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Genotype {
    /// Create genotype from two alleles
    #[must_use]
    pub const fn from_alleles(a1: Allele, a2: Allele) -> Self {
        match (a1, a2) {
            (Allele::Dominant, Allele::Dominant) => Self::HomozygousDominant,
            (Allele::Recessive, Allele::Recessive) => Self::HomozygousRecessive,
            _ => Self::Heterozygous,
        }
    }

    /// Get phenotypic value for additive trait
    /// AA = 1.0, Aa = 0.5, aa = 0.0
    #[must_use]
    pub const fn additive_value(&self) -> f32 {
        match self {
            Self::HomozygousDominant => 1.0,
            Self::Heterozygous => 0.5,
            Self::HomozygousRecessive => 0.0,
        }
    }

    /// Check if genotype shows dominant phenotype
    #[must_use]
    pub const fn shows_dominant_phenotype(&self) -> bool {
        matches!(self, Self::HomozygousDominant | Self::Heterozygous)
    }
}

/// Allele frequencies at a locus
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AlleleFrequencies {
    /// Frequency of dominant allele (p)
    pub p: f32,
    /// Frequency of recessive allele (q = 1 - p)
    pub q: f32,
    /// Generation number
    pub generation: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AlleleFrequencies {
    /// Create from dominant allele frequency
    #[must_use]
    pub fn from_p(p: f32, generation: u32) -> Self {
        let p = p.clamp(0.0, 1.0);
        Self {
            p,
            q: 1.0 - p,
            generation,
        }
    }

    /// Expected genotype frequencies under Hardy-Weinberg equilibrium
    /// AA: p², Aa: 2pq, aa: q²
    #[must_use]
    pub fn hardy_weinberg(&self) -> (f32, f32, f32) {
        let aa_freq = self.p * self.p; // Homozygous dominant
        let het_freq = 2.0 * self.p * self.q; // Heterozygous
        let rr_freq = self.q * self.q; // Homozygous recessive
        (aa_freq, het_freq, rr_freq)
    }

    /// Expected heterozygosity (2pq)
    #[must_use]
    pub fn expected_heterozygosity(&self) -> f32 {
        2.0 * self.p * self.q
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Allele frequency tracker for population genetics analysis
///
/// **NOTE ON NAMING** (P0-FIX): This struct was previously named `WrightFisherModel` but
/// the simulation uses **overlapping generations** with age-structured reproduction,
/// which violates the classic Wright-Fisher assumptions of discrete non-overlapping
/// generations. The mathematical functions (binomial sampling, heterozygosity decay)
/// remain useful for analysis, but this is NOT a true Wright-Fisher implementation.
///
/// **Theoretical Basis (Modified Application):**
/// - Wright (1931): "Evolution in Mendelian Populations" - drift formulas adapted
/// - Fisher (1930): "The Genetical Theory of Natural Selection" - variance principles
///
/// **Implementation Notes:**
/// 1. Uses overlapping generations (agents have continuous ages)
/// 2. Tracks allele frequencies across simulation ticks (not discrete generations)
/// 3. Mathematical formulas (heterozygosity decay, fixation probability) are approximations
/// 4. Useful for monitoring genetic drift but not for strict WF predictions
///
/// **Key Functions:**
/// - `expected_heterozygosity()`: `H_t` = `H_0` * (1 - 1/(2N))^t (approximation)
/// - `frequency_variance()`: `Var(p_t)` = p_0(1-p_0)(1 - (1-1/(2N))^t)
/// - `fixation_probability()`: For neutral alleles ≈ initial frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlleleFrequencyTracker {
    /// Effective population size (Ne)
    pub population_size: u32,
    /// Allele frequency history
    pub frequency_trajectory: Vec<AlleleFrequencies>,
    /// Initial frequency
    pub initial_p: f32,
    /// Current generation
    pub current_generation: u32,
}

impl AlleleFrequencyTracker {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new Wright-Fisher model
    ///
    /// # Arguments
    /// * `population_size` - Effective population size (Ne)
    /// * `initial_p` - Initial frequency of dominant allele
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn new(population_size: u32, initial_p: f32) -> Self {
        let initial_p = initial_p.clamp(0.0, 1.0);
        let initial_freq = AlleleFrequencies::from_p(initial_p, 0);

        Self {
            population_size,
            frequency_trajectory: vec![initial_freq],
            initial_p,
            current_generation: 0,
        }
    }

    /// Simulate one generation of genetic drift
    ///
    /// Uses binomial sampling: each of 2N alleles is sampled
    /// independently from previous generation
    ///
    /// **Formula:**
    /// p_{t+1} ~ Binomial(2N, `p_t`) / (2N)
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn step<R: rand::Rng>(&mut self, rng: &mut R) {
        let Some(last) = self.frequency_trajectory.last() else {
            return;
        };
        let current_p = last.p;

        // Binomial sampling: 2N alleles sampled with probability p
        let n_alleles = 2 * self.population_size;
        let n_dominant = sample_binomial(rng, n_alleles, current_p);

        let new_p = n_dominant as f32 / n_alleles as f32;
        self.current_generation += 1;

        self.frequency_trajectory
            .push(AlleleFrequencies::from_p(new_p, self.current_generation));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simulate multiple generations
    pub fn simulate_generations<R: rand::Rng>(&mut self, generations: u32, rng: &mut R) {
        for _ in 0..generations {
            // Check for fixation
            if let Some(last) = self.frequency_trajectory.last() {
                if last.p <= 0.0 || last.p >= 1.0 {
                    break; // Fixed
                }
            }
            self.step(rng);
        }
    }

    /// Check if allele has fixed (p = 0 or p = 1)
    #[must_use]
    pub fn is_fixed(&self) -> bool {
        self.frequency_trajectory
            .last()
            .is_some_and(|f| f.p <= 0.0 || f.p >= 1.0)
    }

    /// Get fixation status
    #[must_use]
    pub fn fixation_status(&self) -> Option<FixationStatus> {
        self.frequency_trajectory.last().map(|f| {
            if f.p <= 0.0 {
                FixationStatus::FixedRecessive
            } else if f.p >= 1.0 {
                FixationStatus::FixedDominant
            } else {
                FixationStatus::Segregating
            }
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Time to fixation (if fixed)
    #[must_use]
    pub fn time_to_fixation(&self) -> Option<u32> {
        if self.is_fixed() {
            Some(self.current_generation)
        } else {
            None
        }
    }

    /// Expected heterozygosity at current generation
    /// Under drift: `H_t` = `H_0` * (1 - 1/(2N))^t
    #[must_use]
    pub fn expected_heterozygosity(&self) -> f32 {
        let initial_h = 2.0 * self.initial_p * (1.0 - self.initial_p);
        let decay_factor =
            (1.0 - 1.0 / (2.0 * self.population_size as f32)).powi(self.current_generation as i32);
        initial_h * decay_factor
    }

    /// Variance in allele frequency
    /// `Var(p_t)` = p_0(1-p_0) * (1 - (1 - 1/(2N))^t)
    #[must_use]
    pub fn frequency_variance(&self) -> f32 {
        let p0 = self.initial_p;
        let decay =
            (1.0 - 1.0 / (2.0 * self.population_size as f32)).powi(self.current_generation as i32);
        p0 * (1.0 - p0) * (1.0 - decay)
    }

    /// Probability of fixation for neutral allele
    /// For neutral allele: = initial frequency
    #[must_use]
    pub const fn fixation_probability(&self) -> f32 {
        self.initial_p // Neutral drift: prob = initial freq
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Fixation status of an allele
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixationStatus {
    /// Allele is still segregating in population
    Segregating,
    /// Dominant allele fixed (p = 1)
    FixedDominant,
    /// Recessive allele fixed (p = 0)
    FixedRecessive,
}

/// Population genetic statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationGeneticsStats {
    /// Effective population size (Ne)
    pub effective_population_size: u32,
    /// Observed heterozygosity (`H_o`)
    pub observed_heterozygosity: f32,
    /// Expected heterozygosity (`H_e`)
    pub expected_heterozygosity: f32,
    /// Inbreeding coefficient (F = 1 - `H_o/H_e`)
    pub inbreeding_coefficient: f32,
    /// Number of segregating loci
    pub segregating_loci: usize,
    /// Fixation events
    pub fixation_events: u32,
}

/// Binomial sampling helper
/// Samples from Binomial(n, p) using the normal approximation for large n
pub fn sample_binomial<R: rand::Rng>(rng: &mut R, n: u32, p: f32) -> u32 {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    if n == 0 || p <= 0.0 {
        return 0;
    }
    if p >= 1.0 {
        return n;
    }

    // For small n, use direct sampling
    if n <= 30 {
        let mut successes = 0u32;
        for _ in 0..n {
            if rng.gen::<f32>() < p {
                successes += 1;
            }
        }
        return successes;
    }

    // For large n, use normal approximation via Box-Muller transform
    let mean = n as f32 * p;
    let variance = mean * (1.0 - p);
    let std_dev = variance.sqrt();

    // Box-Muller transform for standard normal
    let u1: f32 = rng.gen();
    let u2: f32 = rng.gen();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();

    let sample = mean + std_dev * z;
    sample.max(0.0).min(n as f32).round() as u32
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute linkage disequilibrium D between two loci
/// D = freq(AB) * freq(ab) - freq(Ab) * freq(aB)
///
/// **Range:** [-0.25, 0.25] (max when all four combinations are 0.25)
#[must_use]
pub fn linkage_disequilibrium(
    freq_ab_dominant: f32,  // Both dominant (AB)
    freq_ab_hetero1: f32,   // L1 dominant, L2 recessive (Ab)
    freq_ab_hetero2: f32,   // L1 recessive, L2 dominant (aB)
    freq_ab_recessive: f32, // Both recessive (ab)
) -> f32 {
    let d = freq_ab_dominant * freq_ab_recessive - freq_ab_hetero1 * freq_ab_hetero2;
    d.clamp(-0.25, 0.25)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Normalized linkage disequilibrium (D')
/// Ranges from -1 to 1
#[must_use]
pub fn normalized_ld(d: f32, p_a: f32, p_b: f32) -> f32 {
    let p_a = p_a.clamp(0.001, 0.999);
    let p_b = p_b.clamp(0.001, 0.999);
    let q_a = 1.0 - p_a;
    let q_b = 1.0 - p_b;

    let d_max = if d >= 0.0 {
        d.min(p_a * q_b).min(q_a * p_b)
    } else {
        d.max(-p_a * p_b).max(-q_a * q_b)
    };

    if d_max.abs() < 1e-10 {
        0.0
    } else {
        (d / d_max).clamp(-1.0, 1.0)
    }
}

/// Tajima's D statistic for detecting selection
/// Compares two estimators of θ (4Nμ)
///
/// **Interpretation:**
/// * D ≈ 0: Neutral evolution
/// * D < -2: Purifying selection or population expansion
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// * D > 2: Balancing selection or population contraction
#[must_use]
pub fn tajimas_d(segregating_sites: u32, sample_size: u32, avg_pairwise_diff: f32) -> Option<f32> {
    if sample_size < 2 || segregating_sites == 0 {
        return None;
    }

    let n = sample_size as f64;
    let s = segregating_sites as f64;
    let pi = avg_pairwise_diff as f64;

    // a1 = sum of 1/i for i=1 to n-1
    let a1: f64 = (1..sample_size).map(|i| 1.0 / i as f64).sum();

    // a2 = sum of 1/i² for i=1 to n-1
    let a2: f64 = (1..sample_size).map(|i| 1.0 / (i * i) as f64).sum();

    let b1 = (n + 1.0) / (3.0 * (n - 1.0));
    let b2 = 2.0 * (n * n + n + 3.0) / (9.0 * n * (n - 1.0));

    let c1 = b1 - 1.0 / a1;
    let c2 = b2 - (n + 2.0) / (a1 * n) + a2 / (a1 * a1);

    let e1 = c1 / a1;
    let e2 = c2 / (a1 * a1 + a2);

    let d = (pi - s / a1) / (e1 * s + e2 * s * (s - 1.0)).sqrt();

    Some(d as f32)
}
