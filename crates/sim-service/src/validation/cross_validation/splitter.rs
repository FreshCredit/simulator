//! Data splitters for cross-validation
//!
//! Generates train/validation splits for various CV strategies.

use rand::rngs::StdRng;
use rand::{seq::SliceRandom, SeedableRng};
use std::collections::{BTreeMap, HashMap};

use super::types::{Dataset, Fold};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trait for data splitters
pub trait Splitter {
    /// Generate folds
    fn split(&self, dataset: &Dataset) -> Vec<Fold>;
}

/// Standard train-test split
#[derive(Debug)]
pub struct TrainTestSplit {
    /// test size
    pub test_size: f64,
    /// shuffle
    pub shuffle: bool,
    /// seed
    pub seed: Option<u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Splitter for TrainTestSplit {
    fn split(&self, dataset: &Dataset) -> Vec<Fold> {
        let n = dataset.target.len().min(dataset.features.len());
        if n == 0 {
            return Vec::new();
        }
        let n_test = if n >= 2 {
            ((n as f64 * self.test_size).round() as usize).clamp(1, n - 1)
        } else {
            0
        };
        let n_train = n - n_test;

        let mut indices: Vec<usize> = (0..n).collect();

        if self.shuffle {
            let mut rng = match self.seed {
                Some(s) => StdRng::seed_from_u64(s),
                None => StdRng::from_entropy(),
            };
            indices.shuffle(&mut rng);
        }

        vec![Fold {
            index: 0,
            train_indices: indices[..n_train].to_vec(),
            val_indices: indices[n_train..].to_vec(),
        }]
    }
}

/// K-fold splitter
#[derive(Debug)]
pub struct KFold {
    /// n splits
    pub n_splits: usize,
    /// shuffle
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub shuffle: bool,
    /// seed
    pub seed: Option<u64>,
}

impl Splitter for KFold {
    fn split(&self, dataset: &Dataset) -> Vec<Fold> {
        let n = dataset.target.len().min(dataset.features.len());
        if self.n_splits == 0 || n == 0 {
            return Vec::new();
        }
        let k = self.n_splits.min(n);
        let base_size = n / k;
        let remainder = n % k;

        let mut indices: Vec<usize> = (0..n).collect();

        if self.shuffle {
            let mut rng = match self.seed {
                Some(s) => StdRng::seed_from_u64(s),
                None => StdRng::from_entropy(),
            };
            indices.shuffle(&mut rng);
        }

        let mut folds = Vec::with_capacity(k);
        let mut start = 0;

        for i in 0..k {
            let fold_size = base_size + if i < remainder { 1 } else { 0 };
            let end = (start + fold_size).min(n);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let val_indices = indices[start..end].to_vec();
            let train_indices: Vec<usize> = indices[..start]
                .iter()
                .chain(&indices[end..])
                .copied()
                .collect();

            folds.push(Fold {
                index: i,
                train_indices,
                val_indices,
            });
            start = end;
        }

        folds
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Stratified k-fold splitter (preserves class distribution)
#[derive(Debug)]
pub struct StratifiedKFold {
    /// n splits
    pub n_splits: usize,
    /// shuffle
    pub shuffle: bool,
    /// seed
    pub seed: Option<u64>,
}

impl Splitter for StratifiedKFold {
    fn split(&self, dataset: &Dataset) -> Vec<Fold> {
        let Some(stratify) = dataset.stratify.as_ref() else {
            return Vec::new();
        };
        let n = stratify
            .len()
            .min(dataset.target.len())
            .min(dataset.features.len());
        if self.n_splits == 0 || n == 0 {
            return Vec::new();
        }

        // Group indices by class deterministically using BTreeMap
        let mut class_indices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, label) in stratify.iter().enumerate() {
            class_indices.entry(label.clone()).or_default().push(i);
        }

        // Shuffle each class if requested
        let mut rng = match self.seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        for indices in class_indices.values_mut() {
            if self.shuffle {
                indices.shuffle(&mut rng);
            }
        }

        // Distribute each class round-robin across folds with continuous global indexing
        let effective_splits = self.n_splits.min(n).max(1);
        let mut folds: Vec<Vec<usize>> = vec![Vec::new(); effective_splits];
        let mut global_idx = 0;
        for indices in class_indices.values() {
            for &idx in indices {
                folds[global_idx % effective_splits].push(idx);
                global_idx += 1;
            }
        }

        // Convert to Fold structs
        let mut result = Vec::new();
        let all_indices: Vec<usize> = (0..n).collect();

        for (_i, val_indices) in folds.into_iter().enumerate() {
            if val_indices.is_empty() {
                continue;
            }
            let val_set: std::collections::HashSet<_> = val_indices.iter().copied().collect();
            let train_indices: Vec<_> = all_indices
                .iter()
                .filter(|&&idx| !val_set.contains(&idx))
                .copied()
                .collect();

            if train_indices.is_empty() {
                continue;
            }

            result.push(Fold {
                index: result.len(),
                train_indices,
                val_indices,
            });
        }

        result
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Group k-fold splitter (keeps groups together)
#[derive(Debug)]
pub struct GroupKFold {
    /// n splits
    pub n_splits: usize,
}

impl Splitter for GroupKFold {
    fn split(&self, dataset: &Dataset) -> Vec<Fold> {
        let Some(groups) = dataset.groups.as_ref() else {
            return Vec::new();
        };
        let n = groups
            .len()
            .min(dataset.target.len())
            .min(dataset.features.len());
        if self.n_splits == 0 || n == 0 {
            return Vec::new();
        }

        // Get unique groups
        let mut unique_groups: Vec<String> = groups[..n].to_vec();
        unique_groups.sort();
        unique_groups.dedup();

        if unique_groups.is_empty() {
            return Vec::new();
        }

        // Assign groups to folds round-robin with bounded effective split count
        let effective_splits = self.n_splits.min(unique_groups.len()).max(1);
        let mut group_to_fold: HashMap<String, usize> = HashMap::new();
        for (i, group) in unique_groups.iter().enumerate() {
            group_to_fold.insert(group.clone(), i % effective_splits);
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Create folds
        let mut fold_val_indices: Vec<Vec<usize>> = vec![Vec::new(); effective_splits];

        for (i, group) in groups[..n].iter().enumerate() {
            let fold = *group_to_fold.get(group).unwrap_or(&0);
            fold_val_indices[fold].push(i);
        }

        let all_indices: Vec<usize> = (0..n).collect();
        let mut result = Vec::new();

        for val_indices in fold_val_indices {
            if val_indices.is_empty() {
                continue;
            }
            let val_set: std::collections::HashSet<_> = val_indices.iter().copied().collect();
            let train_indices: Vec<_> = all_indices
                .iter()
                .filter(|&&idx| !val_set.contains(&idx))
                .copied()
                .collect();

            if train_indices.is_empty() {
                continue;
            }

            result.push(Fold {
                index: result.len(),
                train_indices,
                val_indices,
            });
        }

        result
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Time series splitter (respects temporal order)
#[derive(Debug)]
pub struct TimeSeriesSplit {
    /// n splits
    pub n_splits: usize,
    /// max train size
    pub max_train_size: Option<usize>,
    /// test size
    pub test_size: Option<usize>,
}

impl Splitter for TimeSeriesSplit {
    fn split(&self, dataset: &Dataset) -> Vec<Fold> {
        let n = dataset.target.len().min(dataset.features.len());
        if self.n_splits == 0 || n < 2 {
            return Vec::new();
        }

        let k = self.n_splits.min(n - 1).max(1);
        let test_size = self.test_size.unwrap_or_else(|| (n / (k + 1)).max(1));
        if test_size == 0 {
            return Vec::new();
        }
        let total_test = test_size.saturating_mul(k);
        let train_start_base = n.saturating_sub(total_test.saturating_add(test_size));

        let mut folds = Vec::new();

        for i in 0..k {
            let test_end = n
                .saturating_sub((k.saturating_sub(i + 1)).saturating_mul(test_size))
                .min(n);
            let test_start = test_end.saturating_sub(test_size);

            if test_start == 0 || test_start >= test_end {
                continue;
            }

            let train_start = if let Some(max_train) = self.max_train_size {
                test_start.saturating_sub(max_train)
            } else {
                train_start_base.min(test_start)
            };

            let train_indices: Vec<usize> = (train_start..test_start).collect();
            let val_indices: Vec<usize> = (test_start..test_end).collect();

            if train_indices.is_empty() || val_indices.is_empty() {
                continue;
            }

            folds.push(Fold {
                index: folds.len(),
                train_indices,
                val_indices,
            });
        }

        folds
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Leave-one-out splitter
#[derive(Debug)]
pub struct LeaveOneOut;

impl Splitter for LeaveOneOut {
    fn split(&self, dataset: &Dataset) -> Vec<Fold> {
        let n = dataset.target.len().min(dataset.features.len());
        if n < 2 {
            return Vec::new();
        }
        let all_indices: Vec<usize> = (0..n).collect();

        (0..n)
            .map(|i| Fold {
                index: i,
                train_indices: all_indices.iter().filter(|&&x| x != i).copied().collect(),
                val_indices: vec![i],
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset(n: usize) -> Dataset {
        Dataset {
            features: vec![vec![1.0, 2.0]; n],
            target: vec![0.0; n],
            sample_weights: None,
            groups: None,
            stratify: None,
        }
    }

    #[test]
    fn test_kfold_split() {
        let dataset = create_test_dataset(100);
        let kfold = KFold {
            n_splits: 5,
            shuffle: false,
            seed: None,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let folds = kfold.split(&dataset);

        assert_eq!(folds.len(), 5);

        // Each fold should have roughly 20% validation
        for fold in &folds {
            assert_eq!(fold.val_indices.len(), 20);
            assert_eq!(fold.train_indices.len(), 80);
        }

        // All indices should be unique across all folds
        let all_val: Vec<_> = folds.iter().flat_map(|f| &f.val_indices).collect();
        let unique_val: std::collections::HashSet<_> = all_val.iter().collect();
        assert_eq!(all_val.len(), unique_val.len());
    }

    #[test]
    fn test_stratified_kfold() {
        let mut dataset = create_test_dataset(100);
        dataset.stratify = Some(
            (0..100)
                .map(|i| {
                    if i < 50 {
                        "A".to_string()
                    } else {
                        "B".to_string()
                    }
                })
                .collect(),
        );

        let skfold = StratifiedKFold {
            n_splits: 5,
            shuffle: false,
            seed: None,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let folds = skfold.split(&dataset);

        // Check that class distribution is preserved
        for fold in &folds {
            let val_a = fold.val_indices.iter().filter(|&&i| i < 50).count();
            let val_b = fold.val_indices.len() - val_a;

            // Should be roughly 50-50 in each fold
            assert!(
                (8..=12).contains(&val_a),
                "Class A distribution off: {val_a}"
            );
            assert!(
                (8..=12).contains(&val_b),
                "Class B distribution off: {val_b}"
            );
        }
    }

    #[test]
    fn test_stratified_kfold_bounded_spread_uneven_classes() {
        // 3 classes of 6 items = 18 total items across 5 folds
        let mut dataset = create_test_dataset(18);
        dataset.stratify = Some(
            (0..18)
                .map(|i| match i % 3 {
                    0 => "C0".to_string(),
                    1 => "C1".to_string(),
                    _ => "C2".to_string(),
                })
                .collect(),
        );

        let skfold = StratifiedKFold {
            n_splits: 5,
            shuffle: false,
            seed: Some(42),
        };
        let folds = skfold.split(&dataset);
        assert_eq!(folds.len(), 5);

        let sizes: Vec<usize> = folds.iter().map(|f| f.val_indices.len()).collect();
        let max_size = sizes.iter().max().copied().unwrap_or(0);
        let min_size = sizes.iter().min().copied().unwrap_or(0);
        assert!(
            max_size - min_size <= 1,
            "Fold sizes {sizes:?} must differ by at most 1"
        );
    }

    #[test]
    fn test_group_kfold() {
        let mut dataset = create_test_dataset(100);
        dataset.groups = Some((0..100).map(|i| format!("group{}", i / 10)).collect());

        let gkf = GroupKFold { n_splits: 5 };
        let folds = gkf.split(&dataset);

        // Each fold should have 2 groups (10 samples each)
        for fold in &folds {
            let unique_groups: std::collections::HashSet<_> = fold
                .val_indices
                .iter()
                .map(|&i| dataset.groups.as_ref().unwrap()[i].clone())
                .collect();

            // All samples in validation should be from same groups
            assert!(
                unique_groups.len() <= 2,
                "Groups split across folds: {unique_groups:?}"
            );
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_time_series_split() {
        let dataset = create_test_dataset(100);
        let tss = TimeSeriesSplit {
            n_splits: 5,
            max_train_size: None,
            test_size: Some(10),
        };

        let folds = tss.split(&dataset);

        assert_eq!(folds.len(), 5);

        // Check temporal ordering
        for i in 1..folds.len() {
            let prev_train_end = folds[i - 1].train_indices.last().copied().unwrap_or(0);
            let curr_train_end = folds[i].train_indices.last().copied().unwrap_or(0);
            assert!(curr_train_end > prev_train_end);
        }
    }

    #[test]
    fn test_train_test_split() {
        let dataset = create_test_dataset(100);
        let tts = TrainTestSplit {
            test_size: 0.2,
            shuffle: true,
            seed: Some(42),
        };

        let folds = tts.split(&dataset);

        assert_eq!(folds.len(), 1);
        assert_eq!(folds[0].val_indices.len(), 20);
        assert_eq!(folds[0].train_indices.len(), 80);
    }

    #[test]
    fn test_leave_one_out_requires_min_two_samples() {
        let dataset_1 = create_test_dataset(1);
        let folds_1 = LeaveOneOut.split(&dataset_1);
        assert!(folds_1.is_empty(), "LOO on n=1 must return empty folds");

        let dataset_3 = create_test_dataset(3);
        let folds_3 = LeaveOneOut.split(&dataset_3);
        assert_eq!(folds_3.len(), 3);
        for f in &folds_3 {
            assert_eq!(f.train_indices.len(), 2);
            assert_eq!(f.val_indices.len(), 1);
        }
    }

    #[test]
    fn test_train_test_split_non_empty_for_small_n() {
        let dataset_2 = create_test_dataset(2);
        let tts = TrainTestSplit {
            test_size: 0.1, // would round to 0 if not clamped!
            shuffle: false,
            seed: None,
        };
        let folds = tts.split(&dataset_2);
        assert_eq!(folds.len(), 1);
        assert_eq!(folds[0].train_indices.len(), 1);
        assert_eq!(folds[0].val_indices.len(), 1);
    }

    #[test]
    fn test_mismatched_features_and_target_bounded_safely() {
        let mut dataset = create_test_dataset(5);
        dataset.target = vec![0.0, 1.0]; // only 2 targets vs 5 features
        let folds = KFold {
            n_splits: 5,
            shuffle: false,
            seed: None,
        }
        .split(&dataset);
        assert_eq!(folds.len(), 2);
        for f in &folds {
            assert!(f.val_indices.iter().all(|&idx| idx < 2));
            assert!(f.train_indices.iter().all(|&idx| idx < 2));
        }
    }
}
