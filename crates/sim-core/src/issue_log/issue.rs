use crate::types::{AgentId, Tick};
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// An issue/problem tracked in the system
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue {
    /// id.
    pub id: u64,
    /// reporter.
    pub reporter: AgentId,
    /// title.
    pub title: String,
    /// description.
    pub description: String,
    /// severity.
    pub severity: IssueSeverity,
    /// category.
    pub category: IssueCategory,
    /// status.
    pub status: IssueStatus,
    /// tick reported.
    pub tick_reported: Tick,
    /// tick resolved.
    pub tick_resolved: Option<Tick>,
    /// resolution.
    pub resolution: Option<String>,
    /// lessons learned.
    pub lessons_learned: Vec<String>,
    /// related agents.
    pub related_agents: Vec<AgentId>,
    /// root cause.
    pub root_cause: Option<String>,
    /// preventive measures.
    pub preventive_measures: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Issue {
    /// Create a new issue
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        id: u64,
        reporter: AgentId,
        title: &str,
        description: &str,
        severity: IssueSeverity,
        category: IssueCategory,
        related_agents: Vec<AgentId>,
        tick: Tick,
    ) -> Self {
        Self {
            id,
            reporter,
            title: title.to_string(),
            description: description.to_string(),
            severity,
            category,
            status: IssueStatus::Open,
            tick_reported: tick,
            tick_resolved: None,
            resolution: None,
            lessons_learned: Vec::new(),
            related_agents,
            root_cause: None,
            preventive_measures: Vec::new(),
        }
    }

    /// Check if issue is resolved
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        matches!(self.status, IssueStatus::Resolved | IssueStatus::Closed)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get resolution time in ticks
    #[must_use]
    pub fn resolution_time(&self) -> Option<u64> {
        self.tick_resolved
            .map(|resolved| resolved - self.tick_reported)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// issue severity.
pub enum IssueSeverity {
    /// low.
    Low, // Minor inconvenience
    /// medium.
    Medium, // Significant problem
    /// high.
    High, // Serious issue
    /// critical.
    Critical, // System-threatening
}

impl IssueSeverity {
    /// Get numeric severity level (higher = more severe)
    #[must_use]
    pub const fn level(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
/// issue category.
pub enum IssueCategory {
    /// technical.
    Technical, // Bug, error
    /// process.
    Process, // Workflow problem
    /// social.
    Social, // Interpersonal conflict
    /// economic.
    Economic, // Financial loss
    /// security.
    Security, // Vulnerability
    /// coordination.
    Coordination, // Failed coordination
    /// knowledge.
    Knowledge, // Missing information
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Technical => write!(f, "Technical"),
            Self::Process => write!(f, "Process"),
            Self::Social => write!(f, "Social"),
            Self::Economic => write!(f, "Economic"),
            Self::Security => write!(f, "Security"),
            Self::Coordination => write!(f, "Coordination"),
            Self::Knowledge => write!(f, "Knowledge"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// issue status.
pub enum IssueStatus {
    /// open.
    Open,
    /// in progress.
    InProgress,
    /// resolved.
    Resolved,
    /// closed.
    Closed,
    /// reopened.
    Reopened,
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::InProgress => write!(f, "InProgress"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Closed => write!(f, "Closed"),
            Self::Reopened => write!(f, "Reopened"),
        }
    }
}
