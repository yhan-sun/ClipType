use super::completion_code;
use cliptype_app::SessionCompletion;
use cliptype_core::{PreparationFailure, TerminalOutcome};

#[test]
fn preserves_every_terminal_outcome_without_generic_failure() {
    let cases = [
        (TerminalOutcome::Completed, 1),
        (TerminalOutcome::Cancelled, 2),
        (TerminalOutcome::TargetChanged, 3),
        (TerminalOutcome::ClipboardChanged, 4),
        (TerminalOutcome::KnownSecurityRestriction, 5),
        (TerminalOutcome::ModifierConflict, 7),
        (TerminalOutcome::TargetEvidenceUnavailable, 8),
        (TerminalOutcome::TargetDisappeared, 9),
        (TerminalOutcome::PartialInput, 10),
        (TerminalOutcome::ProgressUnknown, 11),
        (TerminalOutcome::BlockedCauseUnknown, 12),
        (TerminalOutcome::NativeFailure, 13),
        (TerminalOutcome::InternalInvariant, 14),
        (TerminalOutcome::ModifierSettleTimeout, 15),
    ];
    let mut seen = std::collections::BTreeSet::new();
    assert_eq!(completion_code(None), 0);
    for (outcome, expected) in cases {
        let code = completion_code(Some(SessionCompletion::Finished(outcome)));
        assert_eq!(code, expected);
        assert!(seen.insert(code), "terminal reasons must not collapse");
    }
}

#[test]
fn modifier_timeout_survives_both_preparation_and_execution() {
    assert_eq!(
        completion_code(Some(SessionCompletion::PreparationFailed(
            PreparationFailure::ModifierSettleTimeout,
        ))),
        15
    );
    assert_eq!(
        completion_code(Some(SessionCompletion::PreparationFailed(
            PreparationFailure::Cancelled,
        ))),
        2
    );
}
