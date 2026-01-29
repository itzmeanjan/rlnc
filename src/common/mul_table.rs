use super::gf256::{GF256_ORDER, Gf256};

const fn gen_mul_table() -> [[u8; GF256_ORDER]; GF256_ORDER] {
    let mut table = [[0u8; GF256_ORDER]; GF256_ORDER];

    let mut row_idx = 0;
    while row_idx < GF256_ORDER {
        let mut col_idx = 0;

        while col_idx < GF256_ORDER {
            table[row_idx][col_idx] = Gf256::mul_const(row_idx as u8, col_idx as u8);
            col_idx += 1;
        }
        row_idx += 1;
    }

    table
}

pub const GF256_TABLES: [[u8; GF256_ORDER]; GF256_ORDER] = gen_mul_table();
