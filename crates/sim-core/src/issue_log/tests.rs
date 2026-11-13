use super::*;

// === Basic Issue Creation Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_issue_creation() {
    let issue = Issue::new(
        1,
        100,
        "Test Issue",
        "This is a test",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![101, 102],
        50,
    );

    assert_eq!(issue.id, 1);
    assert_eq!(issue.reporter, 100);
    assert_eq!(issue.title, "Test Issue");
    assert_eq!(issue.description, "This is a test");
    assert_eq!(issue.severity, IssueSeverity::Medium);
    assert_eq!(issue.category, IssueCategory::Technical);
    assert_eq!(issue.status, IssueStatus::Open);
    assert_eq!(issue.tick_reported, 50);
    assert!(issue.tick_resolved.is_none());
    assert_eq!(issue.related_agents, vec![101, 102]);
}

#[test]
fn test_issue_is_resolved() {
    let mut issue = Issue::new(
        1,
        100,
        "Test",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        10,
    );

    assert!(!issue.is_resolved());

    issue.status = IssueStatus::Resolved;
    assert!(issue.is_resolved());

    issue.status = IssueStatus::Closed;
    assert!(issue.is_resolved());

    issue.status = IssueStatus::Open;
    assert!(!issue.is_resolved());
}

#[test]
fn test_resolution_time() {
    let mut issue = Issue::new(
        1,
        100,
        "Test",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        10,
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(issue.resolution_time().is_none());

    issue.tick_resolved = Some(25);
    assert_eq!(issue.resolution_time(), Some(15));
}

// === IssueLog Core Tests ===

#[test]
fn test_issue_log_new() {
    let log = IssueLog::new();
    assert!(log.issues.is_empty());
    assert!(log.patterns.is_empty());
    assert!(log.lessons_learned.is_empty());
    assert_eq!(log.next_issue_id, 1);
    assert_eq!(log.next_pattern_id, 1);
    assert_eq!(log.open_count, 0);
    assert_eq!(log.closed_count, 0);
}

#[test]
fn test_record_issue() {
    let mut log = IssueLog::new();

    let id = log.record_issue(
        100,
        "Network Error",
        "Connection timeout",
        IssueSeverity::High,
        IssueCategory::Technical,
        vec![101],
        50,
    );

    assert_eq!(id, 1);
    assert_eq!(log.total_issues(), 1);
    assert_eq!(log.open_count, 1);

    let issue = log.get_issue(id).unwrap();
    assert_eq!(issue.title, "Network Error");
    assert_eq!(issue.severity, IssueSeverity::High);
}

#[test]
fn test_record_multiple_issues() {
    let mut log = IssueLog::new();

    let id1 = log.record_issue(
        100,
        "Issue 1",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "Desc 1",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    let id2 = log.record_issue(
        101,
        "Issue 2",
        "Desc 2",
        IssueSeverity::Medium,
        IssueCategory::Process,
        vec![],
        20,
    );
    let id3 = log.record_issue(
        102,
        "Issue 3",
        "Desc 3",
        IssueSeverity::High,
        IssueCategory::Social,
        vec![],
        30,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert_eq!(log.total_issues(), 3);
}

// === Resolution Tests ===

#[test]
fn test_resolve_issue_success() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );

    let result = log.resolve_issue(
        id,
        "Fixed by restarting server",
        vec!["Always check server status".to_string()],
        "Server overload",
        vec!["Add monitoring".to_string()],
        20,
    );

    assert!(result.is_ok());
    assert_eq!(log.open_count, 0);
    assert_eq!(log.closed_count, 1);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let issue = log.get_issue(id).unwrap();
    assert_eq!(issue.status, IssueStatus::Resolved);
    assert_eq!(issue.tick_resolved, Some(20));
    assert_eq!(
        issue.resolution,
        Some("Fixed by restarting server".to_string())
    );
    assert_eq!(issue.lessons_learned.len(), 1);
    assert_eq!(issue.root_cause, Some("Server overload".to_string()));
}

#[test]
fn test_resolve_issue_not_found() {
    let mut log = IssueLog::new();

    let result = log.resolve_issue(999, "Resolution", vec![], "Cause", vec![], 20);

    assert_eq!(result, Err(IssueLogError::IssueNotFound(999)));
}

#[test]
fn test_resolve_already_resolved() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );

    // First resolution
    log.resolve_issue(id, "Resolution 1", vec![], "Cause", vec![], 20)
        .unwrap();

    // Second resolution attempt should fail
    let result = log.resolve_issue(id, "Resolution 2", vec![], "Cause", vec![], 30);
    assert_eq!(result, Err(IssueLogError::AlreadyResolved(id)));
}

#[test]
fn test_resolve_empty_resolution() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let result = log.resolve_issue(id, "", vec![], "Cause", vec![], 20);
    assert!(matches!(result, Err(IssueLogError::InvalidResolution(_))));
}

#[test]
fn test_resolve_extracts_lessons() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );

    log.resolve_issue(
        id,
        "Fixed",
        vec!["Lesson 1".to_string(), "Lesson 2".to_string()],
        "Cause",
        vec![],
        20,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    )
    .unwrap();

    assert_eq!(log.lessons_learned.len(), 2);
    assert_eq!(log.lessons_learned[0].lesson, "Lesson 1");
    assert_eq!(log.lessons_learned[0].source_issue, id);
    assert_eq!(log.lessons_learned[0].category, IssueCategory::Technical);
}

// === Issue Lifecycle Tests ===

#[test]
fn test_start_progress() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );

    assert!(log.start_progress(id).is_ok());
    assert_eq!(log.get_issue(id).unwrap().status, IssueStatus::InProgress);
}

#[test]
fn test_close_issue() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.resolve_issue(id, "Fixed", vec![], "Cause", vec![], 20)
        .unwrap();

    assert!(log.close_issue(id).is_ok());
    assert_eq!(log.get_issue(id).unwrap().status, IssueStatus::Closed);
}

#[test]
fn test_close_unresolved_issue_fails() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        vec![],
        10,
    );

    let result = log.close_issue(id);
    assert!(result.is_err());
}

#[test]
fn test_reopen_issue() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.resolve_issue(id, "Fixed", vec![], "Cause", vec![], 20)
        .unwrap();

    assert_eq!(log.closed_count, 1);
    assert_eq!(log.open_count, 0);

    assert!(log.reopen_issue(id).is_ok());

    let issue = log.get_issue(id).unwrap();
    assert_eq!(issue.status, IssueStatus::Reopened);
    assert!(issue.tick_resolved.is_none());
    assert_eq!(log.open_count, 1);
    assert_eq!(log.closed_count, 0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// === Query Tests ===

#[test]
fn test_get_open_issues() {
    let mut log = IssueLog::new();
    log.record_issue(
        100,
        "Open 1",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    let _id2 = log.record_issue(
        101,
        "Open 2",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        vec![],
        20,
    );
    let id3 = log.record_issue(
        102,
        "Resolved",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Social,
        vec![],
        30,
    );

    log.resolve_issue(id3, "Fixed", vec![], "Cause", vec![], 40)
        .unwrap();

    let open = log.get_open_issues();
    assert_eq!(open.len(), 2);
    assert!(open.iter().all(|i| i.id != id3));
}

#[test]
fn test_get_agent_issues() {
    let mut log = IssueLog::new();
    log.record_issue(
        100,
        "Reporter 100",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![101, 102],
        // TAG: surface=simulation owner=research-team rule=SIM-001
        10,
    );
    log.record_issue(
        101,
        "Reporter 101",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![100],
        20,
    );
    log.record_issue(
        102,
        "Reporter 102",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Social,
        vec![200],
        30,
    );

    let issues_100 = log.get_agent_issues(100);
    assert_eq!(issues_100.len(), 2); // Reported one, related in another
                                     // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    let issues_200 = log.get_agent_issues(200);
    assert_eq!(issues_200.len(), 1);
}

#[test]
fn test_get_issues_by_category() {
    let mut log = IssueLog::new();
    log.record_issue(
        100,
        "Tech",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.record_issue(
        101,
        "Tech",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        20,
    );
    log.record_issue(
        102,
        "Social",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Social,
        vec![],
        30,
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let tech_issues = log.get_issues_by_category(IssueCategory::Technical);
    assert_eq!(tech_issues.len(), 2);

    let social_issues = log.get_issues_by_category(IssueCategory::Social);
    assert_eq!(social_issues.len(), 1);
}

#[test]
fn test_get_issues_by_severity() {
    let mut log = IssueLog::new();
    log.record_issue(
        100,
        "Low",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.record_issue(
        101,
        "Critical",
        "Desc",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        IssueSeverity::Critical,
        IssueCategory::Process,
        vec![],
        20,
    );
    log.record_issue(
        102,
        "Critical",
        "Desc",
        IssueSeverity::Critical,
        IssueCategory::Social,
        vec![],
        30,
    );

    let critical = log.get_issues_by_severity(IssueSeverity::Critical);
    assert_eq!(critical.len(), 2);
}

// === Pattern Detection Tests ===

#[test]
fn test_detect_patterns() {
    let mut log = IssueLog::new();

    // Create 3 technical issues
    for i in 0..3 {
        log.record_issue(
            100 + i,
            "Tech Issue",
            "Desc",
            IssueSeverity::Medium,
            IssueCategory::Technical,
            vec![],
            i * 10,
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Create 2 social issues
    for i in 0..2 {
        log.record_issue(
            200 + i,
            "Social Issue",
            "Desc",
            IssueSeverity::Low,
            IssueCategory::Social,
            vec![],
            i * 10 + 50,
        );
    }

    // Detect patterns with threshold of 3
    let patterns = log.detect_patterns(100, 3);
    assert_eq!(patterns.len(), 1); // Only technical meets threshold
    assert_eq!(patterns[0].category, IssueCategory::Technical);
    assert_eq!(patterns[0].frequency, 3);
}

#[test]
fn test_detect_patterns_multiple() {
    let mut log = IssueLog::new();

    // Create issues in multiple categories
    for i in 0..3 {
        log.record_issue(
            100 + i,
            "Tech",
            "Desc",
            IssueSeverity::Medium,
            IssueCategory::Technical,
            vec![],
            i * 10,
        );
        log.record_issue(
            200 + i,
            "Social",
            "Desc",
            IssueSeverity::Low,
            IssueCategory::Social,
            vec![],
            i * 10 + 50,
        );
    }

    let patterns = log.detect_patterns(200, 2);
    assert_eq!(patterns.len(), 2);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// === Lesson Tests ===

#[test]
fn test_lesson_creation() {
    let lesson = Lesson::new("Test lesson", 1, 100, IssueCategory::Technical);
    assert_eq!(lesson.lesson, "Test lesson");
    assert_eq!(lesson.source_issue, 1);
    assert_eq!(lesson.tick_learned, 100);
    assert_eq!(lesson.category, IssueCategory::Technical);
    assert_eq!(lesson.application_count, 0);
    assert_eq!(lesson.effectiveness, 0.5);
}

#[test]
fn test_lesson_apply_effective() {
    let mut lesson = Lesson::new("Test", 1, 100, IssueCategory::Technical);

    lesson.apply(true);
    assert_eq!(lesson.application_count, 1);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!(lesson.effectiveness > 0.5); // Should increase
}

#[test]
fn test_lesson_apply_ineffective() {
    let mut lesson = Lesson::new("Test", 1, 100, IssueCategory::Technical);

    lesson.apply(false);
    assert_eq!(lesson.application_count, 1);
    assert!(lesson.effectiveness < 0.5); // Should decrease
}

#[test]
fn test_lesson_apply_multiple() {
    let mut lesson = Lesson::new("Test", 1, 100, IssueCategory::Technical);

    // Apply multiple times with mixed effectiveness
    lesson.apply(true); // effectiveness goes up
    lesson.apply(true); // goes up more
    lesson.apply(false); // goes down some

    assert_eq!(lesson.application_count, 3);
    assert!(lesson.effectiveness > 0.0 && lesson.effectiveness < 1.0);
}

#[test]
fn test_get_lessons() {
    let mut log = IssueLog::new();
    let id1 = log.record_issue(
        100,
        "Tech",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        // TAG: surface=simulation owner=research-team rule=SIM-001
        10,
    );
    let id2 = log.record_issue(
        101,
        "Social",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Social,
        vec![],
        20,
    );

    log.resolve_issue(
        id1,
        "Fixed",
        vec!["Tech lesson".to_string()],
        "Cause",
        vec![],
        30,
    )
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
    .unwrap();
    log.resolve_issue(
        id2,
        "Fixed",
        vec!["Social lesson".to_string()],
        "Cause",
        vec![],
        40,
    )
    .unwrap();

    let all_lessons = log.get_lessons(None);
    assert_eq!(all_lessons.len(), 2);

    let tech_lessons = log.get_lessons(Some(IssueCategory::Technical));
    assert_eq!(tech_lessons.len(), 1);
    assert_eq!(tech_lessons[0].lesson, "Tech lesson");
}

#[test]
fn test_apply_lesson() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.resolve_issue(id, "Fixed", vec!["Lesson".to_string()], "Cause", vec![], 20)
        .unwrap();

    let result = log.apply_lesson(0, true);
    assert!(result.is_ok());
    assert_eq!(log.lessons_learned[0].application_count, 1);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_apply_lesson_invalid_index() {
    let mut log = IssueLog::new();
    let result = log.apply_lesson(0, true);
    assert!(result.is_err());
}

#[test]
fn test_get_effective_lessons() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.resolve_issue(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        id,
        "Fixed",
        vec!["Lesson 1".to_string(), "Lesson 2".to_string()],
        "Cause",
        vec![],
        20,
    )
    .unwrap();

    // Apply lessons with different effectiveness
    log.apply_lesson(0, true).unwrap();
    log.apply_lesson(0, true).unwrap(); // Very effective
    log.apply_lesson(1, false).unwrap(); // Not effective

    let effective = log.get_effective_lessons(0.6);
    assert_eq!(effective.len(), 1);
    assert_eq!(effective[0].lesson, "Lesson 1");
}

// === Metrics Tests ===

#[test]
fn test_reflection_quality() {
    let mut log = IssueLog::new();

    // No resolved issues = 0 quality
    assert_eq!(log.reflection_quality(), 0.0);

    // Add issue without lessons
    let id1 = log.record_issue(
        100,
        "No lessons",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.resolve_issue(id1, "Fixed", vec![], "Cause", vec![], 20)
        .unwrap();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // One resolved, no lessons = 0 quality
    assert_eq!(log.reflection_quality(), 0.0);

    // Add issue with lessons
    let id2 = log.record_issue(
        101,
        "With lessons",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        30,
    );
    log.resolve_issue(
        id2,
        "Fixed",
        vec!["Lesson".to_string()],
        "Cause",
        vec![],
        40,
    )
    .unwrap();

    // Two resolved, one with lessons = 0.5 quality
    assert_eq!(log.reflection_quality(), 0.5);
}

#[test]
fn test_progress_velocity() {
    let mut log = IssueLog::new();

    // No issues resolved = 0 velocity
    assert_eq!(log.progress_velocity(100, 200), 0.0);

    // Resolve some issues at different times
    let id1 = log.record_issue(
        100,
        "Issue 1",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        50,
    );
    let id2 = log.record_issue(
        101,
        "Issue 2",
        "Desc",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        60,
    );
    let id3 = log.record_issue(
        102,
        "Issue 3",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Social,
        vec![],
        70,
    );

    log.resolve_issue(id1, "Fixed", vec![], "Cause", vec![], 100)
        .unwrap();
    log.resolve_issue(id2, "Fixed", vec![], "Cause", vec![], 110)
        .unwrap();
    log.resolve_issue(id3, "Fixed", vec![], "Cause", vec![], 200)
        .unwrap();

    // Window of 50 ticks ending at 150: issues at 100 and 110 are in window
    let velocity = log.progress_velocity(50, 150);
    assert_eq!(velocity, 2.0 / 50.0);
}

#[test]
fn test_progress_velocity_zero_window() {
    let log = IssueLog::new();
    assert_eq!(log.progress_velocity(0, 100), 0.0);
}

#[test]
fn test_resolution_rate() {
    let mut log = IssueLog::new();

    // No issues = 0 rate
    assert_eq!(log.resolution_rate(), 0.0);

    let id1 = log.record_issue(
        100,
        "Issue 1",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    );
    log.record_issue(
        101,
        "Issue 2",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        20,
    );
    log.record_issue(
        102,
        "Issue 3",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Social,
        vec![],
        30,
    );

    // 0/3 resolved
    assert_eq!(log.resolution_rate(), 0.0);

    log.resolve_issue(id1, "Fixed", vec![], "Cause", vec![], 40)
        .unwrap();

    // 1/3 resolved
    assert!((log.resolution_rate() - 0.333).abs() < 0.01);
}

#[test]
fn test_average_resolution_time() {
    let mut log = IssueLog::new();

    // No resolved issues = None
    assert!(log.average_resolution_time().is_none());

    let id1 = log.record_issue(
        100,
        "Issue 1",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    let id2 = log.record_issue(
        101,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "Issue 2",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        20,
    );

    log.resolve_issue(id1, "Fixed", vec![], "Cause", vec![], 30)
        .unwrap(); // 20 ticks
    log.resolve_issue(id2, "Fixed", vec![], "Cause", vec![], 40)
        .unwrap(); // 20 ticks

    assert_eq!(log.average_resolution_time(), Some(20.0));
}

#[test]
fn test_stats() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.resolve_issue(id, "Fixed", vec!["Lesson".to_string()], "Cause", vec![], 20)
        .unwrap();

    let stats = log.stats();
    assert_eq!(stats.total_issues, 1);
    assert_eq!(stats.open_count, 0);
    assert_eq!(stats.closed_count, 1);
    assert_eq!(stats.resolution_rate, 1.0);
    assert_eq!(stats.reflection_quality, 1.0);
    assert_eq!(stats.lesson_count, 1);
    assert_eq!(stats.avg_resolution_time, Some(10.0));
}

// === High Priority and Stale Issue Tests ===

#[test]
fn test_get_high_priority_issues() {
    let mut log = IssueLog::new();

    let id1 = log.record_issue(
        100,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        "Critical Open",
        "Desc",
        IssueSeverity::Critical,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.record_issue(
        101,
        "Low Open",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        20,
    );
    let id3 = log.record_issue(
        102,
        "High Open",
        "Desc",
        IssueSeverity::High,
        IssueCategory::Social,
        vec![],
        30,
    );
    let id4 = log.record_issue(
        103,
        "Critical Resolved",
        "Desc",
        IssueSeverity::Critical,
        IssueCategory::Security,
        vec![],
        40,
    );

    log.resolve_issue(id4, "Fixed", vec![], "Cause", vec![], 50)
        .unwrap();

    let high_priority = log.get_high_priority_issues();
    assert_eq!(high_priority.len(), 2);
    assert!(high_priority.iter().any(|i| i.id == id1));
    assert!(high_priority.iter().any(|i| i.id == id3));
}

#[test]
fn test_get_stale_issues() {
    let mut log = IssueLog::new();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Create issues at different times
    log.record_issue(
        100,
        "Stale",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.record_issue(
        101,
        "Not Stale",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Process,
        vec![],
        80,
    );
    let id3 = log.record_issue(
        102,
        "Resolved",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Social,
        vec![],
        10,
    );
    log.resolve_issue(id3, "Fixed", vec![], "Cause", vec![], 20)
        .unwrap();

    let stale = log.get_stale_issues(100, 50); // Current tick 100, threshold 50
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].title, "Stale");
}

// === Utility Tests ===

#[test]
fn test_add_preventive_measure() {
    let mut log = IssueLog::new();
    let id = log.record_issue(
        100,
        "Test",
        "Desc",
        IssueSeverity::Medium,
        IssueCategory::Technical,
        vec![],
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        10,
    );

    assert!(log.add_preventive_measure(id, "Add monitoring").is_ok());
    assert!(log.add_preventive_measure(id, "Add alerts").is_ok());

    let issue = log.get_issue(id).unwrap();
    assert_eq!(issue.preventive_measures.len(), 2);
    assert!(issue
        .preventive_measures
        .contains(&"Add monitoring".to_string()));
}

#[test]
fn test_add_preventive_measure_not_found() {
    let mut log = IssueLog::new();
    let result = log.add_preventive_measure(999, "Measure");
    assert_eq!(result, Err(IssueLogError::IssueNotFound(999)));
}

#[test]
fn test_issue_severity_level() {
    assert_eq!(IssueSeverity::Low.level(), 1);
    assert_eq!(IssueSeverity::Medium.level(), 2);
    assert_eq!(IssueSeverity::High.level(), 3);
    assert_eq!(IssueSeverity::Critical.level(), 4);
}

#[test]
fn test_issue_category_display() {
    assert_eq!(format!("{}", IssueCategory::Technical), "Technical");
    assert_eq!(format!("{}", IssueCategory::Process), "Process");
    assert_eq!(format!("{}", IssueCategory::Social), "Social");
}

#[test]
fn test_issue_status_display() {
    assert_eq!(format!("{}", IssueStatus::Open), "Open");
    assert_eq!(format!("{}", IssueStatus::InProgress), "InProgress");
    assert_eq!(format!("{}", IssueStatus::Resolved), "Resolved");
}

#[test]
fn test_error_display() {
    let err = IssueLogError::IssueNotFound(42);
    assert!(err.to_string().contains("42"));

    let err = IssueLogError::AlreadyResolved(42);
    assert!(err.to_string().contains("already resolved"));
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_pattern_average_severity() {
    let mut log = IssueLog::new();

    // Create issues with varying severities
    log.record_issue(
        100,
        "Low",
        "Desc",
        IssueSeverity::Low,
        IssueCategory::Technical,
        vec![],
        10,
    );
    log.record_issue(
        101,
        "High",
        "Desc",
        IssueSeverity::High,
        IssueCategory::Technical,
        vec![],
        20,
    );
    log.record_issue(
        102,
        "Critical",
        "Desc",
        IssueSeverity::Critical,
        IssueCategory::Technical,
        vec![],
        30,
    );

    log.detect_patterns(50, 2);

    let pattern = log.get_pattern(1).unwrap();
    let avg = pattern.average_severity(&log.issues);
    // (1 + 3 + 4) / 3 = 2.67
    assert!((avg - 2.67).abs() < 0.1);
}

#[test]
fn test_default_impl() {
    let log: IssueLog = IssueLog::default();
    assert!(log.issues.is_empty());
    assert_eq!(log.next_issue_id, 0); // Default starts at 0
}
