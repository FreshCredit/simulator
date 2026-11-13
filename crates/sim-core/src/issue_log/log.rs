use std::collections::HashMap;

use crate::types::{AgentId, Tick};
use serde::{Deserialize, Serialize};

use super::issue::{Issue, IssueCategory, IssueSeverity, IssueStatus};
use super::lessons::{Lesson, RecurringPattern};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Issue Log: Pain + Reflection = Progress
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct IssueLog {
    /// issues.
    pub issues: Vec<Issue>,
    /// patterns.
    pub patterns: Vec<RecurringPattern>,
    /// next issue id.
    pub next_issue_id: u64,
    /// next pattern id.
    pub next_pattern_id: u64,
    /// lessons learned.
    pub lessons_learned: Vec<Lesson>,
    /// closed count.
    pub closed_count: usize,
    /// open count.
    pub open_count: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, PartialEq, Eq)]
/// issue log error.
pub enum IssueLogError {
    /// issue not found.
    IssueNotFound(u64),
    /// already resolved.
    AlreadyResolved(u64),
    /// invalid resolution.
    InvalidResolution(String),
}

impl std::fmt::Display for IssueLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IssueNotFound(id) => write!(f, "Issue {id} not found"),
            Self::AlreadyResolved(id) => write!(f, "Issue {id} is already resolved"),
            Self::InvalidResolution(msg) => write!(f, "Invalid resolution: {msg}"),
        }
    }
}

impl std::error::Error for IssueLogError {}

impl IssueLog {
    /// Create a new empty issue log
    #[must_use]
    pub const fn new() -> Self {
        Self {
            issues: Vec::new(),
            patterns: Vec::new(),
            next_issue_id: 1,
            next_pattern_id: 1,
            lessons_learned: Vec::new(),
            closed_count: 0,
            open_count: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a new issue (Pain)
    #[allow(clippy::too_many_arguments)]
    pub fn record_issue(
        &mut self,
        reporter: AgentId,
        title: &str,
        description: &str,
        severity: IssueSeverity,
        category: IssueCategory,
        related_agents: Vec<AgentId>,
        tick: Tick,
    ) -> u64 {
        let id = self.next_issue_id;
        self.next_issue_id += 1;

        let issue = Issue::new(
            id,
            reporter,
            title,
            description,
            severity,
            category,
            related_agents,
            tick,
        );

        self.issues.push(issue);
        self.open_count += 1;

        id
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Mark issue as resolved with reflection (Progress)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn resolve_issue(
        &mut self,
        issue_id: u64,
        resolution: &str,
        lessons: Vec<String>,
        root_cause: &str,
        preventive_measures: Vec<String>,
        tick: Tick,
    ) -> Result<(), IssueLogError> {
        // Find the issue
        let issue = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or(IssueLogError::IssueNotFound(issue_id))?;

        // Check if already resolved
        if issue.is_resolved() {
            return Err(IssueLogError::AlreadyResolved(issue_id));
        }

        // Validate resolution
        if resolution.trim().is_empty() {
            return Err(IssueLogError::InvalidResolution(
                "Resolution cannot be empty".to_string(),
            ));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Update issue
        issue.status = IssueStatus::Resolved;
        issue.tick_resolved = Some(tick);
        issue.resolution = Some(resolution.to_string());
        issue.lessons_learned.clone_from(&lessons);
        issue.root_cause = Some(root_cause.to_string());
        issue.preventive_measures = preventive_measures;

        // Update counts
        self.open_count = self.open_count.saturating_sub(1);
        self.closed_count += 1;

        // Extract lessons
        for lesson_text in lessons {
            let lesson = Lesson::new(&lesson_text, issue_id, tick, issue.category);
            self.lessons_learned.push(lesson);
        }

        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Start working on an issue
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn start_progress(&mut self, issue_id: u64) -> Result<(), IssueLogError> {
        let issue = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or(IssueLogError::IssueNotFound(issue_id))?;

        if issue.status == IssueStatus::Open || issue.status == IssueStatus::Reopened {
            issue.status = IssueStatus::InProgress;
            Ok(())
        } else {
            Err(IssueLogError::InvalidResolution(format!(
                "Cannot start progress on issue with status: {}",
                issue.status
            )))
        }
    }

    /// Close a resolved issue
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn close_issue(&mut self, issue_id: u64) -> Result<(), IssueLogError> {
        let issue = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or(IssueLogError::IssueNotFound(issue_id))?;

        if issue.status == IssueStatus::Resolved {
            issue.status = IssueStatus::Closed;
            Ok(())
        } else {
            Err(IssueLogError::InvalidResolution(
                "Only resolved issues can be closed".to_string(),
            ))
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reopen a closed/resolved issue
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn reopen_issue(&mut self, issue_id: u64) -> Result<(), IssueLogError> {
        let issue = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or(IssueLogError::IssueNotFound(issue_id))?;

        if issue.is_resolved() {
            issue.status = IssueStatus::Reopened;
            issue.tick_resolved = None;
            issue.resolution = None;
            self.open_count += 1;
            self.closed_count = self.closed_count.saturating_sub(1);
            Ok(())
        } else {
            Err(IssueLogError::InvalidResolution(
                "Only resolved issues can be reopened".to_string(),
            ))
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect recurring patterns (systemic issues)
    ///
    /// Groups issues by category and identifies patterns above threshold frequency
    pub fn detect_patterns(
        &mut self,
        tick: Tick,
        frequency_threshold: usize,
    ) -> Vec<&RecurringPattern> {
        let mut category_groups: HashMap<IssueCategory, Vec<&Issue>> = HashMap::new();

        // Group issues by category
        for issue in &self.issues {
            category_groups
                .entry(issue.category)
                .or_default()
                .push(issue);
        }

        // Clear existing patterns before re-detecting
        self.patterns.clear();

        // Detect patterns for each category
        for (category, issues) in category_groups {
            if issues.len() >= frequency_threshold {
                // Collect unique affected agents
                let mut affected_agents: Vec<AgentId> = issues
                    .iter()
                    .flat_map(|i| {
                        let mut agents = vec![i.reporter];
                        agents.extend(&i.related_agents);
                        agents
                    })
                    .collect();
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                affected_agents.sort_unstable();
                affected_agents.dedup();

                // Calculate severity trend
                let first_severity = issues.first().map_or(0.0, |i| i.severity.level() as f32);
                let last_severity = issues.last().map_or(0.0, |i| i.severity.level() as f32);
                let severity_trend = if first_severity > 0.0 {
                    (last_severity - first_severity) / first_severity
                } else {
                    0.0
                };

                // Get time range
                let first_occurrence = issues.iter().map(|i| i.tick_reported).min().unwrap_or(tick);
                let last_occurrence = issues.iter().map(|i| i.tick_reported).max().unwrap_or(tick);

                let pattern = RecurringPattern {
                    pattern_id: self.next_pattern_id,
                    description: format!("Recurring {category:?} issues"),
                    category,
                    frequency: issues.len(),
                    affected_agents,
                    first_occurrence,
                    last_occurrence,
                    severity_trend,
                };

                self.next_pattern_id += 1;
                self.patterns.push(pattern);
            }
        }

        self.patterns.iter().collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get open issues
    #[must_use]
    pub fn get_open_issues(&self) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| {
                matches!(
                    i.status,
                    IssueStatus::Open | IssueStatus::InProgress | IssueStatus::Reopened
                )
            })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get issues by agent (as reporter or related)
    #[must_use]
    pub fn get_agent_issues(&self, agent_id: AgentId) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| i.reporter == agent_id || i.related_agents.contains(&agent_id))
            .collect()
    }

    /// Get issues by category
    #[must_use]
    pub fn get_issues_by_category(&self, category: IssueCategory) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Get issues by severity
    #[must_use]
    pub fn get_issues_by_severity(&self, severity: IssueSeverity) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
    }

    /// Get lessons learned
    ///
    /// Filter by category if specified
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn get_lessons(&self, category: Option<IssueCategory>) -> Vec<&Lesson> {
        match category {
            Some(cat) => self
                .lessons_learned
                .iter()
                .filter(|l| l.category == cat)
                .collect(),
            None => self.lessons_learned.iter().collect(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get lessons by effectiveness threshold
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn get_effective_lessons(&self, min_effectiveness: f32) -> Vec<&Lesson> {
        self.lessons_learned
            .iter()
            .filter(|l| l.effectiveness >= min_effectiveness)
            .collect()
    }

    /// Apply a lesson and track effectiveness
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn apply_lesson(
        &mut self,
        lesson_idx: usize,
        effective: bool,
    ) -> Result<(), IssueLogError> {
        let lesson = self.lessons_learned.get_mut(lesson_idx).ok_or_else(|| {
            IssueLogError::InvalidResolution(format!("Lesson index {lesson_idx} not found"))
        })?;

        lesson.apply(effective);
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute reflection quality metric
    ///
    /// Ratio of issues with lessons / total resolved issues
    #[must_use]
    pub fn reflection_quality(&self) -> f32 {
        let resolved_issues: Vec<_> = self.issues.iter().filter(|i| i.is_resolved()).collect();

        if resolved_issues.is_empty() {
            return 0.0;
        }

        let with_lessons = resolved_issues
            .iter()
            .filter(|i| !i.lessons_learned.is_empty())
            .count();

        with_lessons as f32 / resolved_issues.len() as f32
    }

    /// Compute progress velocity (issues resolved per tick)
    ///
    /// Counts issues resolved in the given window / window size
    #[must_use]
    pub fn progress_velocity(&self, window_ticks: u64, current_tick: Tick) -> f32 {
        if window_ticks == 0 {
            return 0.0;
        }

        let window_start = current_tick.saturating_sub(window_ticks);

        let resolved_in_window = self
            .issues
            .iter()
            .filter(|i| {
                i.tick_resolved
                    .is_some_and(|t| t >= window_start && t <= current_tick)
            })
            .count();

        resolved_in_window as f32 / window_ticks as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get issue by ID
    #[must_use]
    pub fn get_issue(&self, issue_id: u64) -> Option<&Issue> {
        self.issues.iter().find(|i| i.id == issue_id)
    }

    /// Get mutable issue by ID
    #[must_use]
    pub fn get_issue_mut(&mut self, issue_id: u64) -> Option<&mut Issue> {
        self.issues.iter_mut().find(|i| i.id == issue_id)
    }

    /// Get pattern by ID
    #[must_use]
    pub fn get_pattern(&self, pattern_id: u64) -> Option<&RecurringPattern> {
        self.patterns.iter().find(|p| p.pattern_id == pattern_id)
    }

    /// Get total issue count
    #[must_use]
    pub fn total_issues(&self) -> usize {
        self.issues.len()
    }

    /// Get resolution rate (fraction of issues resolved)
    #[must_use]
    pub fn resolution_rate(&self) -> f32 {
        if self.issues.is_empty() {
            return 0.0;
        }

        let resolved = self.issues.iter().filter(|i| i.is_resolved()).count();
        resolved as f32 / self.issues.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get average resolution time for resolved issues
    #[must_use]
    pub fn average_resolution_time(&self) -> Option<f32> {
        let times: Vec<u64> = self
            .issues
            .iter()
            .filter_map(Issue::resolution_time)
            .collect();

        if times.is_empty() {
            return None;
        }

        Some(times.iter().sum::<u64>() as f32 / times.len() as f32)
    }

    /// Get summary statistics
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn stats(&self) -> IssueLogStats {
        IssueLogStats {
            total_issues: self.total_issues(),
            open_count: self.open_count,
            closed_count: self.closed_count,
            resolution_rate: self.resolution_rate(),
            reflection_quality: self.reflection_quality(),
            lesson_count: self.lessons_learned.len(),
            pattern_count: self.patterns.len(),
            avg_resolution_time: self.average_resolution_time(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add a preventive measure to an issue
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn add_preventive_measure(
        &mut self,
        issue_id: u64,
        measure: &str,
    ) -> Result<(), IssueLogError> {
        let issue = self
            .issues
            .iter_mut()
            .find(|i| i.id == issue_id)
            .ok_or(IssueLogError::IssueNotFound(issue_id))?;

        issue.preventive_measures.push(measure.to_string());
        Ok(())
    }

    /// Get high priority issues (High or Critical severity and Open/InProgress)
    #[must_use]
    pub fn get_high_priority_issues(&self) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| {
                matches!(i.severity, IssueSeverity::High | IssueSeverity::Critical)
                    && matches!(
                        i.status,
                        IssueStatus::Open | IssueStatus::InProgress | IssueStatus::Reopened
                    )
            })
            .collect()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Get stale issues (open for longer than threshold ticks)
    #[must_use]
    pub fn get_stale_issues(&self, current_tick: Tick, threshold: u64) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| {
                matches!(
                    i.status,
                    IssueStatus::Open | IssueStatus::InProgress | IssueStatus::Reopened
                ) && (current_tick - i.tick_reported) > threshold
            })
            .collect()
    }
}

/// Statistics summary for the issue log
#[derive(Clone, Debug, PartialEq)]
pub struct IssueLogStats {
    /// total issues.
    pub total_issues: usize,
    /// open count.
    pub open_count: usize,
    /// closed count.
    pub closed_count: usize,
    /// resolution rate.
    pub resolution_rate: f32,
    /// reflection quality.
    pub reflection_quality: f32,
    /// lesson count.
    pub lesson_count: usize,
    /// pattern count.
    pub pattern_count: usize,
    /// avg resolution time.
    pub avg_resolution_time: Option<f32>,
}
