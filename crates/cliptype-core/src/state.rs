//! Pure trigger, transition, and dispatch-decision policy.

use std::fmt;

use crate::TerminalOutcome;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationStage {
    CaptureTarget,
    WaitForModifiers,
    AcquireClipboard,
    BuildPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    Idle,
    Preparing(PreparationStage),
    Injecting,
    Cancelling,
    Finalizing(TerminalOutcome),
}

impl FlowState {
    pub const fn is_active(self) -> bool {
        !matches!(self, Self::Idle | Self::Finalizing(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowEvent {
    TriggerAccepted,
    TargetCaptured,
    ModifiersSettled,
    ClipboardAcquired,
    PlanReady,
    BatchAccepted,
    AllBatchesComplete,
    CancelRequested,
    CancellationObserved,
    Abort(TerminalOutcome),
    Finalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionError {
    pub state: FlowState,
    pub event: FlowEvent,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid transition from {:?} on {:?}",
            self.state, self.event
        )
    }
}

impl std::error::Error for TransitionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerDecision {
    Accept,
    Busy,
}

pub const fn decide_trigger(state: FlowState) -> TriggerDecision {
    match state {
        FlowState::Idle => TriggerDecision::Accept,
        FlowState::Preparing(_)
        | FlowState::Injecting
        | FlowState::Cancelling
        | FlowState::Finalizing(_) => TriggerDecision::Busy,
    }
}

pub fn transition(state: FlowState, event: FlowEvent) -> Result<FlowState, TransitionError> {
    if let FlowEvent::Abort(outcome) = event
        && state.is_active()
        && outcome != TerminalOutcome::Completed
    {
        return Ok(FlowState::Finalizing(outcome));
    }

    match (state, event) {
        (FlowState::Idle, FlowEvent::TriggerAccepted) => {
            Ok(FlowState::Preparing(PreparationStage::CaptureTarget))
        }
        (FlowState::Preparing(PreparationStage::CaptureTarget), FlowEvent::TargetCaptured) => {
            Ok(FlowState::Preparing(PreparationStage::WaitForModifiers))
        }
        (FlowState::Preparing(PreparationStage::WaitForModifiers), FlowEvent::ModifiersSettled) => {
            Ok(FlowState::Preparing(PreparationStage::AcquireClipboard))
        }
        (
            FlowState::Preparing(PreparationStage::AcquireClipboard),
            FlowEvent::ClipboardAcquired,
        ) => Ok(FlowState::Preparing(PreparationStage::BuildPlan)),
        (FlowState::Preparing(PreparationStage::BuildPlan), FlowEvent::PlanReady) => {
            Ok(FlowState::Injecting)
        }
        (FlowState::Injecting, FlowEvent::BatchAccepted) => Ok(FlowState::Injecting),
        (FlowState::Injecting, FlowEvent::AllBatchesComplete) => {
            Ok(FlowState::Finalizing(TerminalOutcome::Completed))
        }
        (FlowState::Preparing(_) | FlowState::Injecting, FlowEvent::CancelRequested) => {
            Ok(FlowState::Cancelling)
        }
        (FlowState::Cancelling, FlowEvent::CancelRequested) => Ok(FlowState::Cancelling),
        (FlowState::Cancelling, FlowEvent::CancellationObserved) => {
            Ok(FlowState::Finalizing(TerminalOutcome::Cancelled))
        }
        (FlowState::Finalizing(_), FlowEvent::Finalized) => Ok(FlowState::Idle),
        _ => Err(TransitionError { state, event }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoInputReason {
    KnownSecurityRestriction,
    BlockedCauseUnknown,
    NativeFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchObservation {
    CompleteBatch,
    NoEvents(NoInputReason),
    Partial,
    ProgressUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    Continue,
    Stop(TerminalOutcome),
}

pub const fn classify_dispatch(observation: DispatchObservation) -> DispatchDecision {
    match observation {
        DispatchObservation::CompleteBatch => DispatchDecision::Continue,
        DispatchObservation::NoEvents(NoInputReason::KnownSecurityRestriction) => {
            DispatchDecision::Stop(TerminalOutcome::KnownSecurityRestriction)
        }
        DispatchObservation::NoEvents(NoInputReason::BlockedCauseUnknown) => {
            DispatchDecision::Stop(TerminalOutcome::BlockedCauseUnknown)
        }
        DispatchObservation::NoEvents(NoInputReason::NativeFailure) => {
            DispatchDecision::Stop(TerminalOutcome::NativeFailure)
        }
        DispatchObservation::Partial => DispatchDecision::Stop(TerminalOutcome::PartialInput),
        DispatchObservation::ProgressUnknown => {
            DispatchDecision::Stop(TerminalOutcome::ProgressUnknown)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DispatchDecision, DispatchObservation, FlowEvent, FlowState, NoInputReason,
        PreparationStage, TriggerDecision, classify_dispatch, decide_trigger, transition,
    };
    use crate::{RetryDisposition, TerminalOutcome};

    #[test]
    fn follows_the_valid_happy_path() {
        let mut state = FlowState::Idle;
        for event in [
            FlowEvent::TriggerAccepted,
            FlowEvent::TargetCaptured,
            FlowEvent::ModifiersSettled,
            FlowEvent::ClipboardAcquired,
            FlowEvent::PlanReady,
            FlowEvent::BatchAccepted,
            FlowEvent::AllBatchesComplete,
            FlowEvent::Finalized,
        ] {
            state = transition(state, event).expect("valid transition");
        }

        assert_eq!(state, FlowState::Idle);
    }

    #[test]
    fn non_idle_trigger_is_busy() {
        assert_eq!(decide_trigger(FlowState::Idle), TriggerDecision::Accept);
        assert_eq!(
            decide_trigger(FlowState::Preparing(PreparationStage::CaptureTarget)),
            TriggerDecision::Busy
        );
        assert_eq!(decide_trigger(FlowState::Injecting), TriggerDecision::Busy);
    }

    #[test]
    fn cancellation_and_abort_finalize_before_idle() {
        let cancelling = transition(FlowState::Injecting, FlowEvent::CancelRequested)
            .expect("cancel request is valid");
        let finalizing = transition(cancelling, FlowEvent::CancellationObserved)
            .expect("cancellation observation is valid");

        assert_eq!(
            finalizing,
            FlowState::Finalizing(TerminalOutcome::Cancelled)
        );
        assert_eq!(
            transition(finalizing, FlowEvent::Finalized),
            Ok(FlowState::Idle)
        );

        assert_eq!(
            transition(
                FlowState::Injecting,
                FlowEvent::Abort(TerminalOutcome::TargetChanged),
            ),
            Ok(FlowState::Finalizing(TerminalOutcome::TargetChanged))
        );
    }

    #[test]
    fn invalid_sequences_are_rejected() {
        assert!(transition(FlowState::Idle, FlowEvent::PlanReady).is_err());
        assert!(
            transition(
                FlowState::Preparing(PreparationStage::CaptureTarget),
                FlowEvent::AllBatchesComplete,
            )
            .is_err()
        );
    }

    #[test]
    fn partial_and_unknown_dispatch_stop_without_retry() {
        for (observation, outcome) in [
            (DispatchObservation::Partial, TerminalOutcome::PartialInput),
            (
                DispatchObservation::ProgressUnknown,
                TerminalOutcome::ProgressUnknown,
            ),
            (
                DispatchObservation::NoEvents(NoInputReason::BlockedCauseUnknown),
                TerminalOutcome::BlockedCauseUnknown,
            ),
        ] {
            assert_eq!(
                classify_dispatch(observation),
                DispatchDecision::Stop(outcome)
            );
            assert_eq!(outcome.retry_disposition(), RetryDisposition::Never);
        }
    }
}
