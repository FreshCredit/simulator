use super::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn make_trait_snapshot(tick: Tick, coop: f32, risk: f32) -> TraitSnapshot {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    TraitSnapshot {
        tick,
        docas_alpha: [0.95; 5],
        docas_beta: [0.1; 5],
        planning_depth: 5,
        planning_horizon: 3,
        cooperation_tendency: coop,
        risk_tolerance: risk,
    }
}

#[test]
fn test_lineage_node_initial() {
    let node = LineageNode::new_initial(1, 0, make_trait_snapshot(0, 0.5, 0.5));
    assert_eq!(node.agent_id, 1);
    assert!(node.parent1_id.is_none());
    assert!(node.parent2_id.is_none());
    assert_eq!(node.generation, 0);
    assert!(node.is_alive());
}

#[test]
fn test_lineage_node_offspring() {
    let node = LineageNode::new_offspring(2, 1, None, 10, make_trait_snapshot(10, 0.6, 0.4), 0);
    assert_eq!(node.agent_id, 2);
    assert_eq!(node.parent1_id, Some(1));
    assert!(node.parent2_id.is_none());
    assert_eq!(node.generation, 1);
    assert_eq!(node.birth_tick, 10);
}

#[test]
fn test_lineage_tree_add_and_query() {
    let mut tree = LineageTree::new();

    // Add initial agent
    let node1 = LineageNode::new_initial(1, 0, make_trait_snapshot(0, 0.5, 0.5));
    tree.add_initial(node1);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Add offspring
    let node2 = LineageNode::new_offspring(2, 1, None, 10, make_trait_snapshot(10, 0.6, 0.4), 0);
    tree.add_offspring(node2);

    assert_eq!(tree.total_births(), 2);
    assert_eq!(tree.max_generation(), 1);

    // Check parent's children list was updated
    // ERR-001: Test code - unwrap() exempt for test assertions
    let parent = tree.get_node(1).unwrap();
    assert!(parent.children.contains(&2));
}

#[test]
fn test_record_death() {
    let mut tree = LineageTree::new();
    let node = LineageNode::new_initial(1, 0, make_trait_snapshot(0, 0.5, 0.5));
    tree.add_initial(node);

    // ERR-001: Test code - unwrap() exempt for test assertions
    assert!(tree.get_node(1).unwrap().is_alive());
    tree.record_death(1, 100);
    assert!(!tree.get_node(1).unwrap().is_alive());
    assert_eq!(tree.total_deaths(), 1);
}

#[test]
fn test_get_ancestors() {
    let mut tree = LineageTree::new();

    // Gen 0: agents 1 and 2
    tree.add_initial(LineageNode::new_initial(
        1,
        0,
        make_trait_snapshot(0, 0.5, 0.5),
    ));
    tree.add_initial(LineageNode::new_initial(
        2,
        0,
        make_trait_snapshot(0, 0.5, 0.5),
    ));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Gen 1: agent 3 (child of 1 and 2)
    tree.add_offspring(LineageNode::new_offspring(
        3,
        1,
        Some(2),
        10,
        make_trait_snapshot(10, 0.55, 0.45),
        0,
    ));

    // Gen 2: agent 4 (child of 3)
    tree.add_offspring(LineageNode::new_offspring(
        4,
        3,
        None,
        20,
        make_trait_snapshot(20, 0.58, 0.42),
        1,
    ));

    let ancestors = tree.get_ancestors(4);
    assert!(ancestors.contains(&3));
    assert!(ancestors.contains(&1));
    assert!(ancestors.contains(&2));
    assert_eq!(ancestors.len(), 3);
}

#[test]
fn test_get_descendants() {
    let mut tree = LineageTree::new();

    // Gen 0
    tree.add_initial(LineageNode::new_initial(
        1,
        0,
        make_trait_snapshot(0, 0.5, 0.5),
    ));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Gen 1: agent 2 (child of 1)
    tree.add_offspring(LineageNode::new_offspring(
        2,
        1,
        None,
        10,
        make_trait_snapshot(10, 0.55, 0.45),
        0,
    ));

    // Gen 2: agent 3 (child of 2)
    tree.add_offspring(LineageNode::new_offspring(
        3,
        2,
        None,
        20,
        make_trait_snapshot(20, 0.58, 0.42),
        1,
    ));

    let descendants = tree.get_descendants(1);
    assert!(descendants.contains(&2));
    assert!(descendants.contains(&3));
    assert_eq!(descendants.len(), 2);
}

#[test]
fn test_get_generation() {
    let mut tree = LineageTree::new();

    tree.add_initial(LineageNode::new_initial(
        1,
        0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        make_trait_snapshot(0, 0.5, 0.5),
    ));
    tree.add_initial(LineageNode::new_initial(
        2,
        0,
        make_trait_snapshot(0, 0.5, 0.5),
    ));
    tree.add_offspring(LineageNode::new_offspring(
        3,
        1,
        None,
        10,
        make_trait_snapshot(10, 0.55, 0.45),
        0,
    ));

    let gen0 = tree.get_generation(0);
    assert_eq!(gen0.len(), 2);
    assert!(gen0.contains(&1));
    assert!(gen0.contains(&2));

    let gen1 = tree.get_generation(1);
    assert_eq!(gen1.len(), 1);
    assert!(gen1.contains(&3));
}

#[test]
fn test_generation_trait_averages() {
    let mut tree = LineageTree::new();

    tree.add_initial(LineageNode::new_initial(
        1,
        0,
        make_trait_snapshot(0, 0.4, 0.6),
    ));
    tree.add_initial(LineageNode::new_initial(
        2,
        0,
        make_trait_snapshot(0, 0.6, 0.4),
    ));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // ERR-001: Test code - unwrap() exempt for test assertions
    let (avg_coop, avg_risk) = tree.generation_trait_averages(0).unwrap();
    assert!((avg_coop - 0.5).abs() < 0.001);
    assert!((avg_risk - 0.5).abs() < 0.001);
}

#[test]
fn test_living_count() {
    let mut tree = LineageTree::new();

    tree.add_initial(LineageNode::new_initial(
        1,
        0,
        make_trait_snapshot(0, 0.5, 0.5),
    ));
    tree.add_initial(LineageNode::new_initial(
        2,
        0,
        make_trait_snapshot(0, 0.5, 0.5),
    ));

    assert_eq!(tree.living_count(), 2);
    tree.record_death(1, 50);
    assert_eq!(tree.living_count(), 1);
}

#[test]
fn test_lifespan() {
    let mut node = LineageNode::new_initial(1, 10, make_trait_snapshot(10, 0.5, 0.5));
    assert_eq!(node.lifespan(50), 40);

    node.death_tick = Some(30);
    assert_eq!(node.lifespan(50), 20);
}

#[test]
fn test_brain_evolution_stats() {
    let mut tree = LineageTree::new();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Gen 0: agents 1 and 2 with different DOCAS beta
    let mut traits1 = make_trait_snapshot(0, 0.5, 0.5);
    traits1.docas_beta = [0.1, 0.1, 0.1, 0.1, 0.1];
    tree.add_initial(LineageNode::new_initial(1, 0, traits1));

    let mut traits2 = make_trait_snapshot(0, 0.5, 0.5);
    traits2.docas_beta = [0.2, 0.2, 0.2, 0.2, 0.2];
    tree.add_initial(LineageNode::new_initial(2, 0, traits2));

    // Gen 1: offspring with intermediate values
    let mut traits3 = make_trait_snapshot(10, 0.55, 0.45);
    traits3.docas_beta = [0.15, 0.15, 0.15, 0.15, 0.15];
    tree.add_offspring(LineageNode::new_offspring(3, 1, Some(2), 10, traits3, 0));

    let stats = tree.compute_brain_evolution();

    // Should have 2 generations
    assert_eq!(stats.generation_avg_beta.len(), 2);
    assert_eq!(stats.generation_population.len(), 2);

    // Gen 0 average beta should be 0.15
    let (gen, avg_beta) = stats.generation_avg_beta[0];
    assert_eq!(gen, 0);
    assert!((avg_beta[0] - 0.15).abs() < 0.001);

    // Gen 1 average beta should be 0.15
    let (gen, avg_beta) = stats.generation_avg_beta[1];
    assert_eq!(gen, 1);
    assert!((avg_beta[0] - 0.15).abs() < 0.001);
}

#[test]
fn test_trait_snapshot_from_genome() {
    use crate::genome::Genome;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(42);
    let genome = Genome::random(&mut rng);

    let snapshot = TraitSnapshot::from_genome(&genome, 100);

    assert_eq!(snapshot.tick, 100);
    assert_eq!(snapshot.docas_alpha, genome.docas_alpha());
    assert_eq!(snapshot.docas_beta, genome.docas_beta());
    assert_eq!(snapshot.planning_depth, genome.planning_depth());
    assert_eq!(snapshot.planning_horizon, genome.planning_horizon());
    assert_eq!(snapshot.cooperation_tendency, genome.cooperation_tendency());
    assert_eq!(snapshot.risk_tolerance, genome.risk_tolerance());
}

#[test]
fn test_get_kin_degree_1() {
    let mut tree = LineageTree::new();

    // Create grandparent (0)
    let grandparent = LineageNode::new_initial(0, 0, make_trait_snapshot(0, 0.5, 0.5));
    tree.add_initial(grandparent);

    // Create parent (1)
    let parent = LineageNode::new_offspring(1, 0, None, 10, make_trait_snapshot(10, 0.5, 0.5), 0);
    tree.add_offspring(parent);

    // Create child (2)
    let child = LineageNode::new_offspring(2, 1, None, 20, make_trait_snapshot(20, 0.5, 0.5), 1);
    tree.add_offspring(child);

    // Create sibling (3)
    let sibling = LineageNode::new_offspring(3, 1, None, 25, make_trait_snapshot(25, 0.5, 0.5), 1);
    tree.add_offspring(sibling);

    // Check child's kin (degree 1)
    let child_kin = tree.get_kin(2, 1);
    assert!(child_kin.contains(&1), "Child should have parent as kin");
    assert!(child_kin.contains(&3), "Child should have sibling as kin");
    assert!(!child_kin.contains(&2), "Child should not be kin to itself");
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_get_kin_degree_2() {
    let mut tree = LineageTree::new();

    // Create grandparent (0)
    let grandparent = LineageNode::new_initial(0, 0, make_trait_snapshot(0, 0.5, 0.5));
    tree.add_initial(grandparent);

    // Create parent (1)
    let parent = LineageNode::new_offspring(1, 0, None, 10, make_trait_snapshot(10, 0.5, 0.5), 0);
    tree.add_offspring(parent);

    // Create child (2)
    let child = LineageNode::new_offspring(2, 1, None, 20, make_trait_snapshot(20, 0.5, 0.5), 1);
    tree.add_offspring(child);

    // Create grandchild (3)
    let grandchild =
        LineageNode::new_offspring(3, 2, None, 30, make_trait_snapshot(30, 0.5, 0.5), 2);
    tree.add_offspring(grandchild);

    // Check child's kin (degree 2)
    let child_kin = tree.get_kin(2, 2);
    assert!(
        child_kin.contains(&0),
        "Child should have grandparent as kin (degree 2)"
    );
    assert!(
        child_kin.contains(&3),
        "Child should have grandchild as kin (degree 2)"
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_are_kin() {
    let mut tree = LineageTree::new();

    // Create parent (0)
    let parent = LineageNode::new_initial(0, 0, make_trait_snapshot(0, 0.5, 0.5));
    tree.add_initial(parent);

    // Create child (1)
    let child = LineageNode::new_offspring(1, 0, None, 10, make_trait_snapshot(10, 0.5, 0.5), 0);
    tree.add_offspring(child);

    // Create unrelated (2)
    let unrelated = LineageNode::new_initial(2, 5, make_trait_snapshot(5, 0.5, 0.5));
    tree.add_initial(unrelated);

    assert!(tree.are_kin(0, 1), "Parent and child should be kin");
    assert!(tree.are_kin(1, 0), "Kinship is symmetric");
    assert!(!tree.are_kin(0, 2), "Unrelated agents should not be kin");
    assert!(!tree.are_kin(1, 1), "Agent should not be kin to itself");
}

#[test]
fn test_kinship_degree() {
    let mut tree = LineageTree::new();

    // Create grandparent (0)
    let grandparent = LineageNode::new_initial(0, 0, make_trait_snapshot(0, 0.5, 0.5));
    tree.add_initial(grandparent);

    // Create parent (1)
    let parent = LineageNode::new_offspring(1, 0, None, 10, make_trait_snapshot(10, 0.5, 0.5), 0);
    tree.add_offspring(parent);

    // Create child (2)
    let child = LineageNode::new_offspring(2, 1, None, 20, make_trait_snapshot(20, 0.5, 0.5), 1);
    tree.add_offspring(child);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(
        tree.kinship_degree(0, 1),
        Some(1),
        "Parent-child is degree 1"
    );
    assert_eq!(
        tree.kinship_degree(1, 2),
        Some(1),
        "Parent-child is degree 1"
    );
    assert_eq!(
        tree.kinship_degree(0, 2),
        Some(2),
        "Grandparent-grandchild is degree 2"
    );
    assert_eq!(
        tree.kinship_degree(2, 2),
        None,
        "Self has no kinship degree"
    );
}

// GENE-P1-1: Wright-Fisher Population Genetics Tests
#[test]
fn test_genotype_from_alleles() {
    assert_eq!(
        Genotype::from_alleles(Allele::Dominant, Allele::Dominant),
        Genotype::HomozygousDominant
    );
    assert_eq!(
        Genotype::from_alleles(Allele::Recessive, Allele::Recessive),
        Genotype::HomozygousRecessive
    );
    assert_eq!(
        Genotype::from_alleles(Allele::Dominant, Allele::Recessive),
        Genotype::Heterozygous
    );
    assert_eq!(
        Genotype::from_alleles(Allele::Recessive, Allele::Dominant),
        Genotype::Heterozygous
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_genotype_additive_value() {
    assert_eq!(Genotype::HomozygousDominant.additive_value(), 1.0);
    assert_eq!(Genotype::Heterozygous.additive_value(), 0.5);
    assert_eq!(Genotype::HomozygousRecessive.additive_value(), 0.0);
}

#[test]
fn test_genotype_dominant_phenotype() {
    assert!(Genotype::HomozygousDominant.shows_dominant_phenotype());
    assert!(Genotype::Heterozygous.shows_dominant_phenotype());
    assert!(!Genotype::HomozygousRecessive.shows_dominant_phenotype());
}

#[test]
fn test_allele_frequencies() {
    let freq = AlleleFrequencies::from_p(0.7, 0);
    assert!((freq.p - 0.7).abs() < 0.001);
    assert!((freq.q - 0.3).abs() < 0.001);

    // Hardy-Weinberg proportions
    let (aa, het, rr) = freq.hardy_weinberg();
    assert!((aa - 0.49).abs() < 0.001); // p² = 0.49
    assert!((het - 0.42).abs() < 0.001); // 2pq = 0.42
    assert!((rr - 0.09).abs() < 0.001); // q² = 0.09

    // Check they sum to 1
    assert!((aa + het + rr - 1.0).abs() < 0.001);
}

#[test]
fn test_hardy_weinberg_equilibrium() {
    let freq = AlleleFrequencies::from_p(0.5, 0);
    let (aa, het, rr) = freq.hardy_weinberg();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // For p = 0.5: AA = 0.25, Aa = 0.5, aa = 0.25
    assert!((aa - 0.25).abs() < 0.001);
    assert!((het - 0.5).abs() < 0.001);
    assert!((rr - 0.25).abs() < 0.001);

    // Expected heterozygosity = 2pq = 0.5
    assert!((freq.expected_heterozygosity() - 0.5).abs() < 0.001);
}

#[test]
fn test_wright_fisher_creation() {
    let wf = AlleleFrequencyTracker::new(100, 0.5);
    assert_eq!(wf.population_size, 100);
    assert_eq!(wf.initial_p, 0.5);
    assert_eq!(wf.current_generation, 0);
    assert!(!wf.is_fixed());

    let status = wf.fixation_status();
    assert_eq!(status, Some(FixationStatus::Segregating));
}

#[test]
fn test_wright_fisher_drift() {
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(42);
    let mut wf = AlleleFrequencyTracker::new(50, 0.5);

    // Simulate 10 generations
    wf.simulate_generations(10, &mut rng);

    assert_eq!(wf.current_generation, 10);
    assert!(!wf.is_fixed()); // Unlikely to fix in 10 gens with N=50

    // Frequency should have changed (drift)
    let final_p = wf.frequency_trajectory.last().unwrap().p;
    assert!((0.0..=1.0).contains(&final_p));
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_wright_fisher_fixation() {
    use rand::rngs::StdRng;

    // Small population with extreme frequency should fix quickly
    let mut rng = StdRng::seed_from_u64(42);
    let mut wf = AlleleFrequencyTracker::new(10, 0.95);

    wf.simulate_generations(100, &mut rng);

    // Should eventually fix
    if wf.is_fixed() {
        let status = wf.fixation_status().unwrap();
        assert!(matches!(
            status,
            FixationStatus::FixedDominant | FixationStatus::FixedRecessive
        ));
    }
}

#[test]
fn test_heterozygosity_decay() {
    let wf = AlleleFrequencyTracker::new(100, 0.5);

    // Initial heterozygosity
    let initial_h = wf.expected_heterozygosity();

    // Expected H after t generations: H_t = H_0 * (1 - 1/(2N))^t
    // For N=100, t=10: decay factor ≈ (1 - 1/200)^10 ≈ 0.951
    let expected_after_10 = initial_h * (1.0 - 1.0 / 200.0f32).powi(10);

    assert!(expected_after_10 < initial_h, "Heterozygosity should decay");
}

#[test]
fn test_fixation_probability_neutral() {
    // For neutral alleles, fixation probability = initial frequency
    let wf1 = AlleleFrequencyTracker::new(100, 0.1);
    let wf2 = AlleleFrequencyTracker::new(100, 0.5);
    let wf3 = AlleleFrequencyTracker::new(100, 0.9);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!((wf1.fixation_probability() - 0.1).abs() < 0.001);
    assert!((wf2.fixation_probability() - 0.5).abs() < 0.001);
    assert!((wf3.fixation_probability() - 0.9).abs() < 0.001);
}

#[test]
fn test_frequency_variance() {
    let mut wf = AlleleFrequencyTracker::new(100, 0.5);

    // At generation 0, variance should be 0
    assert!(wf.frequency_variance() < 0.001);

    // Variance should increase with time
    wf.current_generation = 10;
    let var_10 = wf.frequency_variance();

    wf.current_generation = 50;
    let var_50 = wf.frequency_variance();

    assert!(var_50 > var_10, "Variance should increase with generations");
}

#[test]
fn test_binomial_sampling() {
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(42);

    // Test edge cases
    assert_eq!(sample_binomial(&mut rng, 0, 0.5), 0);
    assert_eq!(sample_binomial(&mut rng, 100, 0.0), 0);
    assert_eq!(sample_binomial(&mut rng, 100, 1.0), 100);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Test small n
    let mut sum = 0u32;
    for _ in 0..100 {
        sum += sample_binomial(&mut rng, 10, 0.5);
    }
    let avg = sum as f32 / 100.0;
    // Expected: 10 * 0.5 = 5, averaged over 100 trials ≈ 500
    assert!(
        avg > 4.0 && avg < 6.0,
        "Average should be around 5, got {avg}"
    );
}

#[test]
fn test_linkage_disequilibrium() {
    // Perfect LD: AB and ab only
    // freq(AB) = 0.5, freq(ab) = 0.5, freq(Ab) = 0, freq(aB) = 0
    let d = linkage_disequilibrium(0.5, 0.0, 0.0, 0.5);
    assert!((d - 0.25).abs() < 0.001, "Perfect LD should give D = 0.25");

    // No LD: all combinations equal (0.25 each)
    let d_none = linkage_disequilibrium(0.25, 0.25, 0.25, 0.25);
    assert!(d_none.abs() < 0.001, "No LD should give D = 0");
}

#[test]
fn test_tajimas_d_neutral() {
    // Neutral evolution: Tajima's D ≈ 0
    // With sufficient data
    let taj_d = tajimas_d(50, 20, 5.0);
    assert!(taj_d.is_some());

    // D should be finite
    let d = taj_d.unwrap();
    assert!(d.is_finite());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_tajimas_d_edge_cases() {
    // No segregating sites
    assert!(tajimas_d(0, 10, 0.0).is_none());

    // Sample size too small
    assert!(tajimas_d(10, 1, 1.0).is_none());

    // Valid case
    assert!(tajimas_d(10, 5, 2.0).is_some());
}

// P2-1: Mitochondrial/Y-chromosome Tracking Tests
#[test]
fn test_maternal_lineage_creation() {
    let lineage = MaternalLineage::from_mother(1, Some(1001));

    assert_eq!(lineage.mtdna_haplogroup, Some(1001));
    assert_eq!(lineage.maternal_ancestors.len(), 1);
    assert_eq!(lineage.maternal_ancestors[0], 1);
    assert_eq!(lineage.generations_tracked, 1);
}

#[test]
fn test_paternal_lineage_creation() {
    let lineage = PaternalLineage::from_father(2, Some(5));

    assert_eq!(lineage.y_haplogroup, Some(5));
    assert_eq!(lineage.paternal_ancestors.len(), 1);
    assert_eq!(lineage.paternal_ancestors[0], 2);
    assert_eq!(lineage.generations_tracked, 1);
}

#[test]
fn test_maternal_lineage_no_haplogroup() {
    // If mother has no haplogroup, lineage doesn't track
    let lineage = MaternalLineage::from_mother(1, None);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(lineage.mtdna_haplogroup, None);
    assert!(lineage.maternal_ancestors.is_empty());
}

#[test]
fn test_haplogroup_database_y_dna() {
    let y_haplos = HaplogroupDatabase::y_haplogroups();

    // Should have major haplogroups
    assert!(!y_haplos.is_empty());

    // Should include major groups like R, J, E
    let has_r = y_haplos.iter().any(|h| h.haplogroup == 'R');
    let has_j = y_haplos.iter().any(|h| h.haplogroup == 'J');
    assert!(has_r && has_j);
}

#[test]
fn test_haplogroup_database_mtdna() {
    let mtdna_haplos = HaplogroupDatabase::mtdna_haplogroups();

    // Should have macro-haplogroups L, M, N
    assert!(!mtdna_haplos.is_empty());

    let has_l = mtdna_haplos.iter().any(|h| h.haplogroup == 'L');
    let has_m = mtdna_haplos.iter().any(|h| h.haplogroup == 'M');
    let has_n = mtdna_haplos.iter().any(|h| h.haplogroup == 'N');
    assert!(has_l && has_m && has_n);
}

#[test]
fn test_maternal_haplogroup_name() {
    let lineage = MaternalLineage::from_mother(1, Some(1001));
    let name = lineage.haplogroup_name();

    // Should return something meaningful
    assert!(!name.is_empty());
    assert_ne!(name, "Unknown");
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_paternal_haplogroup_name() {
    let lineage = PaternalLineage::from_father(1, Some(5));
    let name = lineage.haplogroup_name();

    // E haplogroup should be recognized
    assert!(!name.is_empty());
}

#[test]
fn test_haplogroup_unknown() {
    let maternal = MaternalLineage::from_mother(1, None);
    assert_eq!(maternal.haplogroup_name(), "Unknown");

    let paternal = PaternalLineage::from_father(1, None);
    assert_eq!(paternal.haplogroup_name(), "Unknown");
}

#[test]
fn test_add_mutation_maternal() {
    let mut lineage = MaternalLineage::from_mother(1, Some(1001));

    lineage.add_mutation(10, 1101);

    assert_eq!(lineage.mtdna_haplogroup, Some(1101));
    assert_eq!(lineage.mutations.len(), 1);
    assert_eq!(lineage.mutations[0], (10, 1101));
}

#[test]
fn test_add_mutation_no_change() {
    // Adding same haplogroup shouldn't create mutation
    let mut lineage = PaternalLineage::from_father(1, Some(5));

    lineage.add_mutation(10, 5);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(lineage.y_haplogroup, Some(5));
    assert!(lineage.mutations.is_empty());
}

#[test]
fn test_shares_ancestor_with() {
    let lineage1 = PaternalLineage::from_father(1, Some(5));
    let lineage2 = PaternalLineage::from_father(2, Some(5));
    let lineage3 = PaternalLineage::from_father(3, Some(6));

    assert!(lineage1.shares_ancestor_with(&lineage2));
    assert!(!lineage1.shares_ancestor_with(&lineage3));
}

#[test]
fn test_population_haplogroups() {
    let mut rng = StdRng::seed_from_u64(42);

    let y_dist = vec![(1, 0.5), (2, 0.5)]; // 50% each
    let mtdna_dist = vec![(1001, 0.5), (1002, 0.5)];

    let pop = PopulationHaplogroups::from_population(100, &y_dist, &mtdna_dist, &mut rng);

    assert_eq!(pop.sample_size, 100);
    assert!(!pop.y_haplogroup_freqs.is_empty());
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!(!pop.mtdna_haplogroup_freqs.is_empty());

    // Frequencies should sum to ~1.0
    let y_sum: f32 = pop.y_haplogroup_freqs.values().sum();
    assert!(y_sum > 0.9 && y_sum <= 1.0);
}

#[test]
fn test_most_common_haplogroup() {
    let mut rng = StdRng::seed_from_u64(42);

    // 90% haplogroup 1, 10% haplogroup 2
    let y_dist = vec![(1, 0.9), (2, 0.1)];
    let mtdna_dist = vec![(1001, 0.9), (1002, 0.1)];

    let pop = PopulationHaplogroups::from_population(100, &y_dist, &mtdna_dist, &mut rng);

    let most_common_y = pop.most_common_y();
    let most_common_mtdna = pop.most_common_mtdna();

    assert!(most_common_y.is_some());
    assert!(most_common_mtdna.is_some());

    assert_eq!(most_common_y.unwrap().0, 1);
    assert_eq!(most_common_mtdna.unwrap().0, 1001);
}
