//! Compile-time executable function, helps you in generating lookup tables, so that you can perform NEON, AVX2 and SSSE3
//! optimized SIMD vector x single-scalar multiplication over GF(2^8), during RLNC erasure-coding. These table generation
//! logic is from <https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1100-L1105>.
//!
//! If you invoke `generate_gf256_simd_mul_low_table()`, it should generate `htd->low` part, described in above link.
//! Plain Rust code which should regenerate same table is as follows.
//!
//! ```rust,ignore
//! const GF256_ORDER: usize = 256;
//! const GF256_BIT_WIDTH: usize =  u8::BITS as usize;
//! const GF256_HALF_ORDER: usize = 1usize << (GF256_BIT_WIDTH / 2);
//!
//! let _ = (0..=((GF256_ORDER - 1) as u8))
//!        .map(|a| {
//!            (0..(GF256_HALF_ORDER as u8))
//!                .map(move |b| Gf256::mul_const(a, b))
//!                .collect::<Vec<u8>>()
//!        })
//!        .collect::<Vec<Vec<u8>>>();
//! ```
//!
//! If you invoke `generate_gf256_simd_mul_high_table()`, it should generate `htd->high` part, described in above link.
//! Plain Rust code which should regenerate same table is as follows.
//!
//! ```rust,ignore
//! let _ = (0..=((GF256_ORDER - 1) as u8))
//!        .map(|a| {
//!            (0..(GF256_HALF_ORDER as u8))
//!                .map(move |b| Gf256::mul_const(a, b << 4))
//!                .collect::<Vec<u8>>()
//!        })
//!        .collect::<Vec<Vec<u8>>>();
//! ```
use super::gf256::{GF256_HALF_ORDER, GF256_ORDER, Gf256};

const fn generate_gf256_simd_mul_low_table() -> [[u8; 2 * GF256_HALF_ORDER]; GF256_ORDER] {
    let mut table = [[0u8; 2 * GF256_HALF_ORDER]; GF256_ORDER];

    let mut row_idx = 0;
    while row_idx < GF256_ORDER {
        let mut col_idx = 0;

        while col_idx < GF256_HALF_ORDER {
            table[row_idx][col_idx] = Gf256::mul_const(row_idx as u8, col_idx as u8);
            col_idx += 1;
        }

        row_idx += 1;
    }

    table
}

const fn generate_gf256_simd_mul_high_table() -> [[u8; 2 * GF256_HALF_ORDER]; GF256_ORDER] {
    let mut table = [[0u8; 2 * GF256_HALF_ORDER]; GF256_ORDER];

    let mut row_idx = 0;
    while row_idx < GF256_ORDER {
        let mut col_idx = 0;

        while col_idx < GF256_HALF_ORDER {
            table[row_idx][col_idx] = Gf256::mul_const(row_idx as u8, (col_idx << 4) as u8);
            col_idx += 1;
        }

        row_idx += 1;
    }

    table
}

/// AVX2, SSSE3 and NEON optimized SIMD multiplication over GF(2^8) uses this lookup table, which is generated following
/// <https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1100-L1105>.
/// This table holds `htd->low` part, described in above link.
pub const GF256_SIMD_MUL_TABLE_LOW: [[u8; 2 * GF256_HALF_ORDER]; GF256_ORDER] = generate_gf256_simd_mul_low_table();

/// AVX2, SSSE3 and NEON optimized SIMD multiplication over GF(2^8) uses this lookup table, which is generated following
/// <https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1100-L1105>.
/// This table holds `htd->high` part, described in above link.
pub const GF256_SIMD_MUL_TABLE_HIGH: [[u8; 2 * GF256_HALF_ORDER]; GF256_ORDER] = generate_gf256_simd_mul_high_table();
