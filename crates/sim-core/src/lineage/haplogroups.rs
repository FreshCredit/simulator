use crate::types::AgentId;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// P2-1: MITOCHONDRIAL AND Y-CHROMOSOME HAPLOGROUP TRACKING
// Per Cavalli-Sforza: Maternal (mtDNA) and paternal (Y-DNA) lineage analysis
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of genetic lineage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineageType {
    /// Maternal lineage (mitochondrial DNA)
    /// Inherited from mother only, no recombination
    Mitochondrial,
    /// Paternal lineage (Y-chromosome)
    /// Inherited from father only, no recombination
    YChromosome,
    /// Autosomal (regular chromosomes)
    /// Inherited from both parents with recombination
    Autosomal,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Haplogroup marker
/// Represents a specific mutation that defines a haplogroup
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HaplogroupMarker {
    /// Marker ID (e.g., "M173" for R1b)
    pub id: u32,
    /// Haplogroup letter designation (e.g., 'R', 'J', 'E')
    pub haplogroup: char,
    /// Subclade (e.g., "1b", "2", "a1")
    pub subclade: String,
    /// When this marker arose (generation)
    pub origin_generation: u32,
    /// Geographic region of origin
    pub origin_region: String,
}

/// Haplogroup database for major human Y-DNA and mtDNA haplogroups
#[derive(Debug)]
pub struct HaplogroupDatabase;

impl HaplogroupDatabase {
    /// Major Y-DNA haplogroups
    #[must_use]
    pub fn y_haplogroups() -> Vec<HaplogroupMarker> {
        vec![
            // Major branches
            HaplogroupMarker {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                id: 1,
                haplogroup: 'A',
                subclade: String::new(),
                origin_generation: 0,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 2,
                haplogroup: 'B',
                subclade: String::new(),
                origin_generation: 100,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 3,
                haplogroup: 'C',
                subclade: String::new(),
                origin_generation: 200,
                origin_region: "Asia".to_string(),
            },
            HaplogroupMarker {
                id: 4,
                haplogroup: 'D',
                subclade: String::new(),
                origin_generation: 200,
                origin_region: "Asia".to_string(),
            },
            HaplogroupMarker {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                id: 5,
                haplogroup: 'E',
                subclade: String::new(),
                origin_generation: 150,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 6,
                haplogroup: 'F',
                subclade: String::new(),
                origin_generation: 250,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 7,
                haplogroup: 'G',
                subclade: String::new(),
                origin_generation: 300,
                origin_region: "Middle East".to_string(),
            },
            HaplogroupMarker {
                id: 8,
                haplogroup: 'H',
                subclade: String::new(),
                origin_generation: 300,
                origin_region: "South Asia".to_string(),
            },
            HaplogroupMarker {
                id: 9,
                haplogroup: 'I',
                subclade: String::new(),
                origin_generation: 350,
                origin_region: "Europe".to_string(),
            },
            HaplogroupMarker {
                id: 10,
                haplogroup: 'J',
                // TAG: surface=simulation owner=research-team rule=SIM-001
                subclade: String::new(),
                origin_generation: 300,
                origin_region: "Middle East".to_string(),
            },
            HaplogroupMarker {
                id: 11,
                haplogroup: 'K',
                subclade: String::new(),
                origin_generation: 280,
                origin_region: "Asia".to_string(),
            },
            HaplogroupMarker {
                id: 12,
                haplogroup: 'L',
                subclade: String::new(),
                origin_generation: 320,
                origin_region: "South Asia".to_string(),
            },
            HaplogroupMarker {
                id: 13,
                haplogroup: 'N',
                subclade: String::new(),
                origin_generation: 350,
                origin_region: "Asia".to_string(),
            },
            HaplogroupMarker {
                id: 14,
                haplogroup: 'O',
                subclade: String::new(),
                origin_generation: 350,
                origin_region: "East Asia".to_string(),
            },
            HaplogroupMarker {
                id: 15,
                haplogroup: 'Q',
                subclade: String::new(),
                origin_generation: 300,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                origin_region: "Central Asia".to_string(),
            },
            HaplogroupMarker {
                id: 16,
                haplogroup: 'R',
                subclade: String::new(),
                origin_generation: 300,
                origin_region: "Central Asia".to_string(),
            },
            // Subclades
            HaplogroupMarker {
                id: 101,
                haplogroup: 'R',
                subclade: "1a".to_string(),
                origin_generation: 400,
                origin_region: "Europe".to_string(),
            },
            HaplogroupMarker {
                id: 102,
                haplogroup: 'R',
                subclade: "1b".to_string(),
                origin_generation: 400,
                origin_region: "Europe".to_string(),
            },
            HaplogroupMarker {
                id: 103,
                haplogroup: 'J',
                subclade: "1".to_string(),
                origin_generation: 380,
                origin_region: "Middle East".to_string(),
            },
            HaplogroupMarker {
                id: 104,
                haplogroup: 'J',
                subclade: "2".to_string(),
                origin_generation: 380,
                origin_region: "Middle East".to_string(),
            },
        ]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Major mtDNA haplogroups
    #[must_use]
    pub fn mtdna_haplogroups() -> Vec<HaplogroupMarker> {
        vec![
            // Macro-haplogroups
            HaplogroupMarker {
                id: 1001,
                haplogroup: 'L',
                subclade: String::new(),
                origin_generation: 0,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 1002,
                haplogroup: 'M',
                subclade: String::new(),
                origin_generation: 500,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 1003,
                haplogroup: 'N',
                subclade: String::new(),
                origin_generation: 500,
                origin_region: "Africa".to_string(),
            },
            // L subclades (African)
            HaplogroupMarker {
                id: 1101,
                haplogroup: 'L',
                subclade: "0".to_string(),
                origin_generation: 200,
                origin_region: "Africa".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
            },
            HaplogroupMarker {
                id: 1102,
                haplogroup: 'L',
                subclade: "1".to_string(),
                origin_generation: 250,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 1103,
                haplogroup: 'L',
                subclade: "2".to_string(),
                origin_generation: 250,
                origin_region: "Africa".to_string(),
            },
            HaplogroupMarker {
                id: 1104,
                haplogroup: 'L',
                subclade: "3".to_string(),
                origin_generation: 300,
                origin_region: "Africa".to_string(),
            },
            // M subclades (Asian)
            HaplogroupMarker {
                id: 1201,
                haplogroup: 'M',
                subclade: "1".to_string(),
                origin_generation: 600,
                origin_region: "East Asia".to_string(),
            },
            HaplogroupMarker {
                id: 1202,
                haplogroup: 'M',
                subclade: "2".to_string(),
                origin_generation: 600,
                origin_region: "South Asia".to_string(),
            },
            // TAG: surface=simulation owner=research-team rule=SIM-001
            // N subclades (Eurasian)
            HaplogroupMarker {
                id: 1301,
                haplogroup: 'N',
                subclade: "1".to_string(),
                origin_generation: 600,
                origin_region: "Europe".to_string(),
            },
            HaplogroupMarker {
                id: 1302,
                haplogroup: 'N',
                subclade: "2".to_string(),
                origin_generation: 600,
                origin_region: "Europe".to_string(),
            },
            HaplogroupMarker {
                id: 1303,
                haplogroup: 'A',
                subclade: String::new(),
                origin_generation: 700,
                origin_region: "Asia".to_string(),
            },
            HaplogroupMarker {
                id: 1304,
                haplogroup: 'B',
                subclade: String::new(),
                origin_generation: 700,
                origin_region: "Asia".to_string(),
            },
            HaplogroupMarker {
                id: 1305,
                haplogroup: 'X',
                subclade: String::new(),
                origin_generation: 650,
                origin_region: "West Eurasia".to_string(),
            },
        ]
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Haplogroup assignment for an individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaplogroupAssignment {
    /// Type of lineage
    pub lineage_type: LineageType,
    /// Assigned haplogroup marker
    pub marker: HaplogroupMarker,
    /// Confidence of assignment [0, 1]
    pub confidence: f32,
    /// Ancestral haplogroups (for tracing lineage)
    pub ancestral_path: Vec<u32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Maternal lineage tracker (mtDNA)
/// Tracks inheritance through the mother line only
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaternalLineage {
    /// Current haplogroup marker ID
    pub mtdna_haplogroup: Option<u32>,
    /// Matrilineal ancestor chain
    pub maternal_ancestors: Vec<AgentId>,
    /// Number of generations tracked
    pub generations_tracked: u32,
    /// Mutation history
    pub mutations: Vec<(u32, u32)>, // (generation, marker_id)
}

impl MaternalLineage {
    /// Create new maternal lineage from mother
    #[must_use]
    pub fn from_mother(mother_id: AgentId, mother_haplogroup: Option<u32>) -> Self {
        let mut ancestors = Vec::new();
        if mother_haplogroup.is_some() {
            ancestors.push(mother_id);
        }

        Self {
            mtdna_haplogroup: mother_haplogroup,
            maternal_ancestors: ancestors,
            generations_tracked: 1,
            mutations: Vec::new(),
        }
    }

    /// Add mutation at current generation
    pub fn add_mutation(&mut self, generation: u32, new_marker: u32) {
        if let Some(old_marker) = self.mtdna_haplogroup {
            if old_marker != new_marker {
                self.mutations.push((generation, new_marker));
                self.mtdna_haplogroup = Some(new_marker);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get haplogroup name
    #[must_use]
    pub fn haplogroup_name(&self) -> String {
        match self.mtdna_haplogroup {
            None => "Unknown".to_string(),
            Some(id) => {
                let markers = HaplogroupDatabase::mtdna_haplogroups();
                markers.iter().find(|m| m.id == id).map_or_else(
                    || format!("Haplo-{id}"),
                    |m| format!("{}{}", m.haplogroup, m.subclade),
                )
            }
        }
    }
}

/// Paternal lineage tracker (Y-DNA)
/// Tracks inheritance through the father line only
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaternalLineage {
    /// Current haplogroup marker ID
    pub y_haplogroup: Option<u32>,
    /// Patrilineal ancestor chain
    pub paternal_ancestors: Vec<AgentId>,
    /// Number of generations tracked
    pub generations_tracked: u32,
    /// Mutation history
    pub mutations: Vec<(u32, u32)>, // (generation, marker_id)
    /// Surname/lineage name (if applicable)
    pub lineage_name: Option<String>,
}

impl PaternalLineage {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new paternal lineage from father
    #[must_use]
    pub fn from_father(father_id: AgentId, father_haplogroup: Option<u32>) -> Self {
        let mut ancestors = Vec::new();
        if father_haplogroup.is_some() {
            ancestors.push(father_id);
        }

        Self {
            y_haplogroup: father_haplogroup,
            paternal_ancestors: ancestors,
            generations_tracked: 1,
            mutations: Vec::new(),
            lineage_name: None,
        }
    }

    /// Add mutation at current generation
    pub fn add_mutation(&mut self, generation: u32, new_marker: u32) {
        if let Some(old_marker) = self.y_haplogroup {
            if old_marker != new_marker {
                self.mutations.push((generation, new_marker));
                self.y_haplogroup = Some(new_marker);
            }
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Get haplogroup name
    #[must_use]
    pub fn haplogroup_name(&self) -> String {
        match self.y_haplogroup {
            None => "Unknown".to_string(),
            Some(id) => {
                let markers = HaplogroupDatabase::y_haplogroups();
                markers.iter().find(|m| m.id == id).map_or_else(
                    || format!("Haplo-{id}"),
                    |m| format!("{}{}", m.haplogroup, m.subclade),
                )
            }
        }
    }

    /// Check if two lineages share a common ancestor (same haplogroup)
    #[must_use]
    pub const fn shares_ancestor_with(&self, other: &Self) -> bool {
        match (self.y_haplogroup, other.y_haplogroup) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Population haplogroup frequency tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationHaplogroups {
    /// Y-DNA haplogroup frequencies
    pub y_haplogroup_freqs: HashMap<u32, f32>,
    /// mtDNA haplogroup frequencies
    pub mtdna_haplogroup_freqs: HashMap<u32, f32>,
    /// Sample size
    pub sample_size: u32,
}

impl PopulationHaplogroups {
    /// Create from population sample
    pub fn from_population<R: Rng>(
        population_size: u32,
        y_distribution: &[(u32, f32)],
        mtdna_distribution: &[(u32, f32)],
        rng: &mut R,
    ) -> Self {
        let mut y_freqs = HashMap::new();
        let mut mtdna_freqs = HashMap::new();

        // Assign Y-haplogroups
        for _ in 0..population_size {
            let r: f32 = rng.gen();
            let mut cumsum = 0.0;
            for (haplo, freq) in y_distribution {
                cumsum += freq;
                if r < cumsum {
                    *y_freqs.entry(*haplo).or_insert(0.0) += 1.0;
                    break;
                }
            }
        }

        // Assign mtDNA haplogroups
        for _ in 0..population_size {
            let r: f32 = rng.gen();
            let mut cumsum = 0.0;
            for (haplo, freq) in mtdna_distribution {
                cumsum += freq;
                if r < cumsum {
                    *mtdna_freqs.entry(*haplo).or_insert(0.0) += 1.0;
                    break;
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Normalize
        for count in y_freqs.values_mut() {
            *count /= population_size as f32;
        }
        for count in mtdna_freqs.values_mut() {
            *count /= population_size as f32;
        }

        Self {
            y_haplogroup_freqs: y_freqs,
            mtdna_haplogroup_freqs: mtdna_freqs,
            sample_size: population_size,
        }
    }

    /// Get most common Y-haplogroup
    #[must_use]
    pub fn most_common_y(&self) -> Option<(u32, f32)> {
        self.y_haplogroup_freqs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&k, &v)| (k, v))
    }

    /// Get most common mtDNA haplogroup
    #[must_use]
    pub fn most_common_mtdna(&self) -> Option<(u32, f32)> {
        self.mtdna_haplogroup_freqs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&k, &v)| (k, v))
    }
}
