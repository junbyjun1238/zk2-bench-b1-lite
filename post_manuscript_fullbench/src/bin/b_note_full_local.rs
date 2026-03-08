use std::time::Instant;

use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof};
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use halo2_proofs::poly::kzg::multiopen::{ProverSHPLONK, VerifierSHPLONK};
use halo2_proofs::poly::kzg::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use rand_core::OsRng;
use zcg_ab_bench::shared_inputs::InputProfile;
use zcg_ab_bench::baseline_b::{
    BaselineBNoteCircuit, ADVICE_COLS, FIXED_COLS, INSTANCE_COLS, LIN_CONSTRAINTS_PER_REP,
    LOOKUP_CELLS_PER_REP, MUL_CONSTRAINTS_PER_REP, ROWS_PER_REP,
};

const MIN_K: u32 = 17;
const MAX_K: u32 = 21;

#[derive(Debug)]
struct Args {
    scale: usize,
    k_run_override: Option<u32>,
    known_k_min: Option<u32>,
    probe_k_min_only: bool,
    input_profile: InputProfile,
}

fn parse_input_profile(value: &str) -> Result<InputProfile, String> {
    match value {
        "standard" => Ok(InputProfile::Standard),
        "boundary" => Ok(InputProfile::Boundary),
        "adversarial" => Ok(InputProfile::Adversarial),
        _ => Err(format!("invalid --input-profile value: {value}")),
    }
}

fn catch_unwind_silent<F, R>(f: F) -> Result<R, Box<dyn std::any::Any + Send>>
where
    F: FnOnce() -> R,
{
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(prev_hook);
    result
}

fn find_k_min(circuit: &BaselineBNoteCircuit<Fr>) -> Result<u32, String> {
    for k in MIN_K..=MAX_K {
        let maybe_prover = catch_unwind_silent(|| MockProver::run(k, circuit, vec![]));
        if let Ok(Ok(_)) = maybe_prover {
            return Ok(k);
        }
    }
    Err(format!(
        "failed to find feasible k in range [{MIN_K}, {MAX_K}] for B_note"
    ))
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let mut scale: usize = 1;
    let mut k_run_override: Option<u32> = None;
    let mut known_k_min: Option<u32> = None;
    let mut probe_k_min_only = false;
    let mut input_profile = InputProfile::Standard;
    while let Some(arg) = args.next() {
        if arg == "--scale" {
            let value = args
                .next()
                .ok_or_else(|| "missing value for --scale".to_string())?;
            scale = value
                .parse::<usize>()
                .map_err(|_| format!("invalid --scale value: {value}"))?;
        } else if arg == "--k-run" {
            let value = args
                .next()
                .ok_or_else(|| "missing value for --k-run".to_string())?;
            let parsed = value
                .parse::<u32>()
                .map_err(|_| format!("invalid --k-run value: {value}"))?;
            k_run_override = Some(parsed);
        } else if arg == "--known-k-min" {
            let value = args
                .next()
                .ok_or_else(|| "missing value for --known-k-min".to_string())?;
            let parsed = value
                .parse::<u32>()
                .map_err(|_| format!("invalid --known-k-min value: {value}"))?;
            known_k_min = Some(parsed);
        } else if arg == "--probe-k-min" {
            probe_k_min_only = true;
        } else if arg == "--input-profile" {
            let value = args
                .next()
                .ok_or_else(|| "missing value for --input-profile".to_string())?;
            input_profile = parse_input_profile(&value)?;
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }
    if scale == 0 {
        return Err("--scale must be positive".to_string());
    }
    Ok(Args {
        scale,
        k_run_override,
        known_k_min,
        probe_k_min_only,
        input_profile,
    })
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let scale = args.scale;
    let circuit = BaselineBNoteCircuit::<Fr>::with_profile(scale, args.input_profile);

    if args.probe_k_min_only {
        let k_min = find_k_min(&circuit)?;
        println!("{{\"k_min\":{k_min}}}");
        return Ok(());
    }

    let synth_start = Instant::now();
    let synth_ms = synth_start.elapsed().as_secs_f64() * 1_000.0;

    let k_min = if let Some(v) = args.known_k_min {
        if !(MIN_K..=MAX_K).contains(&v) {
            return Err(format!(
                "provided --known-k-min={v} is outside [{MIN_K}, {MAX_K}] for B_note"
            ));
        }
        v
    } else {
        find_k_min(&circuit)?
    };
    let k_run = if let Some(v) = args.k_run_override {
        if v < k_min {
            return Err(format!(
                "requested --k-run={v} is below k_min={k_min} for B_note at scale={scale}"
            ));
        }
        if v > MAX_K {
            return Err(format!(
                "requested --k-run={v} exceeds MAX_K={MAX_K} for B_note binary"
            ));
        }
        v
    } else {
        k_min
    };

    let params = ParamsKZG::<Bn256>::new(k_run);

    let keygen_vk_start = Instant::now();
    let vk = keygen_vk(&params, &circuit).map_err(|e| format!("keygen_vk failed: {e:?}"))?;
    let keygen_vk_ms = keygen_vk_start.elapsed().as_secs_f64() * 1_000.0;

    let keygen_pk_start = Instant::now();
    let pk = keygen_pk(&params, vk, &circuit).map_err(|e| format!("keygen_pk failed: {e:?}"))?;
    let keygen_pk_ms = keygen_pk_start.elapsed().as_secs_f64() * 1_000.0;

    let prove_start = Instant::now();
    let mut transcript = Blake2bWrite::<Vec<u8>, _, Challenge255<_>>::init(vec![]);
    create_proof::<KZGCommitmentScheme<Bn256>, ProverSHPLONK<Bn256>, _, _, _, _>(
        &params,
        &pk,
        &[circuit.clone()],
        &[&[]],
        OsRng,
        &mut transcript,
    )
    .map_err(|e| format!("create_proof failed: {e:?}"))?;
    let proof = transcript.finalize();
    let prove_ms = prove_start.elapsed().as_secs_f64() * 1_000.0;

    let verify_start = Instant::now();
    let strategy = SingleStrategy::new(&params);
    let mut verify_transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    let verify_result = verify_proof::<KZGCommitmentScheme<Bn256>, VerifierSHPLONK<Bn256>, _, _, _>(
        &params,
        pk.get_vk(),
        strategy,
        &[&[]],
        &mut verify_transcript,
    );
    let verify_ms = verify_start.elapsed().as_secs_f64() * 1_000.0;
    if let Err(e) = verify_result {
        return Err(format!("verify_proof failed: {e:?}"));
    }

    let logical_lookup_cells = (LOOKUP_CELLS_PER_REP * scale) as u64;
    let logical_mul_constraints = (MUL_CONSTRAINTS_PER_REP * scale) as u64;
    let logical_lin_constraints = (LIN_CONSTRAINTS_PER_REP * scale) as u64;
    let physical_rows = (ROWS_PER_REP * scale) as u64;
    let advice_cols = ADVICE_COLS as u64;
    let fixed_cols = FIXED_COLS as u64;
    let instance_cols = INSTANCE_COLS as u64;
    let proof_bytes: u64 = proof.len() as u64;
    let input_profile = match args.input_profile {
        InputProfile::Standard => "standard",
        InputProfile::Boundary => "boundary",
        InputProfile::Adversarial => "adversarial",
    };

    let payload = format!(
        "{{\"n\":{scale},\"R_hor\":{scale},\"R_car\":{scale},\"k_min\":{k_min},\"k_run\":{k_run},\
\"logical_lookup_cells\":{logical_lookup_cells},\"logical_mul_constraints\":{logical_mul_constraints},\
\"logical_lin_constraints\":{logical_lin_constraints},\"physical_rows\":{physical_rows},\
\"advice_cols\":{advice_cols},\"fixed_cols\":{fixed_cols},\"instance_cols\":{instance_cols},\
\"synth_ms\":{synth_ms},\"keygen_vk_ms\":{keygen_vk_ms},\"keygen_pk_ms\":{keygen_pk_ms},\
\"prove_ms\":{prove_ms},\"verify_ms\":{verify_ms},\"proof_bytes\":{proof_bytes},\
\"status\":\"full-local-ok\",\"notes\":\"B_note full-local measured with real keygen/prove/verify (Blake2b transcript); input_profile={input_profile}\"}}"
    );
    println!("{payload}");
    Ok(())
}
