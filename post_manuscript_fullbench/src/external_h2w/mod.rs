use std::marker::PhantomData;

use maingate::halo2::halo2curves::ff::PrimeField;
use maingate::halo2::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error},
};
use maingate::{MainGate, MainGateConfig, MainGateInstructions, RegionCtx};

pub const ACTIVE_ROWS_PER_REP: usize = 4 + 4 + 4 + 12 + 4; // inv/EF/EE/hor/car
pub const LOOKUP_CELLS_PER_REP: usize = 0;
pub const MUL_CONSTRAINTS_PER_REP: usize = ACTIVE_ROWS_PER_REP * (31 + 31 + 31 + 66);
pub const LIN_CONSTRAINTS_PER_REP: usize = ACTIVE_ROWS_PER_REP * 4; // x/y/z/q recomposition
pub const ADVICE_COLS: usize = 5;
pub const FIXED_COLS: usize = 9;
pub const INSTANCE_COLS: usize = 1;

#[derive(Clone, Debug)]
pub struct ExternalH2WConfig {
    pub main_gate_config: MainGateConfig,
}

#[derive(Clone, Debug)]
pub struct ExternalH2W66BitCircuit<F: PrimeField> {
    pub repetitions: usize,
    marker: PhantomData<F>,
}

impl<F: PrimeField + From<u64>> ExternalH2W66BitCircuit<F> {
    pub fn new(repetitions: usize) -> Self {
        assert!(
            repetitions > 0,
            "repetitions must be positive for external_h2w baseline"
        );
        Self {
            repetitions,
            marker: PhantomData,
        }
    }

}

impl<F: PrimeField + From<u64>> Circuit<F> for ExternalH2W66BitCircuit<F> {
    type Config = ExternalH2WConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            repetitions: self.repetitions,
            marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let main_gate_config = MainGate::<F>::configure(meta);
        ExternalH2WConfig { main_gate_config }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let main_gate = MainGate::<F>::new(config.main_gate_config);
        layouter.assign_region(
            || "external halo2wrong 66-bit decomposition region",
            |region| {
                let mut ctx = RegionCtx::new(region, 0);

                for rep in 0..self.repetitions {
                    // Row-family-equivalent decomposition workload:
                    // per active row: x(31), y(31), z(31), q(66)
                    for row in 0..ACTIVE_ROWS_PER_REP {
                        let seed = rep as u64 * 10_000 + row as u64 * 97 + 12_345;
                        let x_u64 = seed & ((1u64 << 31) - 1);
                        let y_u64 = (seed.wrapping_mul(3) + 7) & ((1u64 << 31) - 1);
                        let z_u64 = (seed.wrapping_mul(5) + 11) & ((1u64 << 31) - 1);
                        // q target is 66-bit class; witness uses u64 so top two bits stay zero.
                        let q_u64 = (seed.wrapping_mul(13) + 17) & ((1u64 << 62) - 1);

                        let x = main_gate.assign_value(&mut ctx, Value::known(F::from(x_u64)))?;
                        let y = main_gate.assign_value(&mut ctx, Value::known(F::from(y_u64)))?;
                        let z = main_gate.assign_value(&mut ctx, Value::known(F::from(z_u64)))?;
                        let q = main_gate.assign_value(&mut ctx, Value::known(F::from(q_u64)))?;

                        let _ = main_gate.to_bits(&mut ctx, &x, 31)?;
                        let _ = main_gate.to_bits(&mut ctx, &y, 31)?;
                        let _ = main_gate.to_bits(&mut ctx, &z, 31)?;
                        let _ = main_gate.to_bits(&mut ctx, &q, 66)?;
                    }
                }

                Ok(())
            },
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ExternalH2W66BitCircuit;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr;

    #[test]
    fn test_external_h2w_66bit_valid() {
        let circuit = ExternalH2W66BitCircuit::<Fr>::new(1);
        let prover = MockProver::run(13, &circuit, vec![vec![]]).expect("mock prover should build");
        prover.assert_satisfied();
    }
}
