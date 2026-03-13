use zcg_ab_bench::integration::{
    build_b_note_circuit, integration_metadata, verify_mock, IntegrationArm,
};
use zcg_ab_bench::shared_inputs::InputProfile;

fn main() {
    let metadata = integration_metadata(IntegrationArm::BNote);
    println!("Halo2 integration demo for B_note");
    println!("recommended_k={}", metadata.recommended_k);
    println!("rows_per_rep={}", metadata.rows_per_rep);
    println!(
        "logical_counts={{lookups:{}, mul:{}, lin:{}}}",
        metadata.logical_lookup_cells_per_rep,
        metadata.logical_mul_constraints_per_rep,
        metadata.logical_lin_constraints_per_rep
    );

    verify_mock(IntegrationArm::BNote, 1, InputProfile::Boundary)
        .expect("B_note integration demo should verify under MockProver");

    let _circuit = build_b_note_circuit(1, InputProfile::Boundary);
    println!("mock_verification=ok");
    println!("next_step=embed build_b_note_circuit(...) in your Halo2-side test harness or adapter crate");
}
