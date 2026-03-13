use crate::baseline_a::{
    BaselineASecureCircuit, ADVICE_COLS as A_ADVICE_COLS, FIXED_COLS as A_FIXED_COLS,
    INSTANCE_COLS as A_INSTANCE_COLS, LIN_CONSTRAINTS_PER_REP as A_LIN_CONSTRAINTS_PER_REP,
    LOOKUP_CELLS_PER_REP as A_LOOKUP_CELLS_PER_REP,
    MUL_CONSTRAINTS_PER_REP as A_MUL_CONSTRAINTS_PER_REP, ROWS_PER_REP as A_ROWS_PER_REP,
};
use crate::baseline_b::{
    BaselineBNoteCircuit, ADVICE_COLS as B_ADVICE_COLS, FIXED_COLS as B_FIXED_COLS,
    INSTANCE_COLS as B_INSTANCE_COLS, LIN_CONSTRAINTS_PER_REP as B_LIN_CONSTRAINTS_PER_REP,
    LOOKUP_CELLS_PER_REP as B_LOOKUP_CELLS_PER_REP,
    MUL_CONSTRAINTS_PER_REP as B_MUL_CONSTRAINTS_PER_REP, ROWS_PER_REP as B_ROWS_PER_REP,
};
use crate::shared_inputs::InputProfile;
use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::bn256::Fr;

pub type IntegrationField = Fr;

pub const A_SECURE_RECOMMENDED_K: usize = 10;
pub const B_NOTE_RECOMMENDED_K: usize = 17;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrationArm {
    ASecure,
    BNote,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegrationMetadata {
    pub arm: IntegrationArm,
    pub recommended_k: usize,
    pub rows_per_rep: usize,
    pub logical_lookup_cells_per_rep: usize,
    pub logical_mul_constraints_per_rep: usize,
    pub logical_lin_constraints_per_rep: usize,
    pub advice_cols: usize,
    pub fixed_cols: usize,
    pub instance_cols: usize,
}

pub fn integration_metadata(arm: IntegrationArm) -> IntegrationMetadata {
    match arm {
        IntegrationArm::ASecure => IntegrationMetadata {
            arm,
            recommended_k: A_SECURE_RECOMMENDED_K,
            rows_per_rep: A_ROWS_PER_REP,
            logical_lookup_cells_per_rep: A_LOOKUP_CELLS_PER_REP,
            logical_mul_constraints_per_rep: A_MUL_CONSTRAINTS_PER_REP,
            logical_lin_constraints_per_rep: A_LIN_CONSTRAINTS_PER_REP,
            advice_cols: A_ADVICE_COLS,
            fixed_cols: A_FIXED_COLS,
            instance_cols: A_INSTANCE_COLS,
        },
        IntegrationArm::BNote => IntegrationMetadata {
            arm,
            recommended_k: B_NOTE_RECOMMENDED_K,
            rows_per_rep: B_ROWS_PER_REP,
            logical_lookup_cells_per_rep: B_LOOKUP_CELLS_PER_REP,
            logical_mul_constraints_per_rep: B_MUL_CONSTRAINTS_PER_REP,
            logical_lin_constraints_per_rep: B_LIN_CONSTRAINTS_PER_REP,
            advice_cols: B_ADVICE_COLS,
            fixed_cols: B_FIXED_COLS,
            instance_cols: B_INSTANCE_COLS,
        },
    }
}

pub fn build_a_secure_circuit(
    repetitions: usize,
    input_profile: InputProfile,
) -> BaselineASecureCircuit<IntegrationField> {
    BaselineASecureCircuit::with_profile(repetitions, input_profile)
}

pub fn build_b_note_circuit(
    repetitions: usize,
    input_profile: InputProfile,
) -> BaselineBNoteCircuit<IntegrationField> {
    BaselineBNoteCircuit::with_profile(repetitions, input_profile)
}

pub fn verify_a_secure_mock(
    repetitions: usize,
    input_profile: InputProfile,
) -> Result<(), String> {
    let circuit = build_a_secure_circuit(repetitions, input_profile);
    let prover = MockProver::run(A_SECURE_RECOMMENDED_K as u32, &circuit, vec![])
        .map_err(|err| format!("MockProver construction failed: {err:?}"))?;
    prover
        .verify()
        .map_err(|failures| format!("Verification failed: {failures:?}"))
}

pub fn verify_b_note_mock(
    repetitions: usize,
    input_profile: InputProfile,
) -> Result<(), String> {
    let circuit = build_b_note_circuit(repetitions, input_profile);
    let prover = MockProver::run(B_NOTE_RECOMMENDED_K as u32, &circuit, vec![])
        .map_err(|err| format!("MockProver construction failed: {err:?}"))?;
    prover
        .verify()
        .map_err(|failures| format!("Verification failed: {failures:?}"))
}

pub fn verify_mock(
    arm: IntegrationArm,
    repetitions: usize,
    input_profile: InputProfile,
) -> Result<(), String> {
    match arm {
        IntegrationArm::ASecure => verify_a_secure_mock(repetitions, input_profile),
        IntegrationArm::BNote => verify_b_note_mock(repetitions, input_profile),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        integration_metadata, verify_mock, IntegrationArm, A_SECURE_RECOMMENDED_K,
        B_NOTE_RECOMMENDED_K,
    };
    use crate::shared_inputs::InputProfile;

    #[test]
    fn test_integration_metadata_matches_b_note_contract() {
        let meta = integration_metadata(IntegrationArm::BNote);
        assert_eq!(meta.recommended_k, B_NOTE_RECOMMENDED_K);
        assert!(meta.logical_lookup_cells_per_rep > 0);
        assert!(meta.rows_per_rep > 0);
    }

    #[test]
    fn test_integration_metadata_matches_a_secure_contract() {
        let meta = integration_metadata(IntegrationArm::ASecure);
        assert_eq!(meta.recommended_k, A_SECURE_RECOMMENDED_K);
        assert!(meta.logical_mul_constraints_per_rep > 0);
        assert!(meta.rows_per_rep > 0);
    }

    #[test]
    fn test_integration_verify_mock_b_note_boundary() {
        verify_mock(IntegrationArm::BNote, 1, InputProfile::Boundary)
            .expect("B_note boundary integration path should verify");
    }

    #[test]
    fn test_integration_verify_mock_a_secure_standard() {
        verify_mock(IntegrationArm::ASecure, 1, InputProfile::Standard)
            .expect("A_secure standard integration path should verify");
    }
}
