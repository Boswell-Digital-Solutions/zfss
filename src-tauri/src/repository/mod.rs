//! ZFSS Repository Layer
//!
//! Provides append-only access patterns for canonical objects so callers never issue UPDATE/DELETE
//! on `issues`, `signals`, `decisions`, `artifacts`, or `responses`.

use crate::models::ids::{ArtifactId, DecisionId, IssueId, ResponseId, SignalId};
use crate::models::{
    ApprovalState, Artifact, ArtifactCreate, ArtifactSummary, Decision, DecisionCreate,
    DecisionHistoryEntry, Issue, IssueCreate, IssueStatus, IssueSummary, Response, ResponseCreate,
    ResponseSummary, Signal, SignalCreate, SignalStatus,
};
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

/// Insert a new signal and record its initial status in `signal_status_history`.
pub async fn append_signal(pool: &PgPool, input: SignalCreate, created_by: &str) -> Result<Signal> {
    let signal_id = SignalId::new();
    let environment = to_json_value(input.environment)?;
    let reporter = to_json_value(input.reporter)?;
    let mut tx = pool
        .begin()
        .await
        .context("begin signal append transaction")?;

    sqlx::query(
        "INSERT INTO signals (id, source, raw_text, app_key, app_version, environment, reporter, status, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'new', $8)",
    )
    .bind(signal_id.as_str())
    .bind(input.source.as_str())
    .bind(input.raw_text)
    .bind(input.app_key)
    .bind(input.app_version)
    .bind(environment)
    .bind(reporter)
    .bind(created_by)
    .execute(&mut *tx)
    .await
    .context("failed to insert signal")?;

    sqlx::query(
        "INSERT INTO signal_status_history (signal_id, new_status, changed_by, reason)
        VALUES ($1, 'new', $2, 'Initial capture')",
    )
    .bind(signal_id.as_str())
    .bind(created_by)
    .execute(&mut *tx)
    .await
    .context("failed to insert signal status history for new signal")?;

    tx.commit()
        .await
        .context("commit signal insert transaction")?;

    get_signal(pool, signal_id.as_str()).await?.ok_or_else(|| {
        anyhow!(
            "signal {} was inserted but could not be loaded afterwards",
            signal_id.as_str()
        )
    })
}

/// List signals, optionally filtered by lifecycle status, ordered newest first while deriving status/link state from history tables.
pub async fn list_signals(
    pool: &PgPool,
    status_filter: Option<SignalStatus>,
    limit: i64,
) -> Result<Vec<Signal>> {
    let limit = limit.clamp(1, 100);
    let status_value = status_filter.map(|s| s.as_str().to_string());

    let rows = sqlx::query_as::<_, SignalRow>(
        r#"
        SELECT
            s.id,
            s.source,
            s.raw_text,
            s.app_key,
            s.app_version,
            s.environment,
            s.reporter,
            COALESCE(latest_status.new_status, s.status) AS status,
            latest_link.issue_id AS linked_issue_id,
            s.created_at,
            s.created_by
        FROM signals s
        LEFT JOIN LATERAL (
            SELECT new_status
            FROM signal_status_history h
            WHERE h.signal_id = s.id
            ORDER BY h.changed_at DESC, h.id DESC
            LIMIT 1
        ) AS latest_status ON TRUE
        LEFT JOIN LATERAL (
            SELECT issue_id
            FROM signal_links l
            WHERE l.signal_id = s.id
            ORDER BY l.linked_at DESC, l.id DESC
            LIMIT 1
        ) AS latest_link ON TRUE
        WHERE ($1::text IS NULL OR COALESCE(latest_status.new_status, s.status) = $1)
        ORDER BY s.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(status_value.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("failed to query signals")?;

    Ok(rows.into_iter().map(|row| row.into_signal()).collect())
}

/// Fetch the detailed signal record, including the latest status and linked issue (via append-only tables).
pub async fn get_signal(pool: &PgPool, signal_id: &str) -> Result<Option<Signal>> {
    let row = sqlx::query_as::<_, SignalRow>(
        r#"
        SELECT
            s.id,
            s.source,
            s.raw_text,
            s.app_key,
            s.app_version,
            s.environment,
            s.reporter,
            COALESCE(latest_status.new_status, s.status) AS status,
            latest_link.issue_id AS linked_issue_id,
            s.created_at,
            s.created_by
        FROM signals s
        LEFT JOIN LATERAL (
            SELECT new_status
            FROM signal_status_history h
            WHERE h.signal_id = s.id
            ORDER BY h.changed_at DESC, h.id DESC
            LIMIT 1
        ) AS latest_status ON TRUE
        LEFT JOIN LATERAL (
            SELECT issue_id
            FROM signal_links l
            WHERE l.signal_id = s.id
            ORDER BY l.linked_at DESC, l.id DESC
            LIMIT 1
        ) AS latest_link ON TRUE
        WHERE s.id = $1
        "#,
    )
    .bind(signal_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch signal")?;

    Ok(row.map(|row| row.into_signal()))
}

/// Link a signal to an issue by recording an append-only history entry and logging the link event.
pub async fn link_signal_to_issue(
    pool: &PgPool,
    signal_id: &str,
    issue_id: &str,
    actor: &str,
) -> Result<Signal> {
    let mut tx = pool
        .begin()
        .await
        .context("begin signal link transaction")?;

    let signal_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM signals WHERE id = $1)")
            .bind(signal_id)
            .fetch_one(&mut *tx)
            .await
            .context("failed to verify signal existence")?;

    if !signal_exists {
        return Err(anyhow!("Signal not found: {}", signal_id));
    }

    let issue_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM issues WHERE id = $1)")
            .bind(issue_id)
            .fetch_one(&mut *tx)
            .await
            .context("failed to verify issue existence")?;

    if !issue_exists {
        return Err(anyhow!("Issue not found: {}", issue_id));
    }

    let current_status: Option<String> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(
            (
                SELECT new_status
                FROM signal_status_history h
                WHERE h.signal_id = $1
                ORDER BY h.changed_at DESC, h.id DESC
                LIMIT 1
            ),
            (
                SELECT status
                FROM signals
                WHERE id = $1
            )
        )
        "#,
    )
    .bind(signal_id)
    .fetch_one(&mut *tx)
    .await
    .context("failed to read current signal status")?;

    sqlx::query(
        "INSERT INTO signal_status_history (signal_id, old_status, new_status, changed_by, reason)
        VALUES ($1, $2, 'linked', $3, 'Linked to issue via UI')",
    )
    .bind(signal_id)
    .bind(current_status.as_deref())
    .bind(actor)
    .execute(&mut *tx)
    .await
    .context("failed to append signal status history for link")?;

    sqlx::query(
        "INSERT INTO signal_links (signal_id, issue_id, linked_by, reason)
        VALUES ($1, $2, $3, 'Linked via desktop application')",
    )
    .bind(signal_id)
    .bind(issue_id)
    .bind(actor)
    .execute(&mut *tx)
    .await
    .context("failed to insert signal link event")?;

    sqlx::query(
        "INSERT INTO audit_log (event_type, entity_type, entity_id, actor, actor_role, action, details)
        VALUES ('signal', 'signal_link', $1, $2, 'Operator', 'link', $3)",
    )
    .bind(signal_id)
    .bind(actor)
    .bind(serde_json::json!({
        "target_issue": issue_id,
        "reason": "Linked from desktop UI"
    }))
    .execute(&mut *tx)
    .await
    .context("failed to record audit trail for link")?;

    tx.commit()
        .await
        .context("commit signal link transaction")?;

    get_signal(pool, signal_id)
        .await?
        .ok_or_else(|| anyhow!("signal {} disappeared after linking", signal_id))
}

fn to_json_value<T>(value: Option<T>) -> Result<Option<Value>>
where
    T: Serialize,
{
    value
        .map(|inner| serde_json::to_value(inner).context("serialize JSON field"))
        .transpose()
}

#[derive(sqlx::FromRow)]
struct SignalRow {
    id: String,
    source: String,
    raw_text: String,
    app_key: Option<String>,
    app_version: Option<String>,
    environment: Option<Value>,
    reporter: Option<Value>,
    status: String,
    linked_issue_id: Option<String>,
    created_at: DateTime<Utc>,
    created_by: String,
}

impl SignalRow {
    fn into_signal(self) -> Signal {
        Signal {
            id: self.id,
            source: self.source,
            raw_text: self.raw_text,
            app_key: self.app_key,
            app_version: self.app_version,
            environment: json_to(self.environment),
            reporter: json_to(self.reporter),
            status: self.status,
            linked_issue_id: self.linked_issue_id,
            created_at: self.created_at,
            created_by: self.created_by,
        }
    }
}

fn json_to<T>(value: Option<Value>) -> Option<T>
where
    T: DeserializeOwned,
{
    value.and_then(|v| serde_json::from_value(v).ok())
}

// ===========================================================================
// ISSUE REPOSITORY
// ===========================================================================

/// Insert a new issue and record its initial status in `issue_status_history`.
pub async fn append_issue(pool: &PgPool, input: IssueCreate, created_by: &str) -> Result<Issue> {
    let issue_id = IssueId::new();
    let mut tx = pool
        .begin()
        .await
        .context("begin issue append transaction")?;

    sqlx::query(
        r#"INSERT INTO issues (id, title, description, classification, severity, frequency, status, close_requires_artifact, created_by)
        VALUES ($1, $2, $3, $4, $5, 1, 'pending_decision', TRUE, $6)"#,
    )
    .bind(issue_id.as_str())
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.classification.as_str())
    .bind(input.severity.as_str())
    .bind(created_by)
    .execute(&mut *tx)
    .await
    .context("failed to insert issue")?;

    sqlx::query(
        "INSERT INTO issue_status_history (issue_id, new_status, changed_by, reason)
        VALUES ($1, 'pending_decision', $2, 'Issue created')",
    )
    .bind(issue_id.as_str())
    .bind(created_by)
    .execute(&mut *tx)
    .await
    .context("failed to insert issue status history")?;

    tx.commit()
        .await
        .context("commit issue insert transaction")?;

    get_issue(pool, issue_id.as_str()).await?.ok_or_else(|| {
        anyhow!(
            "issue {} was inserted but could not be loaded",
            issue_id.as_str()
        )
    })
}

/// List issues, optionally filtered by status, ordered newest first.
pub async fn list_issues(
    pool: &PgPool,
    status_filter: Option<IssueStatus>,
    limit: i64,
) -> Result<Vec<IssueSummary>> {
    let limit = limit.clamp(1, 100);
    let status_value = status_filter.map(|s| s.as_str().to_string());

    let rows = sqlx::query_as::<_, IssueSummaryRow>(
        r#"
        SELECT
            i.id,
            i.title,
            i.classification,
            i.severity,
            COALESCE(latest_status.new_status, i.status) AS status,
            COUNT(s.id) AS signal_count,
            i.created_at
        FROM issues i
        LEFT JOIN LATERAL (
            SELECT new_status
            FROM issue_status_history h
            WHERE h.issue_id = i.id
            ORDER BY h.changed_at DESC, h.id DESC
            LIMIT 1
        ) AS latest_status ON TRUE
        LEFT JOIN signals s ON s.linked_issue_id = i.id
        WHERE ($1::text IS NULL OR COALESCE(latest_status.new_status, i.status) = $1)
        GROUP BY i.id, i.title, i.classification, i.severity, i.status, i.created_at, latest_status.new_status
        ORDER BY i.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(status_value.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("failed to query issues")?;

    Ok(rows.into_iter().map(|row| row.into_summary()).collect())
}

/// Fetch a single issue by ID.
pub async fn get_issue(pool: &PgPool, issue_id: &str) -> Result<Option<Issue>> {
    let row = sqlx::query_as::<_, IssueRow>(
        r#"
        SELECT
            i.id,
            i.title,
            i.description,
            i.classification,
            i.severity,
            i.frequency,
            COALESCE(latest_status.new_status, i.status) AS status,
            i.close_requires_artifact,
            i.created_at,
            i.created_by
        FROM issues i
        LEFT JOIN LATERAL (
            SELECT new_status
            FROM issue_status_history h
            WHERE h.issue_id = i.id
            ORDER BY h.changed_at DESC, h.id DESC
            LIMIT 1
        ) AS latest_status ON TRUE
        WHERE i.id = $1
        "#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch issue")?;

    Ok(row.map(|r| r.into_issue()))
}

/// Transition an issue's status (append-only: inserts into history, updates current).
pub async fn transition_issue_status(
    pool: &PgPool,
    issue_id: &str,
    new_status: IssueStatus,
    actor: &str,
    reason: Option<&str>,
) -> Result<Issue> {
    let mut tx = pool
        .begin()
        .await
        .context("begin issue status transition")?;

    let current_status: Option<String> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(
            (SELECT new_status FROM issue_status_history WHERE issue_id = $1 ORDER BY changed_at DESC, id DESC LIMIT 1),
            (SELECT status FROM issues WHERE id = $1)
        )
        "#,
    )
    .bind(issue_id)
    .fetch_one(&mut *tx)
    .await
    .context("failed to read current issue status")?;

    let current = current_status
        .as_deref()
        .and_then(IssueStatus::from_str)
        .ok_or_else(|| anyhow!("Issue not found: {}", issue_id))?;

    if !current.can_transition_to(new_status) {
        return Err(anyhow!(
            "Invalid status transition: {} -> {}",
            current.as_str(),
            new_status.as_str()
        ));
    }

    sqlx::query(
        "INSERT INTO issue_status_history (issue_id, old_status, new_status, changed_by, reason)
        VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(issue_id)
    .bind(current.as_str())
    .bind(new_status.as_str())
    .bind(actor)
    .bind(reason)
    .execute(&mut *tx)
    .await
    .context("failed to insert issue status history")?;

    // Update the denormalized status column
    sqlx::query("UPDATE issues SET status = $1 WHERE id = $2")
        .bind(new_status.as_str())
        .bind(issue_id)
        .execute(&mut *tx)
        .await
        .context("failed to update issue status")?;

    tx.commit()
        .await
        .context("commit issue status transition")?;

    get_issue(pool, issue_id)
        .await?
        .ok_or_else(|| anyhow!("issue {} disappeared after transition", issue_id))
}

#[derive(sqlx::FromRow)]
struct IssueRow {
    id: String,
    title: String,
    description: Option<String>,
    classification: String,
    severity: String,
    frequency: i32,
    status: String,
    close_requires_artifact: bool,
    created_at: DateTime<Utc>,
    created_by: String,
}

impl IssueRow {
    fn into_issue(self) -> Issue {
        Issue {
            id: self.id,
            title: self.title,
            description: self.description,
            classification: self.classification,
            severity: self.severity,
            frequency: self.frequency,
            status: self.status,
            close_requires_artifact: self.close_requires_artifact,
            created_at: self.created_at,
            created_by: self.created_by,
        }
    }
}

#[derive(sqlx::FromRow)]
struct IssueSummaryRow {
    id: String,
    title: String,
    classification: String,
    severity: String,
    status: String,
    signal_count: i64,
    created_at: DateTime<Utc>,
}

impl IssueSummaryRow {
    fn into_summary(self) -> IssueSummary {
        IssueSummary {
            id: self.id,
            title: self.title,
            classification: self.classification,
            severity: self.severity,
            status: self.status,
            signal_count: self.signal_count,
            created_at: self.created_at,
        }
    }
}

// ===========================================================================
// DECISION REPOSITORY
// ===========================================================================

/// Insert a new decision (append-only). Decisions supersede previous decisions on the same issue.
pub async fn append_decision(
    pool: &PgPool,
    input: DecisionCreate,
    decided_by: &str,
) -> Result<Decision> {
    let decision_id = DecisionId::new();
    let mut tx = pool
        .begin()
        .await
        .context("begin decision append transaction")?;

    // Find current decision to supersede
    let supersedes_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM decisions WHERE issue_id = $1 ORDER BY decided_at DESC LIMIT 1",
    )
    .bind(&input.issue_id)
    .fetch_optional(&mut *tx)
    .await
    .context("failed to find existing decision")?;

    sqlx::query(
        r#"INSERT INTO decisions (id, issue_id, decision_type, rationale, decided_by, steward_deadline_days, supersedes_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(decision_id.as_str())
    .bind(&input.issue_id)
    .bind(input.decision_type.as_str())
    .bind(&input.rationale)
    .bind(decided_by)
    .bind(input.steward_deadline_days)
    .bind(supersedes_id.as_deref())
    .execute(&mut *tx)
    .await
    .context("failed to insert decision")?;

    // Transition issue to 'decided' status
    let current_issue_status: Option<String> = sqlx::query_scalar(
        "SELECT COALESCE((SELECT new_status FROM issue_status_history WHERE issue_id = $1 ORDER BY changed_at DESC, id DESC LIMIT 1), (SELECT status FROM issues WHERE id = $1))",
    )
    .bind(&input.issue_id)
    .fetch_optional(&mut *tx)
    .await
    .context("failed to get issue status")?;

    if let Some(current) = current_issue_status {
        if current == "pending_decision" {
            sqlx::query(
                "INSERT INTO issue_status_history (issue_id, old_status, new_status, changed_by, reason)
                VALUES ($1, $2, 'decided', $3, 'Decision recorded')",
            )
            .bind(&input.issue_id)
            .bind(&current)
            .bind(decided_by)
            .execute(&mut *tx)
            .await
            .context("failed to insert issue status history for decision")?;

            sqlx::query("UPDATE issues SET status = 'decided' WHERE id = $1")
                .bind(&input.issue_id)
                .execute(&mut *tx)
                .await
                .context("failed to update issue status to decided")?;
        }
    }

    tx.commit()
        .await
        .context("commit decision insert transaction")?;

    get_decision(pool, decision_id.as_str())
        .await?
        .ok_or_else(|| {
            anyhow!(
                "decision {} was inserted but could not be loaded",
                decision_id.as_str()
            )
        })
}

/// Fetch a single decision by ID.
pub async fn get_decision(pool: &PgPool, decision_id: &str) -> Result<Option<Decision>> {
    let row = sqlx::query_as::<_, DecisionRow>(
        r#"SELECT id, issue_id, decision_type, rationale, decided_by, decided_at, steward_deadline_days, supersedes_id
        FROM decisions WHERE id = $1"#,
    )
    .bind(decision_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch decision")?;

    Ok(row.map(|r| r.into_decision()))
}

/// List decisions for an issue, ordered by time (newest first).
pub async fn list_decisions_for_issue(
    pool: &PgPool,
    issue_id: &str,
) -> Result<Vec<DecisionHistoryEntry>> {
    let rows = sqlx::query_as::<_, DecisionHistoryRow>(
        r#"
        SELECT
            d.id,
            d.decision_type,
            d.rationale,
            d.decided_by,
            d.decided_at,
            (d.supersedes_id IS NULL AND NOT EXISTS(SELECT 1 FROM decisions d2 WHERE d2.supersedes_id = d.id)) AS is_current
        FROM decisions d
        WHERE d.issue_id = $1
        ORDER BY d.decided_at DESC
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
    .context("failed to list decisions for issue")?;

    Ok(rows.into_iter().map(|r| r.into_entry()).collect())
}

/// Get the current (latest) decision for an issue.
pub async fn get_current_decision_for_issue(
    pool: &PgPool,
    issue_id: &str,
) -> Result<Option<Decision>> {
    let row = sqlx::query_as::<_, DecisionRow>(
        r#"SELECT id, issue_id, decision_type, rationale, decided_by, decided_at, steward_deadline_days, supersedes_id
        FROM decisions WHERE issue_id = $1
        ORDER BY decided_at DESC LIMIT 1"#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch current decision")?;

    Ok(row.map(|r| r.into_decision()))
}

#[derive(sqlx::FromRow)]
struct DecisionRow {
    id: String,
    issue_id: String,
    decision_type: String,
    rationale: String,
    decided_by: String,
    decided_at: DateTime<Utc>,
    steward_deadline_days: i32,
    supersedes_id: Option<String>,
}

impl DecisionRow {
    fn into_decision(self) -> Decision {
        Decision {
            id: self.id,
            issue_id: self.issue_id,
            decision_type: self.decision_type,
            rationale: self.rationale,
            decided_by: self.decided_by,
            decided_at: self.decided_at,
            steward_deadline_days: self.steward_deadline_days,
            supersedes_id: self.supersedes_id,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DecisionHistoryRow {
    id: String,
    decision_type: String,
    rationale: String,
    decided_by: String,
    decided_at: DateTime<Utc>,
    is_current: bool,
}

impl DecisionHistoryRow {
    fn into_entry(self) -> DecisionHistoryEntry {
        DecisionHistoryEntry {
            id: self.id,
            decision_type: self.decision_type,
            rationale: self.rationale,
            decided_by: self.decided_by,
            decided_at: self.decided_at,
            is_current: self.is_current,
        }
    }
}

// ===========================================================================
// ARTIFACT REPOSITORY
// ===========================================================================

/// Insert a new artifact (append-only).
pub async fn append_artifact(
    pool: &PgPool,
    input: ArtifactCreate,
    created_by: &str,
) -> Result<Artifact> {
    let artifact_id = ArtifactId::new();

    sqlx::query(
        r#"INSERT INTO artifacts (id, issue_id, artifact_type, title, description, ref_url, note, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(artifact_id.as_str())
    .bind(&input.issue_id)
    .bind(input.artifact_type.as_str())
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.ref_url)
    .bind(&input.note)
    .bind(created_by)
    .execute(pool)
    .await
    .context("failed to insert artifact")?;

    get_artifact(pool, artifact_id.as_str())
        .await?
        .ok_or_else(|| {
            anyhow!(
                "artifact {} was inserted but could not be loaded",
                artifact_id.as_str()
            )
        })
}

/// Fetch a single artifact by ID.
pub async fn get_artifact(pool: &PgPool, artifact_id: &str) -> Result<Option<Artifact>> {
    let row = sqlx::query_as::<_, ArtifactRow>(
        r#"SELECT id, issue_id, artifact_type, title, description, ref_url, note, verified, verified_by, verified_at, created_at, created_by
        FROM artifacts WHERE id = $1"#,
    )
    .bind(artifact_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch artifact")?;

    Ok(row.map(|r| r.into_artifact()))
}

/// List artifacts for an issue.
pub async fn list_artifacts_for_issue(
    pool: &PgPool,
    issue_id: &str,
) -> Result<Vec<ArtifactSummary>> {
    let rows = sqlx::query_as::<_, ArtifactSummaryRow>(
        r#"SELECT id, artifact_type, title, verified, created_at
        FROM artifacts WHERE issue_id = $1
        ORDER BY created_at DESC"#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
    .context("failed to list artifacts for issue")?;

    Ok(rows.into_iter().map(|r| r.into_summary()).collect())
}

/// Check if an issue has any verified artifacts.
pub async fn has_verified_artifact(pool: &PgPool, issue_id: &str) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM artifacts WHERE issue_id = $1 AND verified = TRUE)",
    )
    .bind(issue_id)
    .fetch_one(pool)
    .await
    .context("failed to check for verified artifacts")?;

    Ok(exists)
}

/// Verify an artifact (Steward only). This is an UPDATE but only on the verified fields.
pub async fn verify_artifact(
    pool: &PgPool,
    artifact_id: &str,
    verified_by: &str,
) -> Result<Artifact> {
    let mut tx = pool.begin().await.context("begin artifact verification")?;

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM artifacts WHERE id = $1)")
        .bind(artifact_id)
        .fetch_one(&mut *tx)
        .await
        .context("failed to verify artifact existence")?;

    if !exists {
        return Err(anyhow!("Artifact not found: {}", artifact_id));
    }

    sqlx::query(
        "UPDATE artifacts SET verified = TRUE, verified_by = $1, verified_at = NOW() WHERE id = $2",
    )
    .bind(verified_by)
    .bind(artifact_id)
    .execute(&mut *tx)
    .await
    .context("failed to verify artifact")?;

    tx.commit().await.context("commit artifact verification")?;

    get_artifact(pool, artifact_id)
        .await?
        .ok_or_else(|| anyhow!("artifact {} disappeared after verification", artifact_id))
}

#[derive(sqlx::FromRow)]
struct ArtifactRow {
    id: String,
    issue_id: String,
    artifact_type: String,
    title: String,
    description: Option<String>,
    ref_url: Option<String>,
    note: Option<String>,
    verified: bool,
    verified_by: Option<String>,
    verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    created_by: String,
}

impl ArtifactRow {
    fn into_artifact(self) -> Artifact {
        Artifact {
            id: self.id,
            issue_id: self.issue_id,
            artifact_type: self.artifact_type,
            title: self.title,
            description: self.description,
            ref_url: self.ref_url,
            note: self.note,
            verified: self.verified,
            verified_by: self.verified_by,
            verified_at: self.verified_at,
            created_at: self.created_at,
            created_by: self.created_by,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ArtifactSummaryRow {
    id: String,
    artifact_type: String,
    title: String,
    verified: bool,
    created_at: DateTime<Utc>,
}

impl ArtifactSummaryRow {
    fn into_summary(self) -> ArtifactSummary {
        ArtifactSummary {
            id: self.id,
            artifact_type: self.artifact_type,
            title: self.title,
            verified: self.verified,
            created_at: self.created_at,
        }
    }
}

// ===========================================================================
// RESPONSE REPOSITORY
// ===========================================================================

/// Insert a new response (append-only).
pub async fn append_response(
    pool: &PgPool,
    input: ResponseCreate,
    drafted_by: &str,
) -> Result<Response> {
    let response_id = ResponseId::new();
    let mut tx = pool
        .begin()
        .await
        .context("begin response append transaction")?;

    sqlx::query(
        r#"INSERT INTO responses (id, signal_id, issue_id, response_class, channel, body, approval_state, drafted_by)
        VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7)"#,
    )
    .bind(response_id.as_str())
    .bind(&input.signal_id)
    .bind(&input.issue_id)
    .bind(&input.response_class)
    .bind(input.channel.as_str())
    .bind(&input.body)
    .bind(drafted_by)
    .execute(&mut *tx)
    .await
    .context("failed to insert response")?;

    sqlx::query(
        "INSERT INTO response_approval_history (response_id, new_state, changed_by, reason)
        VALUES ($1, 'draft', $2, 'Response drafted')",
    )
    .bind(response_id.as_str())
    .bind(drafted_by)
    .execute(&mut *tx)
    .await
    .context("failed to insert response approval history")?;

    tx.commit()
        .await
        .context("commit response insert transaction")?;

    get_response(pool, response_id.as_str())
        .await?
        .ok_or_else(|| {
            anyhow!(
                "response {} was inserted but could not be loaded",
                response_id.as_str()
            )
        })
}

/// Fetch a single response by ID.
pub async fn get_response(pool: &PgPool, response_id: &str) -> Result<Option<Response>> {
    let row = sqlx::query_as::<_, ResponseRow>(
        r#"SELECT id, signal_id, issue_id, response_class, channel, body,
            COALESCE(
                (SELECT new_state FROM response_approval_history WHERE response_id = responses.id ORDER BY changed_at DESC, id DESC LIMIT 1),
                approval_state
            ) AS approval_state,
            policy_violations, drafted_by, drafted_at, approved_by, approved_at, sent_at, blocked_reason
        FROM responses WHERE id = $1"#,
    )
    .bind(response_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch response")?;

    Ok(row.map(|r| r.into_response()))
}

/// List responses for a signal.
pub async fn list_responses_for_signal(
    pool: &PgPool,
    signal_id: &str,
) -> Result<Vec<ResponseSummary>> {
    let rows = sqlx::query_as::<_, ResponseSummaryRow>(
        r#"
        SELECT
            r.id,
            r.signal_id,
            r.channel,
            COALESCE(
                (SELECT new_state FROM response_approval_history WHERE response_id = r.id ORDER BY changed_at DESC, id DESC LIMIT 1),
                r.approval_state
            ) AS approval_state,
            r.drafted_at,
            jsonb_array_length(COALESCE(r.policy_violations, '[]'::jsonb)) > 0 AS has_violations
        FROM responses r
        WHERE r.signal_id = $1
        ORDER BY r.drafted_at DESC
        "#,
    )
    .bind(signal_id)
    .fetch_all(pool)
    .await
    .context("failed to list responses for signal")?;

    Ok(rows.into_iter().map(|r| r.into_summary()).collect())
}

/// Transition response approval state (append-only history).
pub async fn transition_response_state(
    pool: &PgPool,
    response_id: &str,
    new_state: ApprovalState,
    actor: &str,
    reason: Option<&str>,
    blocked_reason: Option<&str>,
) -> Result<Response> {
    let mut tx = pool
        .begin()
        .await
        .context("begin response state transition")?;

    let current_state: Option<String> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(
            (SELECT new_state FROM response_approval_history WHERE response_id = $1 ORDER BY changed_at DESC, id DESC LIMIT 1),
            (SELECT approval_state FROM responses WHERE id = $1)
        )
        "#,
    )
    .bind(response_id)
    .fetch_one(&mut *tx)
    .await
    .context("failed to read current response state")?;

    let current = current_state
        .as_deref()
        .and_then(ApprovalState::from_str)
        .ok_or_else(|| anyhow!("Response not found: {}", response_id))?;

    if !current.can_transition_to(new_state) {
        return Err(anyhow!(
            "Invalid approval state transition: {} -> {}",
            current.as_str(),
            new_state.as_str()
        ));
    }

    sqlx::query(
        "INSERT INTO response_approval_history (response_id, old_state, new_state, changed_by, reason)
        VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(response_id)
    .bind(current.as_str())
    .bind(new_state.as_str())
    .bind(actor)
    .bind(reason)
    .execute(&mut *tx)
    .await
    .context("failed to insert response approval history")?;

    // Update denormalized fields based on state
    match new_state {
        ApprovalState::Approved => {
            sqlx::query(
                "UPDATE responses SET approval_state = $1, approved_by = $2, approved_at = NOW() WHERE id = $3",
            )
            .bind(new_state.as_str())
            .bind(actor)
            .bind(response_id)
            .execute(&mut *tx)
            .await
            .context("failed to update response for approval")?;
        }
        ApprovalState::Sent => {
            sqlx::query("UPDATE responses SET approval_state = $1, sent_at = NOW() WHERE id = $2")
                .bind(new_state.as_str())
                .bind(response_id)
                .execute(&mut *tx)
                .await
                .context("failed to update response for sent")?;
        }
        ApprovalState::Blocked => {
            sqlx::query(
                "UPDATE responses SET approval_state = $1, blocked_reason = $2 WHERE id = $3",
            )
            .bind(new_state.as_str())
            .bind(blocked_reason)
            .bind(response_id)
            .execute(&mut *tx)
            .await
            .context("failed to update response for blocked")?;
        }
        _ => {
            sqlx::query("UPDATE responses SET approval_state = $1 WHERE id = $2")
                .bind(new_state.as_str())
                .bind(response_id)
                .execute(&mut *tx)
                .await
                .context("failed to update response state")?;
        }
    }

    tx.commit()
        .await
        .context("commit response state transition")?;

    get_response(pool, response_id)
        .await?
        .ok_or_else(|| anyhow!("response {} disappeared after transition", response_id))
}

#[derive(sqlx::FromRow)]
struct ResponseRow {
    id: String,
    signal_id: String,
    issue_id: Option<String>,
    response_class: String,
    channel: String,
    body: String,
    approval_state: String,
    policy_violations: Option<Value>,
    drafted_by: String,
    drafted_at: DateTime<Utc>,
    approved_by: Option<String>,
    approved_at: Option<DateTime<Utc>>,
    sent_at: Option<DateTime<Utc>>,
    blocked_reason: Option<String>,
}

impl ResponseRow {
    fn into_response(self) -> Response {
        let violations: Vec<String> = self
            .policy_violations
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Response {
            id: self.id,
            signal_id: self.signal_id,
            issue_id: self.issue_id,
            response_class: self.response_class,
            channel: self.channel,
            body: self.body,
            approval_state: self.approval_state,
            policy_violations: violations,
            drafted_by: self.drafted_by,
            drafted_at: self.drafted_at,
            approved_by: self.approved_by,
            approved_at: self.approved_at,
            sent_at: self.sent_at,
            blocked_reason: self.blocked_reason,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ResponseSummaryRow {
    id: String,
    signal_id: String,
    channel: String,
    approval_state: String,
    drafted_at: DateTime<Utc>,
    has_violations: bool,
}

impl ResponseSummaryRow {
    fn into_summary(self) -> ResponseSummary {
        ResponseSummary {
            id: self.id,
            signal_id: self.signal_id,
            channel: self.channel,
            approval_state: self.approval_state,
            drafted_at: self.drafted_at,
            has_violations: self.has_violations,
        }
    }
}
