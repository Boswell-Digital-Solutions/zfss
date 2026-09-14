-- ZFSS Initial Schema - PostgreSQL
-- NON-NEGOTIABLE: No UPDATE or DELETE on canonical records
-- Lifecycle changes use INSERT-only history/event tables; canonical rows stay immutable

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ===========================================================================
-- USERS / ROLES - Local role mapping
-- ===========================================================================
CREATE TABLE users (
    id              VARCHAR(100) PRIMARY KEY,
    display_name    VARCHAR(255) NOT NULL,
    role            VARCHAR(20) NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT user_role_valid CHECK (role IN ('Steward', 'Operator', 'Engineer', 'AI'))
);

CREATE INDEX idx_users_role ON users(role);

-- ===========================================================================
-- ISSUES - System's understanding (must be created before signals for FK)
-- ===========================================================================
CREATE TABLE issues (
    id                      VARCHAR(32) PRIMARY KEY,
    title                   VARCHAR(500) NOT NULL,
    description             TEXT,
    classification          VARCHAR(20) NOT NULL,
    severity                VARCHAR(20) NOT NULL DEFAULT 'medium',
    frequency               INTEGER NOT NULL DEFAULT 1,
    status                  VARCHAR(30) NOT NULL DEFAULT 'pending_decision',
    close_requires_artifact BOOLEAN NOT NULL DEFAULT TRUE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by              VARCHAR(100) NOT NULL,

    CONSTRAINT issue_id_pattern CHECK (id ~ '^iss_[a-zA-Z0-9]{16,}$'),
    CONSTRAINT issue_classification_valid CHECK (classification IN ('Bug', 'UX', 'Feature', 'Limitation')),
    CONSTRAINT issue_severity_valid CHECK (severity IN ('blocker', 'major', 'minor', 'idea')),
    CONSTRAINT issue_status_valid CHECK (status IN (
        'pending_decision', 'decided', 'in_progress', 'ready_for_verification', 'closed'
    ))
);

CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_classification ON issues(classification);
CREATE INDEX idx_issues_severity ON issues(severity);
CREATE INDEX idx_issues_created_at ON issues(created_at DESC);

-- Issue status history (append-only)
CREATE TABLE issue_status_history (
    id              BIGSERIAL PRIMARY KEY,
    issue_id        VARCHAR(32) NOT NULL REFERENCES issues(id),
    old_status      VARCHAR(30),
    new_status      VARCHAR(30) NOT NULL,
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by      VARCHAR(100) NOT NULL,
    reason          TEXT
);

CREATE INDEX idx_issue_status_history_issue ON issue_status_history(issue_id);

-- ===========================================================================
-- SIGNALS - Raw immutable user expression
-- ===========================================================================
CREATE TABLE signals (
    id              VARCHAR(32) PRIMARY KEY,
    source          VARCHAR(50) NOT NULL,
    raw_text        TEXT NOT NULL,
    app_key         VARCHAR(100),
    app_version     VARCHAR(50),
    environment     JSONB,
    reporter        JSONB,
    status          VARCHAR(20) NOT NULL DEFAULT 'new',
    linked_issue_id VARCHAR(32) REFERENCES issues(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      VARCHAR(100) NOT NULL,

    CONSTRAINT signal_id_pattern CHECK (id ~ '^sig_[a-zA-Z0-9]{16,}$'),
    CONSTRAINT signal_source_valid CHECK (source IN ('in_app', 'email', 'dm', 'call', 'internal', 'partner', 'monitoring')),
    CONSTRAINT signal_status_valid CHECK (status IN ('new', 'linked', 'needs_info', 'responded', 'closed'))
);

CREATE INDEX idx_signals_status ON signals(status);
CREATE INDEX idx_signals_linked_issue ON signals(linked_issue_id);
CREATE INDEX idx_signals_created_at ON signals(created_at DESC);
CREATE INDEX idx_signals_app_key ON signals(app_key);
CREATE INDEX idx_signals_source ON signals(source);

-- Signal status history (append-only)
CREATE TABLE signal_status_history (
    id              BIGSERIAL PRIMARY KEY,
    signal_id       VARCHAR(32) NOT NULL REFERENCES signals(id),
    old_status      VARCHAR(20),
    new_status      VARCHAR(20) NOT NULL,
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by      VARCHAR(100) NOT NULL,
    reason          TEXT
);

CREATE INDEX idx_signal_status_history_signal ON signal_status_history(signal_id);

-- ===========================================================================
-- ATTACHMENTS - Files attached to signals
-- ===========================================================================
CREATE TABLE attachments (
    id              VARCHAR(32) PRIMARY KEY,
    signal_id       VARCHAR(32) NOT NULL REFERENCES signals(id),
    kind            VARCHAR(20) NOT NULL,
    filename        VARCHAR(255),
    ref_url         VARCHAR(1000),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT attachment_id_pattern CHECK (id ~ '^att_[a-zA-Z0-9]{16,}$'),
    CONSTRAINT attachment_kind_valid CHECK (kind IN ('screenshot', 'log', 'video', 'file', 'link'))
);

CREATE INDEX idx_attachments_signal ON attachments(signal_id);

-- ===========================================================================
-- DECISIONS - Declared intent (append-only)
-- ===========================================================================
CREATE TABLE decisions (
    id              VARCHAR(32) PRIMARY KEY,
    issue_id        VARCHAR(32) NOT NULL REFERENCES issues(id),
    decision_type   VARCHAR(30) NOT NULL,
    rationale       TEXT NOT NULL,
    decided_by      VARCHAR(100) NOT NULL,
    decided_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    steward_deadline_days INTEGER NOT NULL DEFAULT 7,
    supersedes_id   VARCHAR(32) REFERENCES decisions(id),

    CONSTRAINT decision_id_pattern CHECK (id ~ '^dec_[a-zA-Z0-9]{16,}$'),
    CONSTRAINT decision_type_valid CHECK (decision_type IN (
        'FixNow', 'FixLater', 'DocumentClarify', 'WontFix', 'DeEscalate'
    )),
    CONSTRAINT decision_rationale_length CHECK (char_length(rationale) >= 10)
);

CREATE INDEX idx_decisions_issue ON decisions(issue_id);
CREATE INDEX idx_decisions_type ON decisions(decision_type);
CREATE INDEX idx_decisions_decided_at ON decisions(decided_at DESC);

-- ===========================================================================
-- ARTIFACTS - Proof of learning
-- ===========================================================================
CREATE TABLE artifacts (
    id              VARCHAR(32) PRIMARY KEY,
    issue_id        VARCHAR(32) NOT NULL REFERENCES issues(id),
    artifact_type   VARCHAR(20) NOT NULL,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    ref_url         VARCHAR(1000),
    note            TEXT,
    verified        BOOLEAN NOT NULL DEFAULT FALSE,
    verified_by     VARCHAR(100),
    verified_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      VARCHAR(100) NOT NULL,

    CONSTRAINT artifact_id_pattern CHECK (id ~ '^art_[a-zA-Z0-9]{16,}$'),
    CONSTRAINT artifact_type_valid CHECK (artifact_type IN (
        'Code', 'Logic', 'Knowledge', 'Test', 'Law'
    ))
);

CREATE INDEX idx_artifacts_issue ON artifacts(issue_id);
CREATE INDEX idx_artifacts_type ON artifacts(artifact_type);
CREATE INDEX idx_artifacts_verified ON artifacts(verified);

-- ===========================================================================
-- RESPONSES - Controlled outward communication
-- ===========================================================================
CREATE TABLE responses (
    id              VARCHAR(32) PRIMARY KEY,
    signal_id       VARCHAR(32) NOT NULL REFERENCES signals(id),
    issue_id        VARCHAR(32) REFERENCES issues(id),
    response_class  VARCHAR(100) NOT NULL,
    channel         VARCHAR(20) NOT NULL,
    body            TEXT NOT NULL,
    approval_state  VARCHAR(20) NOT NULL DEFAULT 'draft',
    policy_violations JSONB DEFAULT '[]'::jsonb,
    drafted_by      VARCHAR(100) NOT NULL,
    drafted_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_by     VARCHAR(100),
    approved_at     TIMESTAMPTZ,
    sent_at         TIMESTAMPTZ,
    blocked_reason  TEXT,

    CONSTRAINT response_id_pattern CHECK (id ~ '^rsp_[a-zA-Z0-9]{16,}$'),
    CONSTRAINT response_channel_valid CHECK (channel IN ('email', 'in_app', 'dm', 'phone', 'other')),
    CONSTRAINT response_approval_state_valid CHECK (approval_state IN (
        'draft', 'pending', 'approved', 'sent', 'blocked'
    ))
);

CREATE INDEX idx_responses_signal ON responses(signal_id);
CREATE INDEX idx_responses_issue ON responses(issue_id);
CREATE INDEX idx_responses_approval_state ON responses(approval_state);
CREATE INDEX idx_responses_drafted_at ON responses(drafted_at DESC);

-- Response approval history (append-only)
CREATE TABLE response_approval_history (
    id              BIGSERIAL PRIMARY KEY,
    response_id     VARCHAR(32) NOT NULL REFERENCES responses(id),
    old_state       VARCHAR(20),
    new_state       VARCHAR(20) NOT NULL,
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by      VARCHAR(100) NOT NULL,
    reason          TEXT
);

CREATE INDEX idx_response_approval_history_response ON response_approval_history(response_id);

-- ===========================================================================
-- AUDIT LOG - Append-only system audit trail
-- ===========================================================================
CREATE TABLE audit_log (
    id              BIGSERIAL PRIMARY KEY,
    event_type      VARCHAR(50) NOT NULL,
    entity_type     VARCHAR(20) NOT NULL,
    entity_id       VARCHAR(32) NOT NULL,
    actor           VARCHAR(100) NOT NULL,
    actor_role      VARCHAR(20) NOT NULL,
    action          VARCHAR(50) NOT NULL,
    details         JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_actor ON audit_log(actor);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);

-- ===========================================================================
-- VIEWS for common queries
-- ===========================================================================

-- Open issues with signal count
CREATE VIEW v_issues_with_signal_count AS
SELECT
    i.*,
    COUNT(s.id) AS signal_count,
    MAX(s.created_at) AS last_signal_at
FROM issues i
LEFT JOIN signals s ON s.linked_issue_id = i.id
GROUP BY i.id;

-- Issues pending closure (have verified artifacts)
CREATE VIEW v_issues_ready_for_closure AS
SELECT i.*
FROM issues i
WHERE i.status = 'ready_for_verification'
  AND i.close_requires_artifact = TRUE
  AND EXISTS (
      SELECT 1 FROM artifacts a WHERE a.issue_id = i.id AND a.verified = TRUE
  );

-- Signals awaiting triage (new, unlinked)
CREATE VIEW v_signals_pending_triage AS
SELECT s.*
FROM signals s
WHERE s.status = 'new'
  AND s.linked_issue_id IS NULL
ORDER BY s.created_at DESC;

-- Insert default Steward user
INSERT INTO users (id, display_name, role, is_active)
VALUES ('system', 'System', 'Steward', TRUE)
ON CONFLICT (id) DO NOTHING;
