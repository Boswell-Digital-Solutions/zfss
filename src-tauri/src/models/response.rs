//! Response model - Controlled outward communication
//!
//! Responses follow an approval workflow:
//! draft -> pending -> approved -> sent (or blocked)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response communication channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseChannel {
    Email,
    InApp,
    Dm,
    Phone,
    Other,
}

impl ResponseChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseChannel::Email => "email",
            ResponseChannel::InApp => "in_app",
            ResponseChannel::Dm => "dm",
            ResponseChannel::Phone => "phone",
            ResponseChannel::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "email" => Some(ResponseChannel::Email),
            "in_app" => Some(ResponseChannel::InApp),
            "dm" => Some(ResponseChannel::Dm),
            "phone" => Some(ResponseChannel::Phone),
            "other" => Some(ResponseChannel::Other),
            _ => None,
        }
    }
}

/// Response approval state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Draft,
    Pending,
    Approved,
    Sent,
    Blocked,
}

impl ApprovalState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalState::Draft => "draft",
            ApprovalState::Pending => "pending",
            ApprovalState::Approved => "approved",
            ApprovalState::Sent => "sent",
            ApprovalState::Blocked => "blocked",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(ApprovalState::Draft),
            "pending" => Some(ApprovalState::Pending),
            "approved" => Some(ApprovalState::Approved),
            "sent" => Some(ApprovalState::Sent),
            "blocked" => Some(ApprovalState::Blocked),
            _ => None,
        }
    }

    /// Valid transitions for Response approval workflow
    pub fn can_transition_to(&self, next: ApprovalState) -> bool {
        matches!(
            (self, next),
            (ApprovalState::Draft, ApprovalState::Pending)
                | (ApprovalState::Pending, ApprovalState::Approved)
                | (ApprovalState::Pending, ApprovalState::Blocked)
                | (ApprovalState::Approved, ApprovalState::Sent)
                | (ApprovalState::Blocked, ApprovalState::Draft) // can revise
        )
    }
}

/// Full Response record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub signal_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    pub response_class: String,
    pub channel: String,
    pub body: String,
    pub approval_state: String,
    #[serde(default)]
    pub policy_violations: Vec<String>,
    pub drafted_by: String,
    pub drafted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
}

/// Input for creating a new response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCreate {
    pub signal_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    pub response_class: String,
    pub channel: ResponseChannel,
    pub body: String,
}

/// Response summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSummary {
    pub id: String,
    pub signal_id: String,
    pub channel: String,
    pub approval_state: String,
    pub drafted_at: DateTime<Utc>,
    pub has_violations: bool,
}

#[cfg(test)]
mod tests {
    use super::ApprovalState;

    #[test]
    fn response_transition_matrix_is_fail_closed() {
        use ApprovalState::*;
        let states = [Draft, Pending, Approved, Sent, Blocked];
        let allowed = [
            (Draft, Pending),
            (Pending, Approved),
            (Pending, Blocked),
            (Approved, Sent),
            (Blocked, Draft),
        ];

        for current in states {
            for next in states {
                assert_eq!(
                    current.can_transition_to(next),
                    allowed.contains(&(current, next)),
                    "unexpected response transition: {} -> {}",
                    current.as_str(),
                    next.as_str()
                );
            }
        }
    }
}
