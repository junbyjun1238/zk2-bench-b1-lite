use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector, TableColumn},
    poly::Rotation,
};
use halo2curves::{bn256::Fr, ff::Field};

// Bench-lite harness: build *real* Halo2 constraints/lookups corresponding to
// the paper's primitive building blocks (repair fragment) and a baseline 256-bit
// multi-limb multiplication with explicit cross-products + carry chain.

fn fr_from_u64(x: u64) -> Fr {
    Fr::from(x)
}

fn pow2_u64(k: u32) -> u64 {
    1u64 << k
}

fn fr_pow2(k: u32) -> Fr {
    // 2^k in Fr.
    Fr::from(2u64).pow_vartime([k as u64, 0, 0, 0])
}

// -------------------------
// Shared lookup tables.
// -------------------------

#[derive(Clone, Debug)]
struct LookupTables {
    t16: TableColumn,
    t15: TableColumn,
    t8: TableColumn,
    t5: TableColumn,
    t2: TableColumn,
}

fn configure_tables(meta: &mut ConstraintSystem<Fr>) -> LookupTables {
    LookupTables {
        t16: meta.lookup_table_column(),
        t15: meta.lookup_table_column(),
        t8: meta.lookup_table_column(),
        t5: meta.lookup_table_column(),
        t2: meta.lookup_table_column(),
    }
}

fn load_tables(layouter: &mut impl Layouter<Fr>, t: &LookupTables) -> Result<(), Error> {
    layouter.assign_table(|| "T_16", |mut table| {
        for i in 0..(1u32 << 16) {
            table.assign_cell(
                || "t16",
                t.t16,
                i as usize,
                || Value::known(fr_from_u64(i as u64)),
            )?;
        }
        Ok(())
    })?;

    layouter.assign_table(|| "T_15", |mut table| {
        for i in 0..(1u32 << 15) {
            table.assign_cell(
                || "t15",
                t.t15,
                i as usize,
                || Value::known(fr_from_u64(i as u64)),
            )?;
        }
        Ok(())
    })?;

    layouter.assign_table(|| "T_8", |mut table| {
        for i in 0..8u32 {
            table.assign_cell(
                || "t8",
                t.t8,
                i as usize,
                || Value::known(fr_from_u64(i as u64)),
            )?;
        }
        Ok(())
    })?;

    layouter.assign_table(|| "T_5", |mut table| {
        for i in 0..32u32 {
            table.assign_cell(
                || "t5",
                t.t5,
                i as usize,
                || Value::known(fr_from_u64(i as u64)),
            )?;
        }
        Ok(())
    })?;

    layouter.assign_table(|| "T_2", |mut table| {
        for i in 0..4u32 {
            table.assign_cell(
                || "t2",
                t.t2,
                i as usize,
                || Value::known(fr_from_u64(i as u64)),
            )?;
        }
        Ok(())
    })?;

    Ok(())
}

// ------------------------------------------------------------
// Repair fragment circuit (Option2 note primitive layer).
// ------------------------------------------------------------

#[derive(Clone, Debug)]
struct RepairConfig {
    adv: [Column<Advice>; 8],
    q_residue: Selector,
    q_q31: Selector,
    q_q66: Selector,
    q_bool: Selector,
    q_u8: Selector,
    tables: LookupTables,
}

#[derive(Clone, Debug)]
struct RepairCircuit {
    residues: Vec<u32>, // 44
    q31: Vec<u32>,      // 4 (carry-normalization add-on, optional)
    q66: Vec<u128>,     // 12
    bools: Vec<u8>,     // 12
    u8s: Vec<u8>,       // 4
}

impl RepairCircuit {
    fn new_fragment() -> Self {
        let p: u32 = (1u32 << 31) - 1; // M31 prime

        let residues: Vec<u32> = (0..44u32).map(|i| (i * 1337) % (p - 1)).collect();
        let q31: Vec<u32> = Vec::new();
        let q66: Vec<u128> = (0..12u128)
            .map(|i| (i * 0x1_2345_6789u128) & ((1u128 << 66) - 1))
            .collect();
        let bools: Vec<u8> = (0..12u8).map(|i| i & 1).collect();
        let u8s: Vec<u8> = (0..4u8).map(|i| (i * 2) & 7).collect();

        Self { residues, q31, q66, bools, u8s }
    }

    fn new_paired_package() -> Self {
        let mut c = Self::new_fragment();
        // 4 carry-normalization quotient cells in [0, 2^31).
        c.q31 = (0..4u32).map(|i| (i * 100_003 + 9) & ((1u32 << 31) - 1)).collect();
        c
    }
}

impl Circuit<Fr> for RepairCircuit {
    type Config = RepairConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            residues: vec![0; 44],
            q31: vec![0; self.q31.len()],
            q66: vec![0; 12],
            bools: vec![0; 12],
            u8s: vec![0; 4],
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let adv = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        for c in adv.iter() {
            meta.enable_equality(*c);
        }

        let tables = configure_tables(meta);

        let q_residue = meta.complex_selector();
        let q_q31 = meta.complex_selector();
        let q_q66 = meta.complex_selector();
        let q_bool = meta.selector();
        let q_u8 = meta.complex_selector();

        // Residue row:
        // adv[0]=x0 in T16, adv[1]=x1 in T15, adv[2]=x, adv[3]=nu.
        // Constraints:
        //  (1) x = x0 + 2^16 x1
        //  (2) (x - p) * nu = 1  (non-equality-to-p)
        meta.create_gate("residue recomposition + non-equality", |meta| {
            let q = meta.query_selector(q_residue);
            let x0 = meta.query_advice(adv[0], Rotation::cur());
            let x1 = meta.query_advice(adv[1], Rotation::cur());
            let x = meta.query_advice(adv[2], Rotation::cur());
            let nu = meta.query_advice(adv[3], Rotation::cur());

            let p = fr_from_u64((1u64 << 31) - 1);
            let two16 = fr_from_u64(pow2_u64(16));

            let lin = x.clone() - (x0 + Expression::Constant(two16) * x1);
            let neq = (x - Expression::Constant(p)) * nu - Expression::Constant(fr_from_u64(1));
            vec![q.clone() * lin, q * neq]
        });

        // Residue limb lookups.
        meta.lookup(|meta| {
            let q = meta.query_selector(q_residue);
            let x0 = meta.query_advice(adv[0], Rotation::cur());
            vec![(q * x0, tables.t16)]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_residue);
            let x1 = meta.query_advice(adv[1], Rotation::cur());
            vec![(q * x1, tables.t15)]
        });

        // q31 row (carry-normalization add-on):
        // adv[0]=q0 in T16, adv[1]=q1 in T15, adv[2]=q.
        meta.create_gate("q31 recomposition", |meta| {
            let qsel = meta.query_selector(q_q31);
            let q0 = meta.query_advice(adv[0], Rotation::cur());
            let q1 = meta.query_advice(adv[1], Rotation::cur());
            let qv = meta.query_advice(adv[2], Rotation::cur());
            let two16 = fr_from_u64(pow2_u64(16));
            vec![qsel * (qv - (q0 + Expression::Constant(two16) * q1))]
        });
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q31);
            let q0 = meta.query_advice(adv[0], Rotation::cur());
            vec![(qsel * q0, tables.t16)]
        });
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q31);
            let q1 = meta.query_advice(adv[1], Rotation::cur());
            vec![(qsel * q1, tables.t15)]
        });

        // q66 row:
        // adv[0..3]=q0..q3 in T16, adv[4]=q4 in T2, adv[5]=q.
        meta.create_gate("q66 recomposition", |meta| {
            let qsel = meta.query_selector(q_q66);
            let q0 = meta.query_advice(adv[0], Rotation::cur());
            let q1 = meta.query_advice(adv[1], Rotation::cur());
            let q2 = meta.query_advice(adv[2], Rotation::cur());
            let q3 = meta.query_advice(adv[3], Rotation::cur());
            let q4 = meta.query_advice(adv[4], Rotation::cur());
            let qv = meta.query_advice(adv[5], Rotation::cur());

            let two16 = fr_from_u64(pow2_u64(16));
            let two32 = fr_from_u64(pow2_u64(32));
            let two48 = fr_from_u64(pow2_u64(48));
            let two64 = fr_pow2(64);

            let recomposed =
                q0
                + Expression::Constant(two16) * q1
                + Expression::Constant(two32) * q2
                + Expression::Constant(two48) * q3
                + Expression::Constant(two64) * q4;

            vec![qsel * (qv - recomposed)]
        });

        // q66 limb lookups.
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q66);
            let limb = meta.query_advice(adv[0], Rotation::cur());
            vec![(qsel * limb, tables.t16)]
        });
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q66);
            let limb = meta.query_advice(adv[1], Rotation::cur());
            vec![(qsel * limb, tables.t16)]
        });
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q66);
            let limb = meta.query_advice(adv[2], Rotation::cur());
            vec![(qsel * limb, tables.t16)]
        });
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q66);
            let limb = meta.query_advice(adv[3], Rotation::cur());
            vec![(qsel * limb, tables.t16)]
        });
        meta.lookup(|meta| {
            let qsel = meta.query_selector(q_q66);
            let top = meta.query_advice(adv[4], Rotation::cur());
            vec![(qsel * top, tables.t2)]
        });

        // Boolean row: adv[0]=c, constraint c(c-1)=0.
        meta.create_gate("booleanity", |meta| {
            let q = meta.query_selector(q_bool);
            let c = meta.query_advice(adv[0], Rotation::cur());
            vec![q * c.clone() * (c - Expression::Constant(fr_from_u64(1)))]
        });

        // u8 row: adv[0]=u in T8.
        meta.lookup(|meta| {
            let q = meta.query_selector(q_u8);
            let u = meta.query_advice(adv[0], Rotation::cur());
            vec![(q * u, tables.t8)]
        });

        RepairConfig { adv, q_residue, q_q31, q_q66, q_bool, q_u8, tables }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fr>) -> Result<(), Error> {
        load_tables(&mut layouter, &config.tables)?;

        let p_u32: u32 = (1u32 << 31) - 1;
        let two64 = fr_pow2(64);

        layouter.assign_region(|| "repair fragment rows", |mut region| {
            let mut offset = 0usize;

            // 44 residue rows.
            for &x in self.residues.iter() {
                config.q_residue.enable(&mut region, offset)?;

                let x0 = (x & 0xffff) as u64;
                let x1 = (x >> 16) as u64; // x < 2^31 => x1 < 2^15
                let x_fe = fr_from_u64(x as u64);

                // nu = (x - p)^{-1}, and x != p is ensured by construction.
                assert!(x != p_u32);
                let denom = Fr::from(x as u64) - Fr::from(p_u32 as u64);
                let nu = denom.invert().unwrap();

                region.assign_advice(|| "x0", config.adv[0], offset, || Value::known(fr_from_u64(x0)))?;
                region.assign_advice(|| "x1", config.adv[1], offset, || Value::known(fr_from_u64(x1)))?;
                region.assign_advice(|| "x", config.adv[2], offset, || Value::known(x_fe))?;
                region.assign_advice(|| "nu", config.adv[3], offset, || Value::known(nu))?;

                for col in config.adv.iter().skip(4) {
                    region.assign_advice(|| "pad", *col, offset, || Value::known(fr_from_u64(0)))?;
                }

                offset += 1;
            }

            // Optional 4 q31 rows (paired-package add-on).
            for &q in self.q31.iter() {
                config.q_q31.enable(&mut region, offset)?;

                let q0 = (q & 0xffff) as u64;
                let q1 = (q >> 16) as u64; // <= 2^15-1 since q < 2^31
                let q_fe = fr_from_u64(q as u64);

                region.assign_advice(|| "q31_0", config.adv[0], offset, || Value::known(fr_from_u64(q0)))?;
                region.assign_advice(|| "q31_1", config.adv[1], offset, || Value::known(fr_from_u64(q1)))?;
                region.assign_advice(|| "q31", config.adv[2], offset, || Value::known(q_fe))?;
                for col in config.adv.iter().skip(3) {
                    region.assign_advice(|| "pad", *col, offset, || Value::known(fr_from_u64(0)))?;
                }

                offset += 1;
            }

            // 12 q66 rows.
            for &q in self.q66.iter() {
                config.q_q66.enable(&mut region, offset)?;

                let q0 = (q & 0xffff) as u64;
                let q1 = ((q >> 16) & 0xffff) as u64;
                let q2 = ((q >> 32) & 0xffff) as u64;
                let q3 = ((q >> 48) & 0xffff) as u64;
                let q4 = ((q >> 64) & 0x3) as u64; // 2 bits

                let low64 = (q & 0xffff_ffff_ffff_ffffu128) as u64;
                let high64 = (q >> 64) as u64; // <= 3
                let q_fe = Fr::from(low64) + Fr::from(high64) * two64;

                region.assign_advice(|| "q0", config.adv[0], offset, || Value::known(fr_from_u64(q0)))?;
                region.assign_advice(|| "q1", config.adv[1], offset, || Value::known(fr_from_u64(q1)))?;
                region.assign_advice(|| "q2", config.adv[2], offset, || Value::known(fr_from_u64(q2)))?;
                region.assign_advice(|| "q3", config.adv[3], offset, || Value::known(fr_from_u64(q3)))?;
                region.assign_advice(|| "q4", config.adv[4], offset, || Value::known(fr_from_u64(q4)))?;
                region.assign_advice(|| "q", config.adv[5], offset, || Value::known(q_fe))?;

                region.assign_advice(|| "pad", config.adv[6], offset, || Value::known(fr_from_u64(0)))?;
                region.assign_advice(|| "pad", config.adv[7], offset, || Value::known(fr_from_u64(0)))?;

                offset += 1;
            }

            // 12 boolean rows.
            for &c in self.bools.iter() {
                config.q_bool.enable(&mut region, offset)?;
                region.assign_advice(|| "c", config.adv[0], offset, || Value::known(fr_from_u64(c as u64)))?;
                for col in config.adv.iter().skip(1) {
                    region.assign_advice(|| "pad", *col, offset, || Value::known(fr_from_u64(0)))?;
                }
                offset += 1;
            }

            // 4 u8 rows.
            for &u in self.u8s.iter() {
                config.q_u8.enable(&mut region, offset)?;
                region.assign_advice(|| "u", config.adv[0], offset, || Value::known(fr_from_u64(u as u64)))?;
                for col in config.adv.iter().skip(1) {
                    region.assign_advice(|| "pad", *col, offset, || Value::known(fr_from_u64(0)))?;
                }
                offset += 1;
            }

            Ok(())
        })?;

        Ok(())
    }
}

// ------------------------------------------------------------
// Baseline 256-bit bigfield multiplication (16 x 16-bit limbs).
// ------------------------------------------------------------

#[derive(Clone, Debug)]
struct BigMulConfig {
    // Range-checked u16 limb storage.
    limb_u16: Column<Advice>,
    q_u16: Selector,

    // Carry decomposition (15-bit + 5-bit) + recomposition.
    carry_lo15: Column<Advice>,
    carry_hi5: Column<Advice>,
    carry_val: Column<Advice>,
    q_carry: Selector,

    // Product rows: a,b,prod.
    a: Column<Advice>,
    b: Column<Advice>,
    prod: Column<Advice>,
    q_prod: Selector,

    // Block-start equation cells.
    carry_in: Column<Advice>,
    out_limb: Column<Advice>,
    carry_out: Column<Advice>,
    q_sum: [Selector; 16], // length 1..16

    // Final limb row: c[31] = carry[31].
    q_last: Selector,

    tables: LookupTables,
}

#[derive(Clone, Debug)]
struct BigMulCircuit {
    a: [u16; 16],
    b: [u16; 16],
}

impl BigMulCircuit {
    fn new() -> Self {
        let a = core::array::from_fn(|i| ((i as u32 * 1009 + 7) & 0xffff) as u16);
        let b = core::array::from_fn(|i| ((i as u32 * 917 + 11) & 0xffff) as u16);
        Self { a, b }
    }

    fn schoolbook_product(&self) -> ([u16; 32], [u32; 32]) {
        // Base B = 2^16.
        let base: u64 = 1u64 << 16;

        let mut carry: [u32; 32] = [0u32; 32];
        let mut c: [u16; 32] = [0u16; 32];

        // k = 0..30 have direct product terms.
        for k in 0..31usize {
            let mut sum: u64 = carry[k] as u64;
            for i in 0..16usize {
                if k >= i {
                    let j = k - i;
                    if j < 16 {
                        sum += (self.a[i] as u64) * (self.b[j] as u64);
                    }
                }
            }
            c[k] = (sum % base) as u16;
            carry[k + 1] = (sum / base) as u32;
        }

        // Final limb is the final carry; must fit in u16.
        assert!((carry[31] as u64) < base);
        c[31] = carry[31] as u16;

        (c, carry)
    }
}

impl Circuit<Fr> for BigMulCircuit {
    type Config = BigMulConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { a: [0u16; 16], b: [0u16; 16] }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let limb_u16 = meta.advice_column();
        meta.enable_equality(limb_u16);
        let q_u16 = meta.complex_selector();

        let carry_lo15 = meta.advice_column();
        let carry_hi5 = meta.advice_column();
        let carry_val = meta.advice_column();
        meta.enable_equality(carry_val);
        let q_carry = meta.complex_selector();

        let a = meta.advice_column();
        let b = meta.advice_column();
        let prod = meta.advice_column();
        meta.enable_equality(a);
        meta.enable_equality(b);
        let q_prod = meta.selector();

        let carry_in = meta.advice_column();
        let out_limb = meta.advice_column();
        let carry_out = meta.advice_column();
        meta.enable_equality(carry_in);
        meta.enable_equality(out_limb);
        meta.enable_equality(carry_out);

        let q_sum: [Selector; 16] = core::array::from_fn(|_| meta.selector());
        let q_last = meta.selector();

        let tables = configure_tables(meta);

        // u16 range lookup.
        meta.lookup(|meta| {
            let q = meta.query_selector(q_u16);
            let x = meta.query_advice(limb_u16, Rotation::cur());
            vec![(q * x, tables.t16)]
        });

        // carry limb lookups.
        meta.lookup(|meta| {
            let q = meta.query_selector(q_carry);
            let x = meta.query_advice(carry_lo15, Rotation::cur());
            vec![(q * x, tables.t15)]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_carry);
            let x = meta.query_advice(carry_hi5, Rotation::cur());
            vec![(q * x, tables.t5)]
        });

        // carry recomposition: carry_val = lo15 + 2^15 * hi5.
        meta.create_gate("carry recomposition", |meta| {
            let q = meta.query_selector(q_carry);
            let lo = meta.query_advice(carry_lo15, Rotation::cur());
            let hi = meta.query_advice(carry_hi5, Rotation::cur());
            let v = meta.query_advice(carry_val, Rotation::cur());
            let two15 = fr_from_u64(pow2_u64(15));
            vec![q * (v - (lo + Expression::Constant(two15) * hi))]
        });

        // product gate: prod = a*b.
        meta.create_gate("limb multiplication", |meta| {
            let q = meta.query_selector(q_prod);
            let av = meta.query_advice(a, Rotation::cur());
            let bv = meta.query_advice(b, Rotation::cur());
            let pv = meta.query_advice(prod, Rotation::cur());
            vec![q * (pv - av * bv)]
        });

        // sum/carry gates on block starts.
        for (idx, sel) in q_sum.iter().enumerate() {
            let l = idx + 1;
            let sel = *sel;
            let gate_name: &'static str = Box::leak(format!("sum/carry length {l}").into_boxed_str());
            meta.create_gate(gate_name, move |meta| {
                let q = meta.query_selector(sel);
                let cin = meta.query_advice(carry_in, Rotation::cur());
                let out = meta.query_advice(out_limb, Rotation::cur());
                let cout = meta.query_advice(carry_out, Rotation::cur());

                let mut acc = Expression::Constant(fr_from_u64(0));
                for rot in 0..l {
                    let p = meta.query_advice(prod, Rotation(rot as i32));
                    acc = acc + p;
                }
                let two16 = fr_from_u64(pow2_u64(16));
                vec![q * (acc + cin - out - Expression::Constant(two16) * cout)]
            });
        }

        // Final limb row constraint: out_limb - carry_in = 0.
        meta.create_gate("final limb", |meta| {
            let q = meta.query_selector(q_last);
            let cin = meta.query_advice(carry_in, Rotation::cur());
            let out = meta.query_advice(out_limb, Rotation::cur());
            vec![q * (out - cin)]
        });

        BigMulConfig {
            limb_u16,
            q_u16,
            carry_lo15,
            carry_hi5,
            carry_val,
            q_carry,
            a,
            b,
            prod,
            q_prod,
            carry_in,
            out_limb,
            carry_out,
            q_sum,
            q_last,
            tables,
        }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fr>) -> Result<(), Error> {
        load_tables(&mut layouter, &config.tables)?;

        let (c, carry) = self.schoolbook_product();

        // 1) Assign range-checked u16 limbs (A,B,C) in one column.
        let mut a_cells = Vec::with_capacity(16);
        let mut b_cells = Vec::with_capacity(16);
        let mut c_cells = Vec::with_capacity(32);

        layouter.assign_region(|| "u16 limbs (range-checked)", |mut region| {
            let mut offset = 0usize;

            for &ai in self.a.iter() {
                config.q_u16.enable(&mut region, offset)?;
                let cell = region.assign_advice(|| "a", config.limb_u16, offset, || Value::known(fr_from_u64(ai as u64)))?;
                a_cells.push(cell);
                offset += 1;
            }

            for &bi in self.b.iter() {
                config.q_u16.enable(&mut region, offset)?;
                let cell = region.assign_advice(|| "b", config.limb_u16, offset, || Value::known(fr_from_u64(bi as u64)))?;
                b_cells.push(cell);
                offset += 1;
            }

            for &ci in c.iter() {
                config.q_u16.enable(&mut region, offset)?;
                let cell = region.assign_advice(|| "c", config.limb_u16, offset, || Value::known(fr_from_u64(ci as u64)))?;
                c_cells.push(cell);
                offset += 1;
            }

            Ok(())
        })?;

        // 2) Assign carry values with 15+5 decomposition.
        // carry[0] is fixed 0; we only witness carry[1..31].
        let mut carry_cells: Vec<Option<_>> = Vec::with_capacity(32);
        carry_cells.push(None);

        layouter.assign_region(|| "carry decomposition", |mut region| {
            let mut offset = 0usize;
            for k in 1..32usize {
                let v = carry[k] as u64;
                let lo15 = (v & ((1u64 << 15) - 1)) as u64;
                let hi5 = (v >> 15) as u64;

                config.q_carry.enable(&mut region, offset)?;
                region.assign_advice(|| "lo15", config.carry_lo15, offset, || Value::known(fr_from_u64(lo15)))?;
                region.assign_advice(|| "hi5", config.carry_hi5, offset, || Value::known(fr_from_u64(hi5)))?;
                let cell = region.assign_advice(|| "carry", config.carry_val, offset, || Value::known(fr_from_u64(v)))?;
                carry_cells.push(Some(cell));

                offset += 1;
            }
            Ok(())
        })?;

        // 3) Product rows grouped by k = 0..30.
        layouter.assign_region(|| "products + carry chain", |mut region| {
            let mut offset = 0usize;

            for k in 0..31usize {
                let mut pairs = Vec::new();
                for i in 0..16usize {
                    if k >= i {
                        let j = k - i;
                        if j < 16 {
                            pairs.push((i, j));
                        }
                    }
                }
                let len = pairs.len();
                assert!((1..=16).contains(&len));

                for (t, (i, j)) in pairs.into_iter().enumerate() {
                    config.q_prod.enable(&mut region, offset)?;

                    let a_copy = region.assign_advice(|| "a_copy", config.a, offset, || Value::known(fr_from_u64(self.a[i] as u64)))?;
                    let b_copy = region.assign_advice(|| "b_copy", config.b, offset, || Value::known(fr_from_u64(self.b[j] as u64)))?;
                    region.constrain_equal(a_copy.cell(), a_cells[i].cell())?;
                    region.constrain_equal(b_copy.cell(), b_cells[j].cell())?;

                    let prod_val = (self.a[i] as u64) * (self.b[j] as u64);
                    region.assign_advice(|| "prod", config.prod, offset, || Value::known(fr_from_u64(prod_val)))?;

                    if t == 0 {
                        // Block start: enable length-specific sum/carry gate.
                        config.q_sum[len - 1].enable(&mut region, offset)?;

                        // carry_in = carry[k]
                        let cin_val = if k == 0 { 0u64 } else { carry[k] as u64 };
                        let cin_cell = region.assign_advice(|| "cin", config.carry_in, offset, || Value::known(fr_from_u64(cin_val)))?;
                        if k != 0 {
                            let src = carry_cells[k].as_ref().unwrap();
                            region.constrain_equal(cin_cell.cell(), src.cell())?;
                        }

                        // out_limb = c[k]
                        let out_cell = region.assign_advice(|| "out", config.out_limb, offset, || Value::known(fr_from_u64(c[k] as u64)))?;
                        region.constrain_equal(out_cell.cell(), c_cells[k].cell())?;

                        // carry_out = carry[k+1]
                        let cout_val = carry[k + 1] as u64;
                        let cout_cell = region.assign_advice(|| "cout", config.carry_out, offset, || Value::known(fr_from_u64(cout_val)))?;
                        let src = carry_cells[k + 1].as_ref().unwrap();
                        region.constrain_equal(cout_cell.cell(), src.cell())?;
                    } else {
                        // Non-start rows: set auxiliaries to 0.
                        region.assign_advice(|| "cin=0", config.carry_in, offset, || Value::known(fr_from_u64(0)))?;
                        region.assign_advice(|| "out=0", config.out_limb, offset, || Value::known(fr_from_u64(0)))?;
                        region.assign_advice(|| "cout=0", config.carry_out, offset, || Value::known(fr_from_u64(0)))?;
                    }

                    offset += 1;
                }
            }

            // Final limb row: c[31] == carry[31].
            config.q_last.enable(&mut region, offset)?;

            // carry_in = carry[31]
            let cin_val = carry[31] as u64;
            let cin_cell = region.assign_advice(|| "cin_last", config.carry_in, offset, || Value::known(fr_from_u64(cin_val)))?;
            let src = carry_cells[31].as_ref().unwrap();
            region.constrain_equal(cin_cell.cell(), src.cell())?;

            // out_limb = c[31]
            let out_cell = region.assign_advice(|| "out_last", config.out_limb, offset, || Value::known(fr_from_u64(c[31] as u64)))?;
            region.constrain_equal(out_cell.cell(), c_cells[31].cell())?;

            // Unused in this row.
            region.assign_advice(|| "cout=0", config.carry_out, offset, || Value::known(fr_from_u64(0)))?;
            region.assign_advice(|| "a=0", config.a, offset, || Value::known(fr_from_u64(0)))?;
            region.assign_advice(|| "b=0", config.b, offset, || Value::known(fr_from_u64(0)))?;
            region.assign_advice(|| "prod=0", config.prod, offset, || Value::known(fr_from_u64(0)))?;

            Ok(())
        })?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct FragmentMetrics {
    rows: usize,
    lookup_cells: usize,
    mul_constraints: usize,
    linear_constraints: usize,
}

fn repair_metrics() -> FragmentMetrics {
    // Per promoted Horner row package in this harness:
    // 44 residue rows + 12 q66 rows + 12 boolean rows + 4 u8 rows.
    let residues = 44usize;
    let q66 = 12usize;
    let bools = 12usize;
    let u8s = 4usize;

    FragmentMetrics {
        rows: residues + q66 + bools + u8s,
        lookup_cells: 2 * residues + 5 * q66 + u8s,
        mul_constraints: residues + bools,
        linear_constraints: residues + q66,
    }
}

fn repair_paired_metrics() -> FragmentMetrics {
    let base = repair_metrics();
    FragmentMetrics {
        rows: base.rows + 4,
        lookup_cells: base.lookup_cells + 8,
        mul_constraints: base.mul_constraints,
        linear_constraints: base.linear_constraints + 4,
    }
}

fn b1_lite_metrics(num_limbs: usize) -> FragmentMetrics {
    // This matches the implemented B1-lite circuit (schoolbook + carry chain):
    // - u16 limb rows for A,B,C: 4n
    // - carry decomposition rows: (2n-1)
    // - product rows: n^2
    // - final carry/output check row: 1
    // lookup cells:
    // - limb lookups: 4n
    // - carry lookups (lo15 + hi5): 2*(2n-1)
    // constraints:
    // - multiplicative: n^2 product constraints
    // - linear: carry recomposition (2n-1) + sum/carry (2n-1) + final(1)
    let n = num_limbs;
    FragmentMetrics {
        rows: (4 * n) + (2 * n - 1) + (n * n) + 1,
        lookup_cells: (4 * n) + 2 * (2 * n - 1),
        mul_constraints: n * n,
        linear_constraints: (2 * n - 1) + (2 * n - 1) + 1,
    }
}

fn print_comparison(repair_fragment: FragmentMetrics, repair_b2: FragmentMetrics, b1: FragmentMetrics) {
    println!("---");
    println!("Quantitative comparison (implemented circuits)");
    println!("metric,repair_fragment,repair_b2_paired,b1_lite_baseline,ratio(b1/fragment),ratio(b1/b2)");
    println!(
        "rows,{},{},{},{:.2},{:.2}",
        repair_fragment.rows,
        repair_b2.rows,
        b1.rows,
        b1.rows as f64 / repair_fragment.rows as f64,
        b1.rows as f64 / repair_b2.rows as f64
    );
    println!(
        "lookup_cells,{},{},{},{:.2},{:.2}",
        repair_fragment.lookup_cells,
        repair_b2.lookup_cells,
        b1.lookup_cells,
        b1.lookup_cells as f64 / repair_fragment.lookup_cells as f64,
        b1.lookup_cells as f64 / repair_b2.lookup_cells as f64
    );
    println!(
        "mul_constraints,{},{},{},{:.2},{:.2}",
        repair_fragment.mul_constraints,
        repair_b2.mul_constraints,
        b1.mul_constraints,
        b1.mul_constraints as f64 / repair_fragment.mul_constraints as f64,
        b1.mul_constraints as f64 / repair_b2.mul_constraints as f64
    );
    println!(
        "linear_constraints,{},{},{},{:.2},{:.2}",
        repair_fragment.linear_constraints,
        repair_b2.linear_constraints,
        b1.linear_constraints,
        b1.linear_constraints as f64 / repair_fragment.linear_constraints as f64,
        b1.linear_constraints as f64 / repair_b2.linear_constraints as f64
    );
    println!("note: b1_lite excludes modular reduction and CRT-consistency checks; repair_b2 adds the 31-bit carry-normalization quotient add-on.");
}

fn main() {
    // k must exceed 16 because Halo2 reserves some rows (e.g., for blinding),
    // so a full 2^16-sized lookup table requires k=17.
    let k = 17;

    // Repair fragment (promoted-Horner block only, no carry-normalization add-on).
    let repair = RepairCircuit::new_fragment();
    let prover = MockProver::run(k, &repair, vec![]).expect("run repair circuit");
    assert_eq!(prover.verify(), Ok(()));
    println!("Repair fragment: verified (k={k}). Implemented: T_16/T_15/T_2/T_8 lookups + (x-p)nu=1 + linear recomposition + booleanity.");
    println!("  Counts (paper): 152 lookup cells, 56 multiplicative constraints, 56 linear recomposition constraints (per promoted Horner row)." );

    // Repair B2 paired package: fragment + carry-normalization q31 add-on (4 cells).
    let repair_b2 = RepairCircuit::new_paired_package();
    let prover = MockProver::run(k, &repair_b2, vec![]).expect("run repair_b2 circuit");
    assert_eq!(prover.verify(), Ok(()));
    println!("Repair B2 paired package: verified (k={k}). Adds 4 q31 carry-normalization cells (2 lookups + 1 linear each).");

    // Baseline: explicit schoolbook 256-bit mul (16-bit limbs) with carry chain.
    let bigmul = BigMulCircuit::new();
    let prover = MockProver::run(k, &bigmul, vec![]).expect("run bigmul circuit");
    assert_eq!(prover.verify(), Ok(()));
    println!("Bigfield baseline: verified (k={k}). Implemented: explicit cross-products (a_i*b_j) + carry chain + limb range checks (T_16) + carry decomposition (T_15/T_5)." );

    let repair_m = repair_metrics();
    let repair_b2_m = repair_paired_metrics();
    let b1_m = b1_lite_metrics(16);
    print_comparison(repair_m, repair_b2_m, b1_m);
}


