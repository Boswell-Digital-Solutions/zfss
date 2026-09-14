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
          'trg_responses_prevent_mutation'
      );
    IF guard_count <> 5 THEN
        RAISE EXCEPTION 'expected 5 canonical mutation guards, found %', guard_count;
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

ROLLBACK;
