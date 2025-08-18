use crate::common::{
    gf256::{GF256_HALF_ORDER, Gf256},
    simd_mul_table::{GF256_SIMD_MUL_TABLE_HIGH, GF256_SIMD_MUL_TABLE_LOW},
};

use std::arch::aarch64::{vandq_u8, vdupq_n_u8, veorq_u8, vld1q_u8, vqtbl1q_u8, vshrq_n_u8, vst1q_u8};

#[target_feature(enable = "neon")]
pub unsafe fn mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    let mut iter = vec.chunks_exact_mut(GF256_HALF_ORDER);

    unsafe {
        let l_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
        let h_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
        let l_mask = vdupq_n_u8(0x0f);

        for chunk in iter.by_ref() {
            let chunk_simd = vld1q_u8(chunk.as_ptr() as *const _);

            let chunk_simd_lo = vandq_u8(chunk_simd, l_mask);
            let chunk_simd_lo = vqtbl1q_u8(l_tbl, chunk_simd_lo);

            let chunk_simd_hi = vshrq_n_u8(chunk_simd, 4);
            let chunk_simd_hi = vandq_u8(chunk_simd_hi, l_mask);
            let chunk_simd_hi = vqtbl1q_u8(h_tbl, chunk_simd_hi);

            let res = veorq_u8(chunk_simd_lo, chunk_simd_hi);
            vst1q_u8(chunk.as_mut_ptr() as *mut _, res);
        }
    }

    iter.into_remainder().iter_mut().for_each(|symbol| {
        *symbol = Gf256::mul_const(*symbol, scalar);
    });
}

#[target_feature(enable = "neon")]
pub unsafe fn add_vec_into(vec_dst: &mut [u8], vec_src: &[u8]) {
    let mut iter_dst = vec_dst.chunks_exact_mut(GF256_HALF_ORDER);
    let mut iter_src = vec_src.chunks_exact(GF256_HALF_ORDER);

    unsafe {
        for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
            let chunk_dst_simd = vld1q_u8(chunk_dst.as_ptr() as *const _);
            let chunk_src_simd = vld1q_u8(chunk_src.as_ptr() as *const _);
            let chunk_result = veorq_u8(chunk_dst_simd, chunk_src_simd);

            vst1q_u8(chunk_dst.as_mut_ptr() as *mut _, chunk_result);
        }
    }

    let remainder_dst = iter_dst.into_remainder();
    let remainder_src = iter_src.remainder();

    remainder_dst.iter_mut().zip(remainder_src).for_each(|(a, b)| {
        *a ^= b;
    });
}

#[target_feature(enable = "neon")]
pub unsafe fn mul_vec_by_scalar_then_add_into(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    let mut add_vec_iter = add_into_vec.chunks_exact_mut(GF256_HALF_ORDER);
    let mut mul_vec_iter = mul_vec.chunks_exact(GF256_HALF_ORDER);

    unsafe {
        let l_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
        let h_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
        let l_mask = vdupq_n_u8(0x0f);

        for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
            let mul_vec_chunk_simd = vld1q_u8(mul_vec_chunk.as_ptr() as *const _);

            let chunk_simd_lo = vandq_u8(mul_vec_chunk_simd, l_mask);
            let chunk_simd_lo = vqtbl1q_u8(l_tbl, chunk_simd_lo);

            let chunk_simd_hi = vshrq_n_u8(mul_vec_chunk_simd, 4);
            let chunk_simd_hi = vandq_u8(chunk_simd_hi, l_mask);
            let chunk_simd_hi = vqtbl1q_u8(h_tbl, chunk_simd_hi);

            let scaled_res = veorq_u8(chunk_simd_lo, chunk_simd_hi);

            let add_vec_chunk_simd = vld1q_u8(add_vec_chunk.as_ptr() as *const _);
            let accum_res = veorq_u8(add_vec_chunk_simd, scaled_res);

            vst1q_u8(add_vec_chunk.as_mut_ptr() as *mut _, accum_res);
        }
    }

    add_vec_iter
        .into_remainder()
        .iter_mut()
        .zip(mul_vec_iter.remainder().iter().map(|&src_symbol| Gf256::mul_const(src_symbol, scalar)))
        .for_each(|(res, scaled)| {
            *res ^= scaled;
        });
}
