-- Project lifecycle state from immutable events instead of mutating canonical rows.

CREATE TABLE artifact_verifications (
    id BIGSERIAL PRIMARY KEY,
    artifact_id VARCHAR(32) NOT NULL REFERENCES artifacts(id),
    verified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_by VARCHAR(100) NOT NULL,
    reason TEXT,
    CONSTRAINT artifact_verification_once UNIQUE (artifact_id)
);

CREATE INDEX idx_artifact_verifications_artifact ON artifact_verifications(artifact_id);

ALTER TABLE response_approval_history ADD COLUMN blocked_reason TEXT;

-- Event/history rows are part of the audit chain and are immutable too.
CREATE TRIGGER trg_issue_status_history_prevent_mutation
BEFORE UPDATE OR DELETE ON issue_status_history
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

CREATE TRIGGER trg_signal_status_history_prevent_mutation
BEFORE UPDATE OR DELETE ON signal_status_history
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

CREATE TRIGGER trg_response_approval_history_prevent_mutation
BEFORE UPDATE OR DELETE ON response_approval_history
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

CREATE TRIGGER trg_signal_links_prevent_mutation
BEFORE UPDATE OR DELETE ON signal_links
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

CREATE TRIGGER trg_artifact_verifications_prevent_mutation
BEFORE UPDATE OR DELETE ON artifact_verifications
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

CREATE TRIGGER trg_attachments_prevent_mutation
BEFORE UPDATE OR DELETE ON attachments
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

CREATE TRIGGER trg_audit_log_prevent_mutation
BEFORE UPDATE OR DELETE ON audit_log
FOR EACH ROW EXECUTE FUNCTION zfss_forbid_mutation();

DROP VIEW v_issues_with_signal_count;
CREATE VIEW v_issues_with_signal_count AS
SELECT i.*, COALESCE(latest_status.new_status, i.status) AS current_status,
       COUNT(latest_link.signal_id) AS signal_count,
       MAX(latest_link.linked_at) AS last_signal_at
FROM issues i
LEFT JOIN LATERAL (
    SELECT h.new_status FROM issue_status_history h WHERE h.issue_id = i.id
    ORDER BY h.changed_at DESC, h.id DESC LIMIT 1
) latest_status ON TRUE
LEFT JOIN LATERAL (
    SELECT DISTINCT ON (l.signal_id) l.signal_id, l.linked_at
    FROM signal_links l WHERE l.issue_id = i.id
    ORDER BY l.signal_id, l.linked_at DESC, l.id DESC
) latest_link ON TRUE
GROUP BY i.id, latest_status.new_status;

DROP VIEW v_issues_ready_for_closure;
CREATE VIEW v_issues_ready_for_closure AS
SELECT i.* FROM issues i
LEFT JOIN LATERAL (
    SELECT h.new_status FROM issue_status_history h WHERE h.issue_id = i.id
    ORDER BY h.changed_at DESC, h.id DESC LIMIT 1
) latest_status ON TRUE
WHERE COALESCE(latest_status.new_status, i.status) = 'ready_for_verification'
  AND i.close_requires_artifact = TRUE
  AND EXISTS (
      SELECT 1 FROM artifacts a
      JOIN artifact_verifications v ON v.artifact_id = a.id
      WHERE a.issue_id = i.id
  );

DROP VIEW v_signals_pending_triage;
CREATE VIEW v_signals_pending_triage AS
SELECT s.* FROM signals s
LEFT JOIN LATERAL (
    SELECT h.new_status FROM signal_status_history h WHERE h.signal_id = s.id
    ORDER BY h.changed_at DESC, h.id DESC LIMIT 1
) latest_status ON TRUE
WHERE COALESCE(latest_status.new_status, s.status) = 'new'
  AND NOT EXISTS (SELECT 1 FROM signal_links l WHERE l.signal_id = s.id)
ORDER BY s.created_at DESC;
