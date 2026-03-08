#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedFamily {
    Inv,
    Ef,
    Ee,
    Hor,
    Car,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputProfile {
    Standard,
    Boundary,
    Adversarial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedRowWitness {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub q: u128,
    pub is_q31: bool,
}

pub const P_U32: u32 = 2_147_483_647;
pub const Q31_MAX_U32: u32 = (1u32 << 31) - 1;
pub const Q66_MAX_U128: u128 = (1u128 << 66) - 1;

fn small_offset(rep: usize, local_idx: usize, modulus: u32) -> u32 {
    ((rep as u32) * 17 + local_idx as u32) % modulus
}

pub fn shared_row_witness(
    rep: usize,
    family: SharedFamily,
    local_idx: usize,
    profile: InputProfile,
) -> SharedRowWitness {
    match profile {
        InputProfile::Standard => standard_row_witness(rep, family, local_idx),
        InputProfile::Boundary => boundary_row_witness(rep, family, local_idx),
        InputProfile::Adversarial => adversarial_row_witness(rep, family, local_idx),
    }
}

fn standard_row_witness(rep: usize, family: SharedFamily, local_idx: usize) -> SharedRowWitness {
    let base = small_offset(rep, local_idx, 1024);
    match family {
        SharedFamily::Inv => {
            let x = 1_000 + base;
            SharedRowWitness {
                x,
                y: 1,
                z: x,
                q: 1,
                is_q31: false,
            }
        }
        SharedFamily::Ef => {
            let x = 50_000 + base;
            let y = 1;
            SharedRowWitness {
                x,
                y,
                z: x * y,
                q: (1u128 << 30) + 1 + local_idx as u128,
                is_q31: true,
            }
        }
        SharedFamily::Ee => {
            let x = 75_000 + base;
            SharedRowWitness {
                x,
                y: 0,
                z: 0,
                q: (1u128 << 62) + 11 + local_idx as u128,
                is_q31: false,
            }
        }
        SharedFamily::Hor => {
            let x = 125_000 + base;
            SharedRowWitness {
                x,
                y: 0,
                z: 0,
                q: (1u128 << 61) + 101 + local_idx as u128,
                is_q31: false,
            }
        }
        SharedFamily::Car => {
            let x = 90_000 + base;
            let y = 1_000 + (base % 4_000);
            SharedRowWitness {
                x,
                y,
                z: x + y + 5,
                q: (1u128 << 30) + 1_000 + local_idx as u128,
                is_q31: true,
            }
        }
    }
}

fn boundary_row_witness(rep: usize, family: SharedFamily, local_idx: usize) -> SharedRowWitness {
    let offset = small_offset(rep, local_idx, 64);
    match family {
        SharedFamily::Inv => SharedRowWitness {
            x: P_U32 - 1 - offset,
            y: 1,
            z: P_U32 - 1 - offset,
            q: 1,
            is_q31: false,
        },
        SharedFamily::Ef => {
            let x = P_U32 - 1 - offset;
            SharedRowWitness {
                x,
                y: 1,
                z: x,
                q: (Q31_MAX_U32 - offset) as u128,
                is_q31: true,
            }
        }
        SharedFamily::Ee => SharedRowWitness {
            x: P_U32 - 1 - offset,
            y: 0,
            z: 0,
            q: Q66_MAX_U128 - offset as u128,
            is_q31: false,
        },
        SharedFamily::Hor => SharedRowWitness {
            x: P_U32 - 128 - offset,
            y: 0,
            z: 0,
            q: Q66_MAX_U128 - 128 - offset as u128,
            is_q31: false,
        },
        SharedFamily::Car => {
            let y = offset;
            let x = P_U32 - 6 - y;
            SharedRowWitness {
                x,
                y,
                z: P_U32 - 1,
                q: (Q31_MAX_U32 - offset) as u128,
                is_q31: true,
            }
        }
    }
}

fn adversarial_row_witness(
    rep: usize,
    family: SharedFamily,
    local_idx: usize,
) -> SharedRowWitness {
    let offset = small_offset(rep, local_idx, 4);
    match family {
        SharedFamily::Inv => SharedRowWitness {
            x: P_U32 - 1 - offset,
            y: 1,
            z: P_U32 - 1 - offset,
            q: 1,
            is_q31: false,
        },
        SharedFamily::Ef => SharedRowWitness {
            x: P_U32 - 1 - offset,
            y: 1,
            z: P_U32 - 1 - offset,
            q: (Q31_MAX_U32 - offset) as u128,
            is_q31: true,
        },
        SharedFamily::Ee => SharedRowWitness {
            x: P_U32 - 1 - offset,
            y: 0,
            z: 0,
            q: Q66_MAX_U128 - offset as u128,
            is_q31: false,
        },
        SharedFamily::Hor => SharedRowWitness {
            x: P_U32 - 1 - offset,
            y: 0,
            z: 0,
            q: Q66_MAX_U128 - 8 - offset as u128,
            is_q31: false,
        },
        SharedFamily::Car => SharedRowWitness {
            x: P_U32 - 6,
            y: 0,
            z: P_U32 - 1,
            q: (Q31_MAX_U32 - offset) as u128,
            is_q31: true,
        },
    }
}
