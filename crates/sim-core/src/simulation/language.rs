use crate::agent::Agent;
use crate::environment::SpatialHash;
use crate::language::{process_message_reception, Lexicon, Message};
use crate::types::{Position, Tick};
use rand::rngs::StdRng;
use rand::Rng;
use tracing::debug;

// TAG: surface=simulation owner=research-team rule=SIM-001
impl super::Simulation {
    /// Process Stage 4 (Language) mechanisms
    /// Per §4379-4385: Semantic drift, colexification, cross-cultural alignment
    /// Per §8.2.1 lines 4723-4746: Language toggle contract
    pub(super) fn process_stage4_language(&mut self) {
        // === Initialize lexicons for agents without them ===
        // Per §6773: MVP = static lexicons with colexification from CLICS³ v3.0
        let agents_needing_lexicons: Vec<usize> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive && a.lexicon.is_empty())
            .map(|(i, _)| i)
            .collect();

        let current_tick = self.tick;
        for idx in agents_needing_lexicons {
            let rng = self.rng();
            let lexicon = Lexicon::initialize_base(rng, current_tick);
            self.agents[idx].lexicon = lexicon;
        }

        // === Semantic communication between nearby agents ===
        // Per §8.2.1 lines 4726-4740: Process message reception
        // Alignment factor dynamically scaled by language_drift_rate_factor dial
        let alignment_factor =
            (0.10 * self.config.toggles.dials.language_drift_rate_factor).clamp(0.01, 1.0);
        let coherence_threshold = self.config.toggles.dials.language_coherence_threshold;
        const COMMUNICATION_RADIUS: f32 = 3.0;

        // Find pairs of nearby living agents for communication
        // Per §23.8.2 GAP-P1-05: Use spatial hash for O(n) neighbor lookup instead of O(n²)
        let living_agents: Vec<(usize, Position)> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive && !a.lexicon.is_empty())
            .map(|(i, a)| (i, a.position))
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Build spatial hash for efficient neighbor lookup
        // Cell size = communication radius for optimal single-cell lookup
        let mut spatial_hash = SpatialHash::new(COMMUNICATION_RADIUS);
        spatial_hash.rebuild(&living_agents);

        // Find all pairs within communication radius using spatial hash
        // This is O(n * k) where k is average neighbors per agent, instead of O(n²)
        let communication_pairs = spatial_hash.find_pairs_within_radius(COMMUNICATION_RADIUS);

        // Pre-generate random values for message sizes to avoid borrow issues
        let message_sizes: Vec<(usize, usize)> = {
            let rng = self.rng();
            communication_pairs
                .iter()
                .map(|_| {
                    let size_a = (rng.gen::<f32>() * 3.0) as usize + 2;
                    let size_b = (rng.gen::<f32>() * 3.0) as usize + 2;
                    (size_a, size_b)
                })
                .collect()
        };

        // Process each communication pair
        // To maintain determinism, we process in index order
        for (pair_idx, (idx_a, idx_b)) in communication_pairs.iter().enumerate() {
            let (num_concepts_a, num_concepts_b) = message_sizes[pair_idx];

            // Agent A sends message to Agent B
            // Clone sender's lexicon and build message
            let sender_id_a = self.agents[*idx_a].id;
            let sender_lexicon_a = self.agents[*idx_a].lexicon.clone();
            let concept_ids_a: Vec<u64> = sender_lexicon_a.concepts.keys().copied().collect();
            let content_a: Vec<u64> = concept_ids_a
                .iter()
                .take(num_concepts_a.min(concept_ids_a.len()))
                .copied()
                .collect();
            let message_a = Message::new(sender_id_a, content_a, current_tick);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Receiver (B) processes message from A if lexical coherence threshold is met
            let overlap_ab = sender_lexicon_a.overlap(&self.agents[*idx_b].lexicon);
            if overlap_ab >= coherence_threshold * 0.25 {
                process_message_reception(
                    &mut self.agents[*idx_b].lexicon,
                    &sender_lexicon_a,
                    &message_a,
                    alignment_factor,
                );
            }

            // Agent B sends message to Agent A (bidirectional communication)
            let sender_id_b = self.agents[*idx_b].id;
            let sender_lexicon_b = self.agents[*idx_b].lexicon.clone();
            let concept_ids_b: Vec<u64> = sender_lexicon_b.concepts.keys().copied().collect();
            let content_b: Vec<u64> = concept_ids_b
                .iter()
                .take(num_concepts_b.min(concept_ids_b.len()))
                .copied()
                .collect();
            let message_b = Message::new(sender_id_b, content_b, current_tick);

            // Receiver (A) processes message from B if lexical coherence threshold is met
            let overlap_ba = sender_lexicon_b.overlap(&self.agents[*idx_a].lexicon);
            if overlap_ba >= coherence_threshold * 0.25 {
                process_message_reception(
                    &mut self.agents[*idx_a].lexicon,
                    &sender_lexicon_b,
                    &message_b,
                    alignment_factor,
                );
            }
        }
    }
    /// Process Stage 4: Music as universal trust indicator
    /// Per music.md: Music-DOCAS bridge, genre synchronization, trust building
    /// Includes cadence-based group bonding for cross-language connections
    pub(super) fn process_stage4_music(&mut self) {
        use crate::music::MusicState;
        use rand::SeedableRng;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let current_tick = self.tick;

        let (eng_min, eng_max) = self.config.toggles.dials.music_engagement_range;
        let eng_spread = (eng_max - eng_min).max(0.01);
        let (sync_min, sync_max) = self.config.toggles.dials.music_sync_skill_range;
        let sync_spread = (sync_max - sync_min).max(0.01);

        for agent in &mut self.agents {
            if agent.alive && agent.music_state.is_none() {
                let mut rng = StdRng::seed_from_u64(
                    self.config.seed.wrapping_add(agent.id).wrapping_add(12345),
                );
                let genome_traits: Vec<f32> = agent
                    .genome
                    .alleles
                    .iter()
                    .map(|[a, b]| (a + b) / 2.0)
                    .collect();
                let mut state = MusicState::from_genome(&genome_traits, &mut rng);
                state.engagement = (eng_min + rng.gen::<f32>() * eng_spread).clamp(0.01, 1.0);
                state.sync_skill = (sync_min + rng.gen::<f32>() * sync_spread).clamp(0.01, 1.0);
                agent.music_state = Some(state);
            }
        }

        let (total_preference, music_agent_count) = self.aggregate_music_preferences();

        if music_agent_count > 0 {
            self.update_agent_music_states(current_tick, &total_preference, music_agent_count);
        }

        self.process_music_synchronization();
    }

    pub(super) fn aggregate_music_preferences(&self) -> ([f32; 5], usize) {
        let mut total_preference = [0.0f32; 5];
        let mut music_agent_count = 0;

        for agent in &self.agents {
            if !agent.alive {
                continue;
            }
            if let Some(ref music) = agent.music_state {
                for (i, &pref) in music.genre_preferences.iter().enumerate() {
                    total_preference[i] += pref;
                }
                music_agent_count += 1;
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        (total_preference, music_agent_count)
    }

    pub(super) fn update_agent_music_states(
        &mut self,
        current_tick: Tick,
        total_preference: &[f32; 5],
        music_agent_count: usize,
    ) {
        use crate::music::MusicEvent;

        let population = music_agent_count as f32;
        let _mean_preference: [f32; 5] = [
            total_preference[0] / population,
            total_preference[1] / population,
            total_preference[2] / population,
            total_preference[3] / population,
            total_preference[4] / population,
        ];

        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }
            let Some(ref mut music) = agent.music_state else {
                continue;
            };

            let genre = music.dominant_genre();
            let genre_idx = Self::genre_to_index(genre);
            let event = MusicEvent {
                tick: current_tick,
                genre: genre_idx,
                spectral_character: music.spectral_preference,
                was_synchronized: false,
                participant_count: 1,
            };
            music.record_listening(event, current_tick);
            music.record_docas_listening(current_tick, agent.docas.levels);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let trust_boost = music.engagement * 0.05;
            if trust_boost > 0.0 {
                agent.docas.levels[3] = (agent.docas.levels[3] + trust_boost * 0.02).min(1.0);
            }

            let tau = self.config.toggles.dials.music_metric_hierarchy_tau;
            let tdf = crate::music::TdfMusicVector::with_tau(tau);
            let modulation = genre.docas_modulation_tdf(&tdf);
            for (i, &mod_val) in modulation.iter().enumerate() {
                agent.docas.levels[i] = (agent.docas.levels[i] + mod_val * 0.01).clamp(0.0, 1.0);
            }

            let openness = agent
                .genome
                .alleles
                .first()
                .map_or(0.5, |[a, b]| (a + b) / 2.0);
            let profile = genre.personality_profile();
            if (profile[0] - openness).abs() < 0.3 {
                agent.docas.levels[4] = (agent.docas.levels[4] + 0.01).min(1.0);
            }
        }
    }

    pub(super) const fn genre_to_index(genre: crate::music::MusicGenre) -> usize {
        match genre {
            crate::music::MusicGenre::Classical => 0,
            crate::music::MusicGenre::Dance => 1,
            crate::music::MusicGenre::Melancholic => 2,
            crate::music::MusicGenre::Traditional => 3,
            crate::music::MusicGenre::Ambient => 4,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Process music synchronization between nearby agents
    /// Triggers `SyncPerformance` when cadence similarity > 0.7
    /// Enables cross-language bonding through shared rhythm
    pub(super) fn process_music_synchronization(&mut self) {
        use crate::encounter::PERCEPTION_RADIUS;

        const CADENCE_SIMILARITY_THRESHOLD: f32 = 0.7;
        const MIN_SYNC_PARTICIPANTS: usize = 2;
        const SYNC_DURATION: Tick = 10;

        let agents_with_music: Vec<(usize, u64, f32, f32)> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .filter_map(|(idx, a)| {
                a.music_state
                    .as_ref()
                    .map(|_m| (idx, a.id, a.position.x, a.position.y))
            })
            .collect();

        if agents_with_music.len() < MIN_SYNC_PARTICIPANTS {
            return;
        }

        let synced_groups = Self::find_synced_groups(
            &self.agents,
            &agents_with_music,
            PERCEPTION_RADIUS,
            CADENCE_SIMILARITY_THRESHOLD,
            MIN_SYNC_PARTICIPANTS,
        );

        for group in synced_groups {
            self.apply_sync_performance(&group, SYNC_DURATION);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub(super) fn find_synced_groups(
        agents: &[Agent],
        agents_with_music: &[(usize, u64, f32, f32)],
        perception_radius: f32,
        cadence_threshold: f32,
        min_participants: usize,
    ) -> Vec<Vec<usize>> {
        let mut synced_groups: Vec<Vec<usize>> = Vec::new();
        let mut processed: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (i, (idx1, _, x1, y1)) in agents_with_music.iter().enumerate() {
            if processed.contains(idx1) {
                continue;
            }

            let mut group = vec![*idx1];
            processed.insert(*idx1);

            let Some(music1) = agents[*idx1].music_state.as_ref() else {
                continue;
            };
            let radius_sq = perception_radius.powi(2);

            for (j, (idx2, _, x2, y2)) in agents_with_music.iter().enumerate() {
                if i == j || processed.contains(idx2) {
                    continue;
                }

                let dist_sq = (x1 - x2).powi(2) + (y1 - y2).powi(2);
                if dist_sq > radius_sq {
                    continue;
                }

                let Some(music2) = agents[*idx2].music_state.as_ref() else {
                    continue;
                };
                if music1.cadence_similarity(music2) >= cadence_threshold {
                    group.push(*idx2);
                    processed.insert(*idx2);
                }
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            if group.len() >= min_participants {
                synced_groups.push(group);
            }
        }

        synced_groups
    }

    pub(super) fn apply_sync_performance(&mut self, group: &[usize], sync_duration: Tick) {
        use crate::music::MusicGenre;

        let mut genre_counts = [0usize; 5];
        for &idx in group {
            if let Some(ref music) = self.agents[idx].music_state {
                let g_idx = Self::genre_to_index(music.dominant_genre());
                genre_counts[g_idx] += 1;
            }
        }

        // Deterministic genre selection with stable tie-breaking (prefer lowest index on ties)
        let dominant_genre = genre_counts
            .iter()
            .enumerate()
            .max_by(|(idx1, count1), (idx2, count2)| {
                count1.cmp(count2).then_with(|| idx2.cmp(idx1))
            })
            .and_then(|(idx, &count)| if count > 0 { Some(idx) } else { None })
            .map_or(MusicGenre::Ambient, |g_idx| match g_idx {
                0 => MusicGenre::Classical,
                1 => MusicGenre::Dance,
                2 => MusicGenre::Melancholic,
                3 => MusicGenre::Traditional,
                _ => MusicGenre::Ambient,
            });

        let participant_ids: Vec<u64> = group.iter().map(|&idx| self.agents[idx].id).collect();

        let avg_sync_skill: f32 = group
            .iter()
            .filter_map(|&idx| self.agents[idx].music_state.as_ref())
            .map(|m| m.sync_skill)
            .sum::<f32>()
            / group.len() as f32;

        let performance = crate::music::SyncPerformance {
            participants: participant_ids.clone(),
            genre: dominant_genre,
            duration: sync_duration,
            start_tick: self.tick,
            sync_quality: avg_sync_skill.min(1.0),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let trust_boost = performance.compute_trust_boost();

        for &idx in group {
            let agent = &mut self.agents[idx];
            agent.docas.levels[1] = (agent.docas.levels[1] + trust_boost * 0.5).min(1.0);
            agent.docas.levels[4] = (agent.docas.levels[4] + trust_boost * 0.3).min(1.0);
            agent.docas.levels[0] = (agent.docas.levels[0] + trust_boost * 0.2).min(1.0);

            if let Some(ref mut music) = agent.music_state {
                let event = crate::music::MusicEvent {
                    tick: self.tick,
                    genre: Self::genre_to_index(dominant_genre),
                    spectral_character: music.spectral_preference,
                    was_synchronized: true,
                    participant_count: group.len() as u32,
                };
                music.record_listening(event, self.tick);
            }
        }

        debug!(
            "[SimCore] [MusicSync] tick={} participants={} genre={:?} quality={:.2} trust_boost={:.3}",
            self.tick,
            participant_ids.len(),
            dominant_genre,
            avg_sync_skill,
            trust_boost
        );
    }
}

#[cfg(test)]
mod tests {
    // This file contains only pub(super) methods on Simulation.
    // No standalone pure functions exist for unit testing without
    // instantiating the full Simulation state.
}
