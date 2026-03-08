use std::marker::PhantomData;

use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{Layouter, Region, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector};
use halo2_proofs::poly::Rotation;

pub const FAMILY_INV_ROWS: usize = 4;
pub const FAMILY_EF_ROWS: usize = 4;
pub const FAMILY_EE_ROWS: usize = 4;
pub const FAMILY_HOR_ROWS: usize = 12;
pub const FAMILY_CAR_ROWS: usize = 4;

pub const ACTIVE_ROWS_PER_REP: usize =
    FAMILY_INV_ROWS + FAMILY_EF_ROWS + FAMILY_EE_ROWS + FAMILY_HOR_ROWS + FAMILY_CAR_ROWS;
pub const INACTIVE_ROWS_PER_REP: usize = 1;
pub const ROWS_PER_REP: usize = ACTIVE_ROWS_PER_REP + INACTIVE_ROWS_PER_REP;

pub const X_BITS: usize = 31;
pub const Y_BITS: usize = 31;
pub const Z_BITS: usize = 31;
pub const Q_BITS: usize = 66;
pub const TOTAL_DECOMP_BITS: usize = X_BITS + Y_BITS + Z_BITS + Q_BITS;

pub const LOOKUP_CELLS_PER_REP: usize = 0;
pub const MUL_CONSTRAINTS_PER_REP: usize =
    (ACTIVE_ROWS_PER_REP * TOTAL_DECOMP_BITS) + (ACTIVE_ROWS_PER_REP * 3) + (FAMILY_INV_ROWS + FAMILY_EF_ROWS + FAMILY_EE_ROWS + FAMILY_HOR_ROWS); // 4560
pub const LIN_CONSTRAINTS_PER_REP: usize = (ACTIVE_ROWS_PER_REP * 4) // x/y/z/q recomposition
    + 1 // q31 high-bit zeroing
    + FAMILY_CAR_ROWS // car relation
    + ACTIVE_ROWS_PER_REP // digest binding
    + 1; // inactive-row zero-extension

pub const ADVICE_COLS: usize = 8 + TOTAL_DECOMP_BITS; // x,y,z,q,nu_x,nu_y,nu_z,digest + bits
pub const FIXED_COLS: usize = 6; // 5 family selectors + inactive selector
pub const INSTANCE_COLS: usize = 0;

const P_U32: u32 = 2_147_483_647;
const DIGEST_CONST_U64: u64 = 0xC0DE;
const EE_HOR_COEFF_U64: u64 = 34_359_738_336; // 2^35

#[derive(Clone, Debug)]
pub struct BaselineASecureConfig {
    pub sel_inv: Selector,
    pub sel_ef: Selector,
    pub sel_ee: Selector,
    pub sel_hor: Selector,
    pub sel_car: Selector,
    pub sel_inactive: Selector,

    pub x: Column<Advice>,
    pub y: Column<Advice>,
    pub z: Column<Advice>,
    pub q: Column<Advice>,
    pub nu_x: Column<Advice>,
    pub nu_y: Column<Advice>,
    pub nu_z: Column<Advice>,
    pub digest: Column<Advice>,

    pub x_bits: [Column<Advice>; X_BITS],
    pub y_bits: [Column<Advice>; Y_BITS],
    pub z_bits: [Column<Advice>; Z_BITS],
    pub q_bits: [Column<Advice>; Q_BITS],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ASecureFault {
    PInvWitnessToyAttack,
    OmittedQuotientWiring,
    ClassMapMismatch31_66,
    DigestMismatch,
    InactiveRowZeroExtensionViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Inv,
    Ef,
    Ee,
    Hor,
    Car,
}

#[derive(Clone, Debug)]
pub struct BaselineASecureCircuit<F: Field> {
    pub repetitions: usize,
    pub fault: Option<ASecureFault>,
    marker: PhantomData<F>,
}

impl<F: Field + From<u64>> BaselineASecureCircuit<F> {
    pub fn new(repetitions: usize) -> Self {
        Self::with_fault(repetitions, None)
    }

    pub fn with_fault(repetitions: usize, fault: Option<ASecureFault>) -> Self {
        assert!(
            repetitions > 0,
            "repetitions must be positive for A_secure baseline"
        );
        Self {
            repetitions,
            fault,
            marker: PhantomData,
        }
    }

    fn small_const(v: u64) -> F {
        F::from(v)
    }

    fn fr_pow2(exp: usize) -> F {
        let mut out = F::ONE;
        for _ in 0..exp {
            out += out;
        }
        out
    }

    fn families() -> [(Family, usize); 5] {
        [
            (Family::Inv, FAMILY_INV_ROWS),
            (Family::Ef, FAMILY_EF_ROWS),
            (Family::Ee, FAMILY_EE_ROWS),
            (Family::Hor, FAMILY_HOR_ROWS),
            (Family::Car, FAMILY_CAR_ROWS),
        ]
    }

    fn base_row_values(family: Family, local_idx: usize) -> (u32, u32, u32, u64, bool) {
        match family {
            Family::Inv => (1, 0, 0, 1, false),
            Family::Ef => (1, 1, 1, (1u64 << 30) + 1 + local_idx as u64, true),
            Family::Ee => (0, 1, 0, (1u64 << 62) + 11 + local_idx as u64, false),
            Family::Hor => (0, 1, 0, (1u64 << 61) + 101 + local_idx as u64, false),
            Family::Car => (1, 1, 7, (1u64 << 30) + 1000 + local_idx as u64, true),
        }
    }

    fn assign_bits<const N: usize>(
        region: &mut Region<'_, F>,
        cols: &[Column<Advice>; N],
        offset: usize,
        value: u128,
        used_bits: usize,
        label: &str,
    ) -> Result<(), Error> {
        for i in 0..N {
            let bit = if i < used_bits && ((value >> i) & 1) == 1 {
                F::ONE
            } else {
                F::ZERO
            };
            region.assign_advice(
                || format!("{label}_bit_{i}_row_{offset}"),
                cols[i],
                offset,
                || Value::known(bit),
            )?;
        }
        Ok(())
    }
}

impl<F: Field + From<u64>> Circuit<F> for BaselineASecureCircuit<F> {
    type Config = BaselineASecureConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            repetitions: self.repetitions,
            fault: self.fault,
            marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let sel_inv = meta.complex_selector();
        let sel_ef = meta.complex_selector();
        let sel_ee = meta.complex_selector();
        let sel_hor = meta.complex_selector();
        let sel_car = meta.complex_selector();
        let sel_inactive = meta.selector();

        let x = meta.advice_column();
        let y = meta.advice_column();
        let z = meta.advice_column();
        let q = meta.advice_column();
        let nu_x = meta.advice_column();
        let nu_y = meta.advice_column();
        let nu_z = meta.advice_column();
        let digest = meta.advice_column();

        let x_bits = std::array::from_fn(|_| meta.advice_column());
        let y_bits = std::array::from_fn(|_| meta.advice_column());
        let z_bits = std::array::from_fn(|_| meta.advice_column());
        let q_bits = std::array::from_fn(|_| meta.advice_column());

        let q_active = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, F>| {
            meta.query_selector(sel_inv)
                + meta.query_selector(sel_ef)
                + meta.query_selector(sel_ee)
                + meta.query_selector(sel_hor)
                + meta.query_selector(sel_car)
        };
        let q31 = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, F>| {
            meta.query_selector(sel_ef) + meta.query_selector(sel_car)
        };

        meta.create_gate("A_secure explicit bit decomposition + no-equality", |meta| {
            let qa = q_active(meta);
            let one = Expression::Constant(F::ONE);
            let p = Expression::Constant(Self::small_const(P_U32 as u64));

            let x_e = meta.query_advice(x, Rotation::cur());
            let y_e = meta.query_advice(y, Rotation::cur());
            let z_e = meta.query_advice(z, Rotation::cur());
            let q_e = meta.query_advice(q, Rotation::cur());
            let nu_x_e = meta.query_advice(nu_x, Rotation::cur());
            let nu_y_e = meta.query_advice(nu_y, Rotation::cur());
            let nu_z_e = meta.query_advice(nu_z, Rotation::cur());

            let x_bits_e: [Expression<F>; X_BITS] =
                std::array::from_fn(|i| meta.query_advice(x_bits[i], Rotation::cur()));
            let y_bits_e: [Expression<F>; Y_BITS] =
                std::array::from_fn(|i| meta.query_advice(y_bits[i], Rotation::cur()));
            let z_bits_e: [Expression<F>; Z_BITS] =
                std::array::from_fn(|i| meta.query_advice(z_bits[i], Rotation::cur()));
            let q_bits_e: [Expression<F>; Q_BITS] =
                std::array::from_fn(|i| meta.query_advice(q_bits[i], Rotation::cur()));

            let mut out = Vec::with_capacity(TOTAL_DECOMP_BITS + 7);

            for b in x_bits_e.iter() {
                out.push(qa.clone() * b.clone() * (one.clone() - b.clone()));
            }
            for b in y_bits_e.iter() {
                out.push(qa.clone() * b.clone() * (one.clone() - b.clone()));
            }
            for b in z_bits_e.iter() {
                out.push(qa.clone() * b.clone() * (one.clone() - b.clone()));
            }
            for b in q_bits_e.iter() {
                out.push(qa.clone() * b.clone() * (one.clone() - b.clone()));
            }

            let mut coeff = F::ONE;
            let mut x_recomposed = Expression::Constant(F::ZERO);
            for b in x_bits_e.iter() {
                x_recomposed = x_recomposed + b.clone() * Expression::Constant(coeff);
                coeff += coeff;
            }
            coeff = F::ONE;
            let mut y_recomposed = Expression::Constant(F::ZERO);
            for b in y_bits_e.iter() {
                y_recomposed = y_recomposed + b.clone() * Expression::Constant(coeff);
                coeff += coeff;
            }
            coeff = F::ONE;
            let mut z_recomposed = Expression::Constant(F::ZERO);
            for b in z_bits_e.iter() {
                z_recomposed = z_recomposed + b.clone() * Expression::Constant(coeff);
                coeff += coeff;
            }
            coeff = F::ONE;
            let mut q_recomposed = Expression::Constant(F::ZERO);
            for b in q_bits_e.iter() {
                q_recomposed = q_recomposed + b.clone() * Expression::Constant(coeff);
                coeff += coeff;
            }

            out.push(qa.clone() * (x_e.clone() - x_recomposed));
            out.push(qa.clone() * (y_e.clone() - y_recomposed));
            out.push(qa.clone() * (z_e.clone() - z_recomposed));
            out.push(qa.clone() * (q_e.clone() - q_recomposed));
            out.push(qa.clone() * ((x_e - p.clone()) * nu_x_e - one.clone()));
            out.push(qa.clone() * ((y_e - p.clone()) * nu_y_e - one.clone()));
            out.push(qa * ((z_e - p) * nu_z_e - one));
            out
        });

        meta.create_gate("A_secure q31 class upper bits are zero", |meta| {
            let q31_e = q31(meta);
            let q_bits_e: [Expression<F>; Q_BITS] =
                std::array::from_fn(|i| meta.query_advice(q_bits[i], Rotation::cur()));
            let mut high_sum = Expression::Constant(F::ZERO);
            for b in q_bits_e.iter().skip(31) {
                high_sum = high_sum + b.clone();
            }
            vec![q31_e * high_sum]
        });

        meta.create_gate("A_secure family equations", |meta| {
            let inv = meta.query_selector(sel_inv);
            let ef = meta.query_selector(sel_ef);
            let ee = meta.query_selector(sel_ee);
            let hor = meta.query_selector(sel_hor);
            let car = meta.query_selector(sel_car);

            let x_e = meta.query_advice(x, Rotation::cur());
            let y_e = meta.query_advice(y, Rotation::cur());
            let z_e = meta.query_advice(z, Rotation::cur());
            let q_e = meta.query_advice(q, Rotation::cur());

            let coeff = Expression::Constant(Self::small_const(EE_HOR_COEFF_U64));
            let five = Expression::Constant(Self::small_const(5));

            vec![
                inv * (q_e * x_e.clone() - x_e.clone()),
                ef * (x_e.clone() * y_e.clone() - z_e.clone()),
                ee * (coeff.clone() * x_e.clone() * y_e.clone() - z_e.clone()),
                hor * (coeff * x_e.clone() * y_e - z_e.clone()),
                car * (x_e + meta.query_advice(y, Rotation::cur()) + five - z_e),
            ]
        });

        meta.create_gate("A_secure digest binding", |meta| {
            let qa = q_active(meta);
            let d = meta.query_advice(digest, Rotation::cur());
            let expected = Expression::Constant(Self::small_const(DIGEST_CONST_U64));
            vec![qa * (d - expected)]
        });

        meta.create_gate("A_secure inactive-row zero extension", |meta| {
            let qz = meta.query_selector(sel_inactive);
            let mut sum = meta.query_advice(x, Rotation::cur())
                + meta.query_advice(y, Rotation::cur())
                + meta.query_advice(z, Rotation::cur())
                + meta.query_advice(q, Rotation::cur())
                + meta.query_advice(nu_x, Rotation::cur())
                + meta.query_advice(nu_y, Rotation::cur())
                + meta.query_advice(nu_z, Rotation::cur())
                + meta.query_advice(digest, Rotation::cur());

            for col in x_bits {
                sum = sum + meta.query_advice(col, Rotation::cur());
            }
            for col in y_bits {
                sum = sum + meta.query_advice(col, Rotation::cur());
            }
            for col in z_bits {
                sum = sum + meta.query_advice(col, Rotation::cur());
            }
            for col in q_bits {
                sum = sum + meta.query_advice(col, Rotation::cur());
            }

            vec![qz * sum]
        });

        BaselineASecureConfig {
            sel_inv,
            sel_ef,
            sel_ee,
            sel_hor,
            sel_car,
            sel_inactive,
            x,
            y,
            z,
            q,
            nu_x,
            nu_y,
            nu_z,
            digest,
            x_bits,
            y_bits,
            z_bits,
            q_bits,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let p = Self::small_const(P_U32 as u64);
        for rep in 0..self.repetitions {
            layouter.assign_region(
                || format!("A_secure row-family region rep={rep}"),
                |mut region| {
                    let mut offset = 0usize;

                    for (family, count) in Self::families() {
                        for local in 0..count {
                            match family {
                                Family::Inv => config.sel_inv.enable(&mut region, offset)?,
                                Family::Ef => config.sel_ef.enable(&mut region, offset)?,
                                Family::Ee => config.sel_ee.enable(&mut region, offset)?,
                                Family::Hor => config.sel_hor.enable(&mut region, offset)?,
                                Family::Car => config.sel_car.enable(&mut region, offset)?,
                            }

                            let (x_u, y_u, z_u, mut q_u, is_q31_row) =
                                Self::base_row_values(family, local);

                            let x_f = Self::small_const(x_u as u64);
                            let y_f = Self::small_const(y_u as u64);
                            let z_f = Self::small_const(z_u as u64);
                            let mut q_f = Self::small_const(q_u);
                            let mut digest_f = Self::small_const(DIGEST_CONST_U64);

                            if rep == 0 {
                                match self.fault {
                                    Some(ASecureFault::PInvWitnessToyAttack)
                                        if family == Family::Inv && local == 0 =>
                                    {
                                        q_f = F::ZERO - F::ONE;
                                    }
                                    Some(ASecureFault::OmittedQuotientWiring)
                                        if family == Family::Inv && local == 0 =>
                                    {
                                        q_f -= Self::fr_pow2(17);
                                    }
                                    Some(ASecureFault::ClassMapMismatch31_66)
                                        if family == Family::Ef && local == 0 =>
                                    {
                                        q_u = (1u64 << 40) + 7;
                                        q_f = Self::small_const(q_u);
                                    }
                                    Some(ASecureFault::DigestMismatch)
                                        if family == Family::Inv && local == 0 =>
                                    {
                                        digest_f = Self::small_const(DIGEST_CONST_U64 + 1);
                                    }
                                    _ => {}
                                }
                            }

                            let nu_x_f =
                                Option::<F>::from((x_f - p).invert()).expect("x != p by construction");
                            let nu_y_f =
                                Option::<F>::from((y_f - p).invert()).expect("y != p by construction");
                            let nu_z_f =
                                Option::<F>::from((z_f - p).invert()).expect("z != p by construction");

                            region.assign_advice(
                                || format!("x_rep_{rep}_row_{offset}"),
                                config.x,
                                offset,
                                || Value::known(x_f),
                            )?;
                            region.assign_advice(
                                || format!("y_rep_{rep}_row_{offset}"),
                                config.y,
                                offset,
                                || Value::known(y_f),
                            )?;
                            region.assign_advice(
                                || format!("z_rep_{rep}_row_{offset}"),
                                config.z,
                                offset,
                                || Value::known(z_f),
                            )?;
                            region.assign_advice(
                                || format!("q_rep_{rep}_row_{offset}"),
                                config.q,
                                offset,
                                || Value::known(q_f),
                            )?;
                            region.assign_advice(
                                || format!("nu_x_rep_{rep}_row_{offset}"),
                                config.nu_x,
                                offset,
                                || Value::known(nu_x_f),
                            )?;
                            region.assign_advice(
                                || format!("nu_y_rep_{rep}_row_{offset}"),
                                config.nu_y,
                                offset,
                                || Value::known(nu_y_f),
                            )?;
                            region.assign_advice(
                                || format!("nu_z_rep_{rep}_row_{offset}"),
                                config.nu_z,
                                offset,
                                || Value::known(nu_z_f),
                            )?;
                            region.assign_advice(
                                || format!("digest_rep_{rep}_row_{offset}"),
                                config.digest,
                                offset,
                                || Value::known(digest_f),
                            )?;

                            Self::assign_bits(
                                &mut region,
                                &config.x_bits,
                                offset,
                                x_u as u128,
                                X_BITS,
                                "x",
                            )?;
                            Self::assign_bits(
                                &mut region,
                                &config.y_bits,
                                offset,
                                y_u as u128,
                                Y_BITS,
                                "y",
                            )?;
                            Self::assign_bits(
                                &mut region,
                                &config.z_bits,
                                offset,
                                z_u as u128,
                                Z_BITS,
                                "z",
                            )?;
                            Self::assign_bits(
                                &mut region,
                                &config.q_bits,
                                offset,
                                q_u as u128,
                                Q_BITS,
                                "q",
                            )?;

                            if is_q31_row {
                                debug_assert!(
                                    q_u < (1u64 << 31)
                                        || self.fault == Some(ASecureFault::ClassMapMismatch31_66),
                                    "q31 row must fit 31 bits unless fault is injected"
                                );
                            }

                            offset += 1;
                        }
                    }

                    config.sel_inactive.enable(&mut region, offset)?;
                    let mut inactive_x_f = F::ZERO;
                    let inactive_y_f = F::ZERO;
                    let inactive_z_f = F::ZERO;
                    let inactive_q_f = F::ZERO;
                    let inactive_nu_x_f = F::ZERO;
                    let inactive_nu_y_f = F::ZERO;
                    let inactive_nu_z_f = F::ZERO;
                    let inactive_digest_f = F::ZERO;
                    let mut inactive_x_bits = 0u128;

                    if rep == 0
                        && self.fault == Some(ASecureFault::InactiveRowZeroExtensionViolation)
                    {
                        inactive_x_f = Self::small_const(5);
                        inactive_x_bits = 1u128 << 3;
                    }

                    region.assign_advice(
                        || format!("inactive_x_rep_{rep}"),
                        config.x,
                        offset,
                        || Value::known(inactive_x_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_y_rep_{rep}"),
                        config.y,
                        offset,
                        || Value::known(inactive_y_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_z_rep_{rep}"),
                        config.z,
                        offset,
                        || Value::known(inactive_z_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_q_rep_{rep}"),
                        config.q,
                        offset,
                        || Value::known(inactive_q_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_nu_x_rep_{rep}"),
                        config.nu_x,
                        offset,
                        || Value::known(inactive_nu_x_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_nu_y_rep_{rep}"),
                        config.nu_y,
                        offset,
                        || Value::known(inactive_nu_y_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_nu_z_rep_{rep}"),
                        config.nu_z,
                        offset,
                        || Value::known(inactive_nu_z_f),
                    )?;
                    region.assign_advice(
                        || format!("inactive_digest_rep_{rep}"),
                        config.digest,
                        offset,
                        || Value::known(inactive_digest_f),
                    )?;

                    Self::assign_bits(
                        &mut region,
                        &config.x_bits,
                        offset,
                        inactive_x_bits,
                        X_BITS,
                        "inactive_x",
                    )?;
                    Self::assign_bits(
                        &mut region,
                        &config.y_bits,
                        offset,
                        0,
                        Y_BITS,
                        "inactive_y",
                    )?;
                    Self::assign_bits(
                        &mut region,
                        &config.z_bits,
                        offset,
                        0,
                        Z_BITS,
                        "inactive_z",
                    )?;
                    Self::assign_bits(
                        &mut region,
                        &config.q_bits,
                        offset,
                        0,
                        Q_BITS,
                        "inactive_q",
                    )?;

                    offset += 1;
                    debug_assert_eq!(offset, ROWS_PER_REP);

                    Ok(())
                },
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ASecureFault, BaselineASecureCircuit};
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr;

    fn assert_rejected(circuit: BaselineASecureCircuit<Fr>) {
        let k = 10;
        let prover = MockProver::run(k, &circuit, vec![]).expect("mock prover should build");
        assert!(
            prover.verify().is_err(),
            "expected constraints to reject invalid witness"
        );
    }

    #[test]
    fn test_a_secure_equivalent_valid() {
        let circuit = BaselineASecureCircuit::<Fr>::new(1);
        let k = 10;
        let prover = MockProver::run(k, &circuit, vec![]).expect("mock prover should build");
        prover.assert_satisfied();
    }

    #[test]
    fn test_a_secure_rejects_p_inv_witness_toy_attack() {
        let circuit = BaselineASecureCircuit::<Fr>::with_fault(1, Some(ASecureFault::PInvWitnessToyAttack));
        assert_rejected(circuit);
    }

    #[test]
    fn test_a_secure_rejects_omitted_quotient_wiring() {
        let circuit =
            BaselineASecureCircuit::<Fr>::with_fault(1, Some(ASecureFault::OmittedQuotientWiring));
        assert_rejected(circuit);
    }

    #[test]
    fn test_a_secure_rejects_class_map_mismatch_31_66() {
        let circuit =
            BaselineASecureCircuit::<Fr>::with_fault(1, Some(ASecureFault::ClassMapMismatch31_66));
        assert_rejected(circuit);
    }

    #[test]
    fn test_a_secure_rejects_digest_mismatch() {
        let circuit = BaselineASecureCircuit::<Fr>::with_fault(1, Some(ASecureFault::DigestMismatch));
        assert_rejected(circuit);
    }

    #[test]
    fn test_a_secure_rejects_inactive_row_zero_extension_violation() {
        let circuit = BaselineASecureCircuit::<Fr>::with_fault(
            1,
            Some(ASecureFault::InactiveRowZeroExtensionViolation),
        );
        assert_rejected(circuit);
    }
}
