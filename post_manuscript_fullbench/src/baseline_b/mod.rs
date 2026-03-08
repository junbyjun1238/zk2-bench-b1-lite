use std::marker::PhantomData;

use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector, TableColumn};
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

// Per active row:
// - canonical lookups for x/y/z: 6
// - class31 q lookups: 2 (EF/CAR) or class66 q lookups: 5 (INV/EE/HOR)
// Per repetition:
// - 20 class66 rows, 8 class31 rows
pub const LOOKUP_CELLS_PER_REP: usize = (ACTIVE_ROWS_PER_REP * 6) + (20 * 5) + (8 * 2); // 284
pub const MUL_CONSTRAINTS_PER_REP: usize = (ACTIVE_ROWS_PER_REP * 3) + (FAMILY_INV_ROWS + FAMILY_EF_ROWS + FAMILY_EE_ROWS + FAMILY_HOR_ROWS); // 108
pub const LIN_CONSTRAINTS_PER_REP: usize = (ACTIVE_ROWS_PER_REP * 3)
    + ACTIVE_ROWS_PER_REP // q recomposition
    + FAMILY_CAR_ROWS // car relation
    + ACTIVE_ROWS_PER_REP // digest binding
    + 1 // q31 high-limb zeroing
    + 1; // inactive-row zero-extension gate

pub const ADVICE_COLS: usize = 19;
pub const FIXED_COLS: usize = 9; // 6 selectors + 3 lookup tables
pub const INSTANCE_COLS: usize = 0;

const P_U32: u32 = 2_147_483_647;
const DIGEST_CONST_U64: u64 = 0xC0DE;
const EE_HOR_COEFF_U64: u64 = 34_359_738_336; // 2^35
const T16_TABLE_SIZE: u32 = 4096;
const T15_TABLE_SIZE: u32 = 1024;

#[derive(Clone, Debug)]
pub struct BaselineBNoteConfig {
    pub sel_inv: Selector,
    pub sel_ef: Selector,
    pub sel_ee: Selector,
    pub sel_hor: Selector,
    pub sel_car: Selector,
    pub sel_inactive: Selector,

    pub x0: Column<Advice>,
    pub x1: Column<Advice>,
    pub x: Column<Advice>,
    pub nu_x: Column<Advice>,

    pub y0: Column<Advice>,
    pub y1: Column<Advice>,
    pub y: Column<Advice>,
    pub nu_y: Column<Advice>,

    pub z0: Column<Advice>,
    pub z1: Column<Advice>,
    pub z: Column<Advice>,
    pub nu_z: Column<Advice>,

    pub q0: Column<Advice>,
    pub q1: Column<Advice>,
    pub q2: Column<Advice>,
    pub q3: Column<Advice>,
    pub q4: Column<Advice>,
    pub q: Column<Advice>,

    pub digest: Column<Advice>,

    pub t16: TableColumn,
    pub t15: TableColumn,
    pub t2: TableColumn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BNoteFault {
    PInvWitnessToyAttack,
    OmittedQuotientWiring,
    ClassMapMismatch31_66,
    DigestMismatch,
    InactiveRowZeroExtensionViolation,
}

#[derive(Clone, Debug)]
pub struct BaselineBNoteCircuit<F: Field> {
    pub repetitions: usize,
    pub fault: Option<BNoteFault>,
    marker: PhantomData<F>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Inv,
    Ef,
    Ee,
    Hor,
    Car,
}

impl<F: Field + From<u64>> BaselineBNoteCircuit<F> {
    pub fn new(repetitions: usize) -> Self {
        Self::with_fault(repetitions, None)
    }

    pub fn with_fault(repetitions: usize, fault: Option<BNoteFault>) -> Self {
        assert!(
            repetitions > 0,
            "repetitions must be positive for B_note baseline circuit"
        );
        Self {
            repetitions,
            fault,
            marker: PhantomData,
        }
    }

    fn small_const(n: u64) -> F {
        F::from(n)
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

    fn split_u31(v: u32) -> (u16, u16) {
        let lo = (v & 0xffff) as u16;
        let hi = (v >> 16) as u16; // <= 0x7fff
        (lo, hi)
    }

    fn split_u66(v: u128) -> (u16, u16, u16, u16, u8) {
        let q0 = (v & 0xffff) as u16;
        let q1 = ((v >> 16) & 0xffff) as u16;
        let q2 = ((v >> 32) & 0xffff) as u16;
        let q3 = ((v >> 48) & 0xffff) as u16;
        let q4 = ((v >> 64) & 0x3) as u8;
        (q0, q1, q2, q3, q4)
    }
}

impl<F: Field + From<u64>> Circuit<F> for BaselineBNoteCircuit<F> {
    type Config = BaselineBNoteConfig;
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

        let x0 = meta.advice_column();
        let x1 = meta.advice_column();
        let x = meta.advice_column();
        let nu_x = meta.advice_column();

        let y0 = meta.advice_column();
        let y1 = meta.advice_column();
        let y = meta.advice_column();
        let nu_y = meta.advice_column();

        let z0 = meta.advice_column();
        let z1 = meta.advice_column();
        let z = meta.advice_column();
        let nu_z = meta.advice_column();

        let q0 = meta.advice_column();
        let q1 = meta.advice_column();
        let q2 = meta.advice_column();
        let q3 = meta.advice_column();
        let q4 = meta.advice_column();
        let q = meta.advice_column();

        let digest = meta.advice_column();

        let t16 = meta.lookup_table_column();
        let t15 = meta.lookup_table_column();
        let t2 = meta.lookup_table_column();

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
        let q66 = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, F>| {
            meta.query_selector(sel_inv) + meta.query_selector(sel_ee) + meta.query_selector(sel_hor)
        };

        // Canonical decomposition + non-equality-to-p for x,y,z on all active rows.
        meta.create_gate("B_note canonical binding + non-equality", |meta| {
            let qa = q_active(meta);

            let x0e = meta.query_advice(x0, Rotation::cur());
            let x1e = meta.query_advice(x1, Rotation::cur());
            let xe = meta.query_advice(x, Rotation::cur());
            let nuxe = meta.query_advice(nu_x, Rotation::cur());

            let y0e = meta.query_advice(y0, Rotation::cur());
            let y1e = meta.query_advice(y1, Rotation::cur());
            let ye = meta.query_advice(y, Rotation::cur());
            let nuye = meta.query_advice(nu_y, Rotation::cur());

            let z0e = meta.query_advice(z0, Rotation::cur());
            let z1e = meta.query_advice(z1, Rotation::cur());
            let ze = meta.query_advice(z, Rotation::cur());
            let nuze = meta.query_advice(nu_z, Rotation::cur());

            let two16 = Expression::Constant(Self::fr_pow2(16));
            let p = Expression::Constant(Self::small_const(P_U32 as u64));
            let one = Expression::Constant(F::ONE);

            vec![
                qa.clone() * (xe.clone() - (x0e + two16.clone() * x1e)),
                qa.clone() * (ye.clone() - (y0e + two16.clone() * y1e)),
                qa.clone() * (ze.clone() - (z0e + two16.clone() * z1e)),
                qa.clone() * ((xe - p.clone()) * nuxe - one.clone()),
                qa.clone() * ((ye - p.clone()) * nuye - one.clone()),
                qa.clone() * ((ze - p) * nuze - one),
            ]
        });

        // Quotient recomposition for class31 and class66 rows.
        meta.create_gate("B_note quotient/carry range binding", |meta| {
            let q31e = q31(meta);
            let q66e = q66(meta);

            let q0e = meta.query_advice(q0, Rotation::cur());
            let q1e = meta.query_advice(q1, Rotation::cur());
            let q2e = meta.query_advice(q2, Rotation::cur());
            let q3e = meta.query_advice(q3, Rotation::cur());
            let q4e = meta.query_advice(q4, Rotation::cur());
            let qe = meta.query_advice(q, Rotation::cur());

            let two16 = Expression::Constant(Self::fr_pow2(16));
            let two32 = Expression::Constant(Self::fr_pow2(32));
            let two48 = Expression::Constant(Self::fr_pow2(48));
            let two64 = Expression::Constant(Self::fr_pow2(64));

            vec![
                q31e.clone() * (qe.clone() - (q0e.clone() + two16.clone() * q1e.clone())),
                q31e * (q2e.clone() + q3e.clone() + q4e.clone()),
                q66e * (qe - (q0e + two16 * q1e + two32 * q2e + two48 * q3e + two64 * q4e)),
            ]
        });

        // Family relations.
        meta.create_gate("B_note family equations", |meta| {
            let inv = meta.query_selector(sel_inv);
            let ef = meta.query_selector(sel_ef);
            let ee = meta.query_selector(sel_ee);
            let hor = meta.query_selector(sel_hor);
            let car = meta.query_selector(sel_car);

            let xe = meta.query_advice(x, Rotation::cur());
            let ye = meta.query_advice(y, Rotation::cur());
            let ze = meta.query_advice(z, Rotation::cur());
            let qe = meta.query_advice(q, Rotation::cur());

            let coeff = Expression::Constant(Self::small_const(EE_HOR_COEFF_U64));
            let five = Expression::Constant(Self::small_const(5));

            vec![
                inv * (qe * xe.clone() - xe.clone()),
                ef * (xe.clone() * ye.clone() - ze.clone()),
                ee * (coeff.clone() * xe.clone() * ye.clone() - ze.clone()),
                hor * (coeff * xe.clone() * ye - ze.clone()),
                car * (xe + meta.query_advice(y, Rotation::cur()) + five - ze),
            ]
        });

        // Digest binding for active rows.
        meta.create_gate("B_note digest binding", |meta| {
            let qa = q_active(meta);
            let d = meta.query_advice(digest, Rotation::cur());
            let expected = Expression::Constant(Self::small_const(DIGEST_CONST_U64));
            vec![qa * (d - expected)]
        });

        // Inactive-row zero extension.
        meta.create_gate("B_note inactive-row zero extension", |meta| {
            let qz = meta.query_selector(sel_inactive);
            let sum = meta.query_advice(x0, Rotation::cur())
                + meta.query_advice(x1, Rotation::cur())
                + meta.query_advice(x, Rotation::cur())
                + meta.query_advice(nu_x, Rotation::cur())
                + meta.query_advice(y0, Rotation::cur())
                + meta.query_advice(y1, Rotation::cur())
                + meta.query_advice(y, Rotation::cur())
                + meta.query_advice(nu_y, Rotation::cur())
                + meta.query_advice(z0, Rotation::cur())
                + meta.query_advice(z1, Rotation::cur())
                + meta.query_advice(z, Rotation::cur())
                + meta.query_advice(nu_z, Rotation::cur())
                + meta.query_advice(q0, Rotation::cur())
                + meta.query_advice(q1, Rotation::cur())
                + meta.query_advice(q2, Rotation::cur())
                + meta.query_advice(q3, Rotation::cur())
                + meta.query_advice(q4, Rotation::cur())
                + meta.query_advice(q, Rotation::cur())
                + meta.query_advice(digest, Rotation::cur());
            vec![qz * sum]
        });

        // Canonical lookups for x/y/z.
        meta.lookup("x0 in T16", |meta| {
            let qa = q_active(meta);
            let v = meta.query_advice(x0, Rotation::cur());
            vec![(qa * v, t16)]
        });
        meta.lookup("x1 in T15", |meta| {
            let qa = q_active(meta);
            let v = meta.query_advice(x1, Rotation::cur());
            vec![(qa * v, t15)]
        });
        meta.lookup("y0 in T16", |meta| {
            let qa = q_active(meta);
            let v = meta.query_advice(y0, Rotation::cur());
            vec![(qa * v, t16)]
        });
        meta.lookup("y1 in T15", |meta| {
            let qa = q_active(meta);
            let v = meta.query_advice(y1, Rotation::cur());
            vec![(qa * v, t15)]
        });
        meta.lookup("z0 in T16", |meta| {
            let qa = q_active(meta);
            let v = meta.query_advice(z0, Rotation::cur());
            vec![(qa * v, t16)]
        });
        meta.lookup("z1 in T15", |meta| {
            let qa = q_active(meta);
            let v = meta.query_advice(z1, Rotation::cur());
            vec![(qa * v, t15)]
        });

        // q31 lookups.
        meta.lookup("q31 q0 in T16", |meta| {
            let q31e = q31(meta);
            let v = meta.query_advice(q0, Rotation::cur());
            vec![(q31e * v, t16)]
        });
        meta.lookup("q31 q1 in T15", |meta| {
            let q31e = q31(meta);
            let v = meta.query_advice(q1, Rotation::cur());
            vec![(q31e * v, t15)]
        });

        // q66 lookups.
        meta.lookup("q66 q0 in T16", |meta| {
            let q66e = q66(meta);
            let v = meta.query_advice(q0, Rotation::cur());
            vec![(q66e * v, t16)]
        });
        meta.lookup("q66 q1 in T16", |meta| {
            let q66e = q66(meta);
            let v = meta.query_advice(q1, Rotation::cur());
            vec![(q66e * v, t16)]
        });
        meta.lookup("q66 q2 in T16", |meta| {
            let q66e = q66(meta);
            let v = meta.query_advice(q2, Rotation::cur());
            vec![(q66e * v, t16)]
        });
        meta.lookup("q66 q3 in T16", |meta| {
            let q66e = q66(meta);
            let v = meta.query_advice(q3, Rotation::cur());
            vec![(q66e * v, t16)]
        });
        meta.lookup("q66 q4 in T2", |meta| {
            let q66e = q66(meta);
            let v = meta.query_advice(q4, Rotation::cur());
            vec![(q66e * v, t2)]
        });

        BaselineBNoteConfig {
            sel_inv,
            sel_ef,
            sel_ee,
            sel_hor,
            sel_car,
            sel_inactive,

            x0,
            x1,
            x,
            nu_x,

            y0,
            y1,
            y,
            nu_y,

            z0,
            z1,
            z,
            nu_z,

            q0,
            q1,
            q2,
            q3,
            q4,
            q,

            digest,
            t16,
            t15,
            t2,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // Range tables required by canonical + quotient bindings.
        layouter.assign_table(
            || "T16",
            |mut table| {
                for i in 0..T16_TABLE_SIZE {
                    table.assign_cell(
                        || format!("t16_{i}"),
                        config.t16,
                        i as usize,
                        || Value::known(Self::small_const(i as u64)),
                    )?;
                }
                Ok(())
            },
        )?;
        layouter.assign_table(
            || "T15",
            |mut table| {
                for i in 0..T15_TABLE_SIZE {
                    table.assign_cell(
                        || format!("t15_{i}"),
                        config.t15,
                        i as usize,
                        || Value::known(Self::small_const(i as u64)),
                    )?;
                }
                Ok(())
            },
        )?;
        layouter.assign_table(
            || "T2",
            |mut table| {
                for i in 0..4u32 {
                    table.assign_cell(
                        || format!("t2_{i}"),
                        config.t2,
                        i as usize,
                        || Value::known(Self::small_const(i as u64)),
                    )?;
                }
                Ok(())
            },
        )?;

        layouter.assign_region(
            || "B_note family rows",
            |mut region| {
                let mut global_row = 0usize;
                let mut fault_applied = false;

                for rep in 0..self.repetitions {
                    for (family, count) in Self::families() {
                        for local_idx in 0..count {
                            let offset = global_row;
                            global_row += 1;

                            let q_is_66 = matches!(family, Family::Inv | Family::Ee | Family::Hor);
                            // Keep witnesses within deterministic small bounds so lookup ranges
                            // stay valid across large repetition counts during fixed-k sweeps.
                            let base = ((rep as u32) * 17 + local_idx as u32) % 100;

                            let (mut x_u32, mut y_u32, mut z_u32, mut q_u128) = match family {
                                Family::Inv => {
                                    let x = 1000 + base;
                                    // q*x = x with q=1.
                                    (x, 1, x, 1u128)
                                }
                                Family::Ef => {
                                    let x = 70 + base;
                                    let y = 2;
                                    let z = x.saturating_mul(y);
                                    (x, y, z, 123 + base as u128)
                                }
                                Family::Ee => {
                                    // Keep y=0 so z=0 remains canonical under large coefficient.
                                    let x = 77 + base;
                                    (x, 0, 0, 1u128)
                                }
                                Family::Hor => {
                                    let x = 211 + base;
                                    (x, 0, 0, 2u128)
                                }
                                Family::Car => {
                                    let x = 33 + base;
                                    let y = 55 + base;
                                    let z = x + y + 5;
                                    (x, y, z, 222 + base as u128)
                                }
                            };

                            let mut digest = DIGEST_CONST_U64;
                            if !fault_applied {
                                match self.fault {
                                    Some(BNoteFault::PInvWitnessToyAttack) if family == Family::Inv => {
                                        x_u32 = P_U32;
                                        y_u32 = 1;
                                        z_u32 = P_U32;
                                        fault_applied = true;
                                    }
                                    Some(BNoteFault::OmittedQuotientWiring) if family == Family::Inv => {
                                        q_u128 = 1u128 + 5;
                                        fault_applied = true;
                                    }
                                    Some(BNoteFault::ClassMapMismatch31_66) if family == Family::Ef => {
                                        // Applied below by forcing q4!=0 on class31 row.
                                        fault_applied = true;
                                    }
                                    Some(BNoteFault::DigestMismatch) if family == Family::Inv => {
                                        digest = DIGEST_CONST_U64 + 1;
                                        fault_applied = true;
                                    }
                                    _ => {}
                                }
                            }

                            // Prevent x==p for valid rows (except explicit p^{-1} fault case).
                            if x_u32 == P_U32 && self.fault != Some(BNoteFault::PInvWitnessToyAttack) {
                                x_u32 = P_U32 - 1;
                            }
                            if y_u32 == P_U32 {
                                y_u32 = P_U32 - 1;
                            }
                            if z_u32 == P_U32 {
                                z_u32 = P_U32 - 1;
                            }

                            let (x0, x1) = Self::split_u31(x_u32);
                            let (y0, y1) = Self::split_u31(y_u32);
                            let (z0, z1) = Self::split_u31(z_u32);

                            let p_f = Self::small_const(P_U32 as u64);
                            let x_f = Self::small_const(x_u32 as u64);
                            let y_f = Self::small_const(y_u32 as u64);
                            let z_f = Self::small_const(z_u32 as u64);

                            let nu_x = (x_f - p_f).invert().unwrap_or(F::ZERO);
                            let nu_y = (y_f - p_f).invert().unwrap_or(F::ZERO);
                            let nu_z = (z_f - p_f).invert().unwrap_or(F::ZERO);

                            let (q0, q1, q2, q3, mut q4) = if q_is_66 {
                                Self::split_u66(q_u128)
                            } else {
                                let q31 = q_u128 as u32;
                                let (a, b) = Self::split_u31(q31);
                                (a, b, 0, 0, 0)
                            };

                            // Fault: class-map mismatch on class31 row by introducing non-zero q4.
                            if self.fault == Some(BNoteFault::ClassMapMismatch31_66) && family == Family::Ef {
                                q4 = 1;
                            }

                            let mut q_f = if q_is_66 {
                                let q_low = q_u128 as u64;
                                let q_hi = (q_u128 >> 64) as u64;
                                Self::small_const(q_low) + Self::small_const(q_hi) * Self::fr_pow2(64)
                            } else {
                                Self::small_const((q_u128 as u32) as u64)
                            };
                            if self.fault == Some(BNoteFault::OmittedQuotientWiring) && family == Family::Inv {
                                q_f += F::ONE;
                            }

                            // Enable family selector.
                            match family {
                                Family::Inv => config.sel_inv.enable(&mut region, offset)?,
                                Family::Ef => config.sel_ef.enable(&mut region, offset)?,
                                Family::Ee => config.sel_ee.enable(&mut region, offset)?,
                                Family::Hor => config.sel_hor.enable(&mut region, offset)?,
                                Family::Car => config.sel_car.enable(&mut region, offset)?,
                            }

                            region.assign_advice(|| format!("x0_r{rep}_{offset}"), config.x0, offset, || Value::known(Self::small_const(x0 as u64)))?;
                            region.assign_advice(|| format!("x1_r{rep}_{offset}"), config.x1, offset, || Value::known(Self::small_const(x1 as u64)))?;
                            region.assign_advice(|| format!("x_r{rep}_{offset}"), config.x, offset, || Value::known(x_f))?;
                            region.assign_advice(|| format!("nu_x_r{rep}_{offset}"), config.nu_x, offset, || Value::known(nu_x))?;

                            region.assign_advice(|| format!("y0_r{rep}_{offset}"), config.y0, offset, || Value::known(Self::small_const(y0 as u64)))?;
                            region.assign_advice(|| format!("y1_r{rep}_{offset}"), config.y1, offset, || Value::known(Self::small_const(y1 as u64)))?;
                            region.assign_advice(|| format!("y_r{rep}_{offset}"), config.y, offset, || Value::known(y_f))?;
                            region.assign_advice(|| format!("nu_y_r{rep}_{offset}"), config.nu_y, offset, || Value::known(nu_y))?;

                            region.assign_advice(|| format!("z0_r{rep}_{offset}"), config.z0, offset, || Value::known(Self::small_const(z0 as u64)))?;
                            region.assign_advice(|| format!("z1_r{rep}_{offset}"), config.z1, offset, || Value::known(Self::small_const(z1 as u64)))?;
                            region.assign_advice(|| format!("z_r{rep}_{offset}"), config.z, offset, || Value::known(z_f))?;
                            region.assign_advice(|| format!("nu_z_r{rep}_{offset}"), config.nu_z, offset, || Value::known(nu_z))?;

                            region.assign_advice(|| format!("q0_r{rep}_{offset}"), config.q0, offset, || Value::known(Self::small_const(q0 as u64)))?;
                            region.assign_advice(|| format!("q1_r{rep}_{offset}"), config.q1, offset, || Value::known(Self::small_const(q1 as u64)))?;
                            region.assign_advice(|| format!("q2_r{rep}_{offset}"), config.q2, offset, || Value::known(Self::small_const(q2 as u64)))?;
                            region.assign_advice(|| format!("q3_r{rep}_{offset}"), config.q3, offset, || Value::known(Self::small_const(q3 as u64)))?;
                            region.assign_advice(|| format!("q4_r{rep}_{offset}"), config.q4, offset, || Value::known(Self::small_const(q4 as u64)))?;
                            region.assign_advice(|| format!("q_r{rep}_{offset}"), config.q, offset, || Value::known(q_f))?;

                            region.assign_advice(
                                || format!("digest_r{rep}_{offset}"),
                                config.digest,
                                offset,
                                || Value::known(Self::small_const(digest)),
                            )?;
                        }
                    }

                    // Inactive row per repetition.
                    let inactive_offset = global_row;
                    global_row += 1;
                    config.sel_inactive.enable(&mut region, inactive_offset)?;

                    let mut inactive_x = F::ZERO;
                    if self.fault == Some(BNoteFault::InactiveRowZeroExtensionViolation) && !fault_applied {
                        inactive_x = F::ONE;
                        fault_applied = true;
                    }

                    region.assign_advice(|| format!("inactive_x0_{rep}"), config.x0, inactive_offset, || Value::known(inactive_x))?;
                    region.assign_advice(|| format!("inactive_x1_{rep}"), config.x1, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_x_{rep}"), config.x, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_nu_x_{rep}"), config.nu_x, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_y0_{rep}"), config.y0, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_y1_{rep}"), config.y1, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_y_{rep}"), config.y, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_nu_y_{rep}"), config.nu_y, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_z0_{rep}"), config.z0, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_z1_{rep}"), config.z1, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_z_{rep}"), config.z, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_nu_z_{rep}"), config.nu_z, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_q0_{rep}"), config.q0, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_q1_{rep}"), config.q1, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_q2_{rep}"), config.q2, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_q3_{rep}"), config.q3, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_q4_{rep}"), config.q4, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_q_{rep}"), config.q, inactive_offset, || Value::known(F::ZERO))?;
                    region.assign_advice(|| format!("inactive_digest_{rep}"), config.digest, inactive_offset, || Value::known(F::ZERO))?;
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BNoteFault, BaselineBNoteCircuit};
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr;

    fn assert_rejected(circuit: BaselineBNoteCircuit<Fr>) {
        let k = 13;
        let prover = MockProver::run(k, &circuit, vec![]).expect("mock prover should build");
        assert!(
            prover.verify().is_err(),
            "expected constraint system to reject invalid witness"
        );
    }

    #[test]
    fn test_b_note_valid() {
        let circuit = BaselineBNoteCircuit::<Fr>::new(1);
        let k = 13;
        let prover = MockProver::run(k, &circuit, vec![]).expect("mock prover should build");
        prover.assert_satisfied();
    }

    #[test]
    fn test_b_note_rejects_p_inv_witness_toy_attack() {
        assert_rejected(BaselineBNoteCircuit::<Fr>::with_fault(
            1,
            Some(BNoteFault::PInvWitnessToyAttack),
        ));
    }

    #[test]
    fn test_b_note_rejects_omitted_quotient_wiring() {
        assert_rejected(BaselineBNoteCircuit::<Fr>::with_fault(
            1,
            Some(BNoteFault::OmittedQuotientWiring),
        ));
    }

    #[test]
    fn test_b_note_rejects_class_map_mismatch_31_66() {
        assert_rejected(BaselineBNoteCircuit::<Fr>::with_fault(
            1,
            Some(BNoteFault::ClassMapMismatch31_66),
        ));
    }

    #[test]
    fn test_b_note_rejects_digest_mismatch() {
        assert_rejected(BaselineBNoteCircuit::<Fr>::with_fault(
            1,
            Some(BNoteFault::DigestMismatch),
        ));
    }

    #[test]
    fn test_b_note_rejects_inactive_row_zero_extension_violation() {
        assert_rejected(BaselineBNoteCircuit::<Fr>::with_fault(
            1,
            Some(BNoteFault::InactiveRowZeroExtensionViolation),
        ));
    }
}
