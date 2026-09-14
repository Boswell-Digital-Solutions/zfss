\set ON_ERROR_STOP on

BEGIN;

INSERT INTO signals (id, source, raw_text, status, created_by)
VALUES ('sig_ContractTest0001', 'internal', 'postgres contract test', 'new', 'system');

DO $$
DECLARE mutation_blocked BOOLEAN := FALSE;
BEGIN
    BEGIN
        UPDATE signals SET status = 'linked' WHERE id = 'sig_ContractTest0001';
    EXCEPTION WHEN OTHERS THEN
        IF SQLERRM LIKE 'ZFSS doctrine violation:%' THEN
            mutation_blocked := TRUE;
        ELSE
            RAISE;
        END IF;
    END;
    IF NOT mutation_blocked THEN
        RAISE EXCEPTION 'UPDATE was not blocked for a real canonical row';
    END IF;
END;
$$;

DO $$
DECLARE mutation_blocked BOOLEAN := FALSE;
BEGIN
    BEGIN
        DELETE FROM signals WHERE id = 'sig_ContractTest0001';
    EXCEPTION WHEN OTHERS THEN
        IF SQLERRM LIKE 'ZFSS doctrine violation:%' THEN
            mutation_blocked := TRUE;
        ELSE
            RAISE;
        END IF;
    END;
    IF NOT mutation_blocked THEN
        RAISE EXCEPTION 'DELETE was not blocked for a real canonical row';
    END IF;
END;
$$;

DO $$
DECLARE guard_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO guard_count
    FROM pg_trigger
    WHERE NOT tgisinternal
      AND tgname IN (
          'trg_issues_prevent_mutation',
          'trg_signals_prevent_mutation',
          'trg_decisions_prevent_mutation',
          'trg_artifacts_prevent_mutation',
          'trg_responses_prevent_mutation',
          'trg_issue_status_history_prevent_mutation',
          'trg_signal_status_history_prevent_mutation',
          'trg_response_approval_history_prevent_mutation',
          'trg_signal_links_prevent_mutation',
          'trg_artifact_verifications_prevent_mutation',
          'trg_attachments_prevent_mutation',
          'trg_audit_log_prevent_mutation'
      );
    IF guard_count <> 12 THEN
        RAISE EXCEPTION 'expected 12 append-only mutation guards, found %', guard_count;
    END IF;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM v_signals_pending_triage WHERE id = 'sig_ContractTest0001'
    ) THEN
        RAISE EXCEPTION 'new unlinked signal missing from triage view';
    END IF;
END;
$$;

INSERT INTO issues (id, title, classification, severity, status, created_by)
VALUES ('iss_ContractTest0001', 'contract issue', 'Bug', 'major', 'pending_decision', 'system');
INSERT INTO issue_status_history (issue_id, new_status, changed_by, reason)
VALUES ('iss_ContractTest0001', 'pending_decision', 'system', 'created');
INSERT INTO issue_status_history (issue_id, old_status, new_status, changed_by, reason)
VALUES ('iss_ContractTest0001', 'pending_decision', 'decided', 'system', 'decision recorded');

INSERT INTO artifacts (id, issue_id, artifact_type, title, created_by)
VALUES ('art_ContractTest0001', 'iss_ContractTest0001', 'Test', 'contract proof', 'system');
INSERT INTO artifact_verifications (artifact_id, verified_by, reason)
VALUES ('art_ContractTest0001', 'system', 'contract verification');

DO $$
DECLARE mutation_blocked BOOLEAN := FALSE;
BEGIN
    BEGIN
        UPDATE artifact_verifications SET reason = 'rewritten' WHERE artifact_id = 'art_ContractTest0001';
    EXCEPTION WHEN OTHERS THEN
        IF SQLERRM LIKE 'ZFSS doctrine violation:%' THEN
            mutation_blocked := TRUE;
        ELSE
            RAISE;
        END IF;
    END;
    IF NOT mutation_blocked THEN
        RAISE EXCEPTION 'UPDATE was not blocked for a lifecycle event';
    END IF;
END;
$$;

INSERT INTO responses (id, signal_id, issue_id, response_class, channel, body, drafted_by)
VALUES ('rsp_ContractTest0001', 'sig_ContractTest0001', 'iss_ContractTest0001', 'resolution', 'in_app', 'fixed', 'system');
INSERT INTO response_approval_history (response_id, new_state, changed_by, reason)
VALUES ('rsp_ContractTest0001', 'draft', 'system', 'drafted');
INSERT INTO response_approval_history (response_id, old_state, new_state, changed_by, reason)
VALUES ('rsp_ContractTest0001', 'draft', 'pending', 'system', 'submitted');
INSERT INTO response_approval_history (response_id, old_state, new_state, changed_by, reason)
VALUES ('rsp_ContractTest0001', 'pending', 'approved', 'system', 'approved');

DO $$
BEGIN
    IF (SELECT status FROM issues WHERE id = 'iss_ContractTest0001') <> 'pending_decision' THEN
        RAISE EXCEPTION 'canonical issue row was mutated';
    END IF;
    IF (SELECT new_status FROM issue_status_history WHERE issue_id = 'iss_ContractTest0001' ORDER BY id DESC LIMIT 1) <> 'decided' THEN
        RAISE EXCEPTION 'latest issue state was not projected from history';
    END IF;
    IF (SELECT verified FROM artifacts WHERE id = 'art_ContractTest0001') THEN
        RAISE EXCEPTION 'canonical artifact row was mutated';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM artifact_verifications WHERE artifact_id = 'art_ContractTest0001') THEN
        RAISE EXCEPTION 'artifact verification event missing';
    END IF;
    IF (SELECT approval_state FROM responses WHERE id = 'rsp_ContractTest0001') <> 'draft' THEN
        RAISE EXCEPTION 'canonical response row was mutated';
    END IF;
    IF (SELECT new_state FROM response_approval_history WHERE response_id = 'rsp_ContractTest0001' ORDER BY id DESC LIMIT 1) <> 'approved' THEN
        RAISE EXCEPTION 'latest response state was not projected from history';
    END IF;
END;
$$;

ROLLBACK;
