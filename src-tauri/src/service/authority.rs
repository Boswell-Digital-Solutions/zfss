//! Central role-authority checks for mutating IPC commands.

use crate::models::UserRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityAction {
    LinkSignal,
    CreateIssue,
    MakeDecision,
    CreateArtifact,
    VerifyArtifact,
    DraftResponse,
    ApproveResponse,
    CloseIssue,
}

impl AuthorityAction {
    fn label(self) -> &'static str {
        match self {
            Self::LinkSignal => "link signals",
            Self::CreateIssue => "create issues",
            Self::MakeDecision => "make decisions",
            Self::CreateArtifact => "create artifacts",
            Self::VerifyArtifact => "verify artifacts",
            Self::DraftResponse => "draft responses",
            Self::ApproveResponse => "approve or block responses",
            Self::CloseIssue => "close issues",
        }
    }
}

pub fn require_authority(role: UserRole, action: AuthorityAction) -> Result<(), String> {
    let allowed = match action {
        AuthorityAction::LinkSignal => role.can_link_signal(),
        AuthorityAction::CreateIssue => role.can_create_issue(),
        AuthorityAction::MakeDecision => role.can_make_decision(),
        AuthorityAction::CreateArtifact => role.can_create_artifact(),
        AuthorityAction::VerifyArtifact => role.can_verify_artifact(),
        AuthorityAction::DraftResponse => role.can_draft_response(),
        AuthorityAction::ApproveResponse => role.can_approve_response(),
        AuthorityAction::CloseIssue => role.can_close_issue(),
    };

    if allowed {
        Ok(())
    } else {
        Err(format!(
            "Permission denied: role {} cannot {}",
            role.as_str(),
            action.label()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthorityAction, require_authority};
    use crate::models::UserRole;

    #[test]
    fn ipc_authority_matrix_is_fail_closed() {
        use AuthorityAction::*;
        use UserRole::*;

        let actions = [
            LinkSignal,
            CreateIssue,
            MakeDecision,
            CreateArtifact,
            VerifyArtifact,
            DraftResponse,
            ApproveResponse,
            CloseIssue,
        ];
        let allowed = [
            (Steward, LinkSignal),
            (Steward, CreateIssue),
            (Steward, MakeDecision),
            (Steward, CreateArtifact),
            (Steward, VerifyArtifact),
            (Steward, DraftResponse),
            (Steward, ApproveResponse),
            (Steward, CloseIssue),
            (Operator, LinkSignal),
            (Operator, CreateIssue),
            (Operator, DraftResponse),
            (Engineer, CreateArtifact),
            (AI, DraftResponse),
        ];

        for role in [Steward, Operator, Engineer, AI] {
            for action in actions {
                assert_eq!(
                    require_authority(role, action).is_ok(),
                    allowed.contains(&(role, action)),
                    "authority drift for {} and {:?}",
                    role.as_str(),
                    action
                );
            }
        }
    }
}
