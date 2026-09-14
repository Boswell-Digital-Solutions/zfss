//! Issue model - System's understanding of feedback
//!
//! Issues aggregate multiple signals into a coherent understanding.
//! close_requires_artifact is IMMUTABLE once set.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Controlled vocabulary for issue classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    Bug,
    UX,
    Feature,
    Limitation,
}

impl Classification {
    pub fn as_str(&self) -> &'static str {
        match self {
            Classification::Bug => "Bug",
            Classification::UX => "UX",
            Classification::Feature => "Feature",
            Classification::Limitation => "Limitation",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Bug" => Some(Classification::Bug),
            "UX" => Some(Classification::UX),
            "Feature" => Some(Classification::Feature),
            "Limitation" => Some(Classification::Limitation),
            _ => None,
        }
    }
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Blocker,
    Major,
    Minor,
    Idea,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Blocker => "blocker",
            Severity::Major => "major",
            Severity::Minor => "minor",
            Severity::Idea => "idea",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "blocker" => Some(Severity::Blocker),
            "major" => Some(Severity::Major),
            "minor" => Some(Severity::Minor),
            "idea" => Some(Severity::Idea),
            _ => None,
        }
    }
}

/// Frequency indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    One,
    Some,
    Many,
}

impl Frequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Frequency::One => "one",
            Frequency::Some => "some",
            Frequency::Many => "many",
        }
    }

    /// Convert signal count to frequency
    pub fn from_count(count: i32) -> Self {
        match count {
            0..=1 => Frequency::One,
            2..=5 => Frequency::Some,
            _ => Frequency::Many,
        }
    }
}

/// Issue status in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    PendingDecision,
    Decided,
    InProgress,
    ReadyForVerification,
    Closed,
}

impl IssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueStatus::PendingDecision => "pending_decision",
            IssueStatus::Decided => "decided",
            IssueStatus::InProgress => "in_progress",
            IssueStatus::ReadyForVerification => "ready_for_verification",
            IssueStatus::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending_decision" => Some(IssueStatus::PendingDecision),
            "decided" => Some(IssueStatus::Decided),
            "in_progress" => Some(IssueStatus::InProgress),
            "ready_for_verification" => Some(IssueStatus::ReadyForVerification),
            "closed" => Some(IssueStatus::Closed),
            _ => None,
        }
    }

    /// Valid transitions for Issue lifecycle
    pub fn can_transition_to(&self, next: IssueStatus) -> bool {
        matches!(
            (self, next),
            (IssueStatus::PendingDecision, IssueStatus::Decided)
                | (IssueStatus::Decided, IssueStatus::InProgress)
                | (IssueStatus::InProgress, IssueStatus::ReadyForVerification)
                | (IssueStatus::ReadyForVerification, IssueStatus::Closed)
                | (IssueStatus::ReadyForVerification, IssueStatus::InProgress) // rework
        )
    }
}

/// Full Issue record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub classification: String,
    pub severity: String,
    pub frequency: i32,
    pub status: String,
    pub close_requires_artifact: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

/// Input for creating a new issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCreate {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub classification: Classification,
    pub severity: Severity,
}

/// Issue with linked signal count (for list views)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    pub id: String,
    pub title: String,
    pub classification: String,
    pub severity: String,
    pub status: String,
    pub signal_count: i64,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::IssueStatus;

    #[test]
    fn issue_transition_matrix_is_fail_closed() {
        use IssueStatus::*;
        let states = [
            PendingDecision,
            Decided,
            InProgress,
            ReadyForVerification,
            Closed,
        ];
        let allowed = [
            (PendingDecision, Decided),
            (Decided, InProgress),
            (InProgress, ReadyForVerification),
            (ReadyForVerification, Closed),
            (ReadyForVerification, InProgress),
        ];

        for current in states {
            for next in states {
                assert_eq!(
                    current.can_transition_to(next),
                    allowed.contains(&(current, next)),
                    "unexpected issue transition: {} -> {}",
                    current.as_str(),
                    next.as_str()
                );
            }
        }
    }
}
