//! Signal model - Raw immutable user expression
//!
//! Signals are the entry point for all feedback. Canonical rows are immutable;
//! status transitions and issue links are appended as events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Signal source channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalSource {
    InApp,
    Email,
    Dm,
    Call,
    Internal,
    Partner,
    Monitoring,
}

impl SignalSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalSource::InApp => "in_app",
            SignalSource::Email => "email",
            SignalSource::Dm => "dm",
            SignalSource::Call => "call",
            SignalSource::Internal => "internal",
            SignalSource::Partner => "partner",
            SignalSource::Monitoring => "monitoring",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_app" => Some(SignalSource::InApp),
            "email" => Some(SignalSource::Email),
            "dm" => Some(SignalSource::Dm),
            "call" => Some(SignalSource::Call),
            "internal" => Some(SignalSource::Internal),
            "partner" => Some(SignalSource::Partner),
            "monitoring" => Some(SignalSource::Monitoring),
            _ => None,
        }
    }
}

/// Signal status in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStatus {
    New,
    Linked,
    NeedsInfo,
    Responded,
    Closed,
}

impl SignalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalStatus::New => "new",
            SignalStatus::Linked => "linked",
            SignalStatus::NeedsInfo => "needs_info",
            SignalStatus::Responded => "responded",
            SignalStatus::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "new" => Some(SignalStatus::New),
            "linked" => Some(SignalStatus::Linked),
            "needs_info" => Some(SignalStatus::NeedsInfo),
            "responded" => Some(SignalStatus::Responded),
            "closed" => Some(SignalStatus::Closed),
            _ => None,
        }
    }

    /// Valid transitions for Signal lifecycle
    pub fn can_transition_to(&self, next: SignalStatus) -> bool {
        matches!(
            (self, next),
            (SignalStatus::New, SignalStatus::Linked)
                | (SignalStatus::New, SignalStatus::NeedsInfo)
                | (SignalStatus::Linked, SignalStatus::Responded)
                | (SignalStatus::Linked, SignalStatus::NeedsInfo)
                | (SignalStatus::NeedsInfo, SignalStatus::Linked)
                | (SignalStatus::NeedsInfo, SignalStatus::New)
                | (SignalStatus::Responded, SignalStatus::Closed)
        )
    }
}

/// Environment context for a signal
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalEnvironment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

/// Reporter information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalReporter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
}

/// Full Signal record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub source: String,
    pub raw_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<SignalEnvironment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporter: Option<SignalReporter>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_issue_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

/// Input for creating a new signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCreate {
    pub source: SignalSource,
    pub raw_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<SignalEnvironment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporter: Option<SignalReporter>,
}

/// Signal with attachment count (for list views)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSummary {
    pub id: String,
    pub source: String,
    pub raw_text: String,
    pub status: String,
    pub linked_issue_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub attachment_count: i64,
}

#[cfg(test)]
mod tests {
    use super::SignalStatus;

    #[test]
    fn signal_transition_matrix_is_fail_closed() {
        use SignalStatus::*;
        let states = [New, Linked, NeedsInfo, Responded, Closed];
        let allowed = [
            (New, Linked),
            (New, NeedsInfo),
            (Linked, Responded),
            (Linked, NeedsInfo),
            (NeedsInfo, Linked),
            (NeedsInfo, New),
            (Responded, Closed),
        ];

        for current in states {
            for next in states {
                assert_eq!(
                    current.can_transition_to(next),
                    allowed.contains(&(current, next)),
                    "unexpected signal transition: {} -> {}",
                    current.as_str(),
                    next.as_str()
                );
            }
        }
    }
}
