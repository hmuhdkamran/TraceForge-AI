use contractguard_lib::models::investigation::InvestigationStatus;

#[test]
fn test_valid_forward_transitions() {
    use InvestigationStatus::*;

    assert!(Created.can_transition_to(&WorkspaceReady));
    assert!(WorkspaceReady.can_transition_to(&BaselineRunning));
    assert!(BaselineRunning.can_transition_to(&BaselineCaptured));
    assert!(BaselineCaptured.can_transition_to(&AwaitingBobInvestigation));
    assert!(AwaitingBobInvestigation.can_transition_to(&InvestigationImported));
    assert!(InvestigationImported.can_transition_to(&AwaitingApproval));
    assert!(AwaitingApproval.can_transition_to(&Approved));
    assert!(Approved.can_transition_to(&ImplementationInProgress));
    assert!(ImplementationInProgress.can_transition_to(&ReadyForVerification));
    assert!(ReadyForVerification.can_transition_to(&VerificationRunning));
    assert!(VerificationRunning.can_transition_to(&Verified));
    assert!(Verified.can_transition_to(&Completed));
}

#[test]
fn test_valid_failure_and_retry_transitions() {
    use InvestigationStatus::*;

    // Re-running baseline
    assert!(BaselineCaptured.can_transition_to(&BaselineRunning));
    assert!(AwaitingBobInvestigation.can_transition_to(&BaselineRunning));

    // Re-syncing artifacts
    assert!(InvestigationImported.can_transition_to(&InvestigationImported));
    assert!(AwaitingApproval.can_transition_to(&AwaitingApproval));

    // Verification failure and retry
    assert!(VerificationRunning.can_transition_to(&VerificationFailed));
    assert!(VerificationFailed.can_transition_to(&VerificationRunning));
    assert!(VerificationFailed.can_transition_to(&ImplementationInProgress));
    assert!(Approved.can_transition_to(&VerificationRunning));
    assert!(Verified.can_transition_to(&VerificationRunning));

    // Any state can transition to Failed
    assert!(Created.can_transition_to(&Failed));
    assert!(WorkspaceReady.can_transition_to(&Failed));
    assert!(BaselineRunning.can_transition_to(&Failed));
    assert!(ImplementationInProgress.can_transition_to(&Failed));
}

#[test]
fn test_invalid_transitions_rejected() {
    use InvestigationStatus::*;

    // Cannot jump from Created to Approved or Verified
    assert!(!Created.can_transition_to(&Approved));
    assert!(!Created.can_transition_to(&Verified));
    assert!(!Created.can_transition_to(&Completed));

    // Cannot jump from WorkspaceReady to Verified
    assert!(!WorkspaceReady.can_transition_to(&Verified));

    // Cannot jump from BaselineRunning to Approved
    assert!(!BaselineRunning.can_transition_to(&Approved));

    // Cannot jump from AwaitingApproval to Verified without implementation
    assert!(!AwaitingApproval.can_transition_to(&Verified));

    // Cannot restart from Completed
    assert!(!Completed.can_transition_to(&Created));
    assert!(!Completed.can_transition_to(&BaselineRunning));
}

#[test]
fn test_status_as_str() {
    use InvestigationStatus::*;

    assert_eq!(Created.as_str(), "CREATED");
    assert_eq!(WorkspaceReady.as_str(), "WORKSPACE_READY");
    assert_eq!(BaselineRunning.as_str(), "BASELINE_RUNNING");
    assert_eq!(BaselineCaptured.as_str(), "BASELINE_CAPTURED");
    assert_eq!(AwaitingBobInvestigation.as_str(), "AWAITING_BOB_INVESTIGATION");
    assert_eq!(InvestigationImported.as_str(), "INVESTIGATION_IMPORTED");
    assert_eq!(AwaitingApproval.as_str(), "AWAITING_APPROVAL");
    assert_eq!(Approved.as_str(), "APPROVED");
    assert_eq!(ImplementationInProgress.as_str(), "IMPLEMENTATION_IN_PROGRESS");
    assert_eq!(ReadyForVerification.as_str(), "READY_FOR_VERIFICATION");
    assert_eq!(VerificationRunning.as_str(), "VERIFICATION_RUNNING");
    assert_eq!(Verified.as_str(), "VERIFIED");
    assert_eq!(VerificationFailed.as_str(), "VERIFICATION_FAILED");
    assert_eq!(Completed.as_str(), "COMPLETED");
    assert_eq!(Failed.as_str(), "FAILED");
}
