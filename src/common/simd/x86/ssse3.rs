use crate::common::{
    gf256::{GF256_HALF_ORDER, Gf256},
    simd_mul_table::{GF256_SIMD_MUL_TABLE_HIGH, GF256_SIMD_MUL_TABLE_LOW},
};

#[cfg(target_arch = "x86")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "ssse3")]
pub unsafe fn mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    let mut iter = vec.chunks_exact_mut(GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast());
        let h_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast());
        let l_mask = _mm_set1_epi8(0x0f);

        for chunk in iter.by_ref() {
            let chunk_simd = _mm_lddqu_si128(chunk.as_ptr().cast());

            let chunk_simd_lo = _mm_and_si128(chunk_simd, l_mask);
            let chunk_simd_lo = _mm_shuffle_epi8(l_tbl, chunk_simd_lo);

            let chunk_simd_hi = _mm_srli_epi64(chunk_simd, 4);
            let chunk_simd_hi = _mm_and_si128(chunk_simd_hi, l_mask);
            let chunk_simd_hi = _mm_shuffle_epi8(h_tbl, chunk_simd_hi);

            let res = _mm_xor_si128(chunk_simd_lo, chunk_simd_hi);
            _mm_storeu_si128(chunk.as_mut_ptr().cast(), res);
        }
    }

    iter.into_remainder().iter_mut().for_each(|symbol| {
        *symbol = Gf256::mul_const(*symbol, scalar);
    });
}

#[target_feature(enable = "ssse3")]
pub unsafe fn add_vec_into(vec_dst: &mut [u8], vec_src: &[u8]) {
    let mut iter_dst = vec_dst.chunks_exact_mut(GF256_HALF_ORDER);
    let mut iter_src = vec_src.chunks_exact(GF256_HALF_ORDER);

    unsafe {
        for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
            let chunk_dst_simd = _mm_lddqu_si128(chunk_dst.as_ptr().cast());
            let chunk_src_simd = _mm_lddqu_si128(chunk_src.as_ptr().cast());
            let chunk_result = _mm_xor_si128(chunk_dst_simd, chunk_src_simd);

            _mm_storeu_si128(chunk_dst.as_mut_ptr().cast(), chunk_result);
        }
    }

    let remainder_dst = iter_dst.into_remainder();
    let remainder_src = iter_src.remainder();

    remainder_dst.iter_mut().zip(remainder_src).for_each(|(a, b)| {
        *a ^= b;
    });
}

#[target_feature(enable = "ssse3")]
pub unsafe fn mul_vec_by_scalar_then_add_into(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    let mut add_vec_iter = add_into_vec.chunks_exact_mut(GF256_HALF_ORDER);
    let mut mul_vec_iter = mul_vec.chunks_exact(GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast());
        let h_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast());
        let l_mask = _mm_set1_epi8(0x0f);

        for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
            let mul_vec_chunk_simd = _mm_lddqu_si128(mul_vec_chunk.as_ptr().cast());

            let chunk_simd_lo = _mm_and_si128(mul_vec_chunk_simd, l_mask);
            let chunk_simd_lo = _mm_shuffle_epi8(l_tbl, chunk_simd_lo);

            let chunk_simd_hi = _mm_srli_epi64(mul_vec_chunk_simd, 4);
            let chunk_simd_hi = _mm_and_si128(chunk_simd_hi, l_mask);
            let chunk_simd_hi = _mm_shuffle_epi8(h_tbl, chunk_simd_hi);

            let scaled_res = _mm_xor_si128(chunk_simd_lo, chunk_simd_hi);

            let add_vec_chunk_simd = _mm_lddqu_si128(add_vec_chunk.as_ptr().cast());
            let accum_res = _mm_xor_si128(add_vec_chunk_simd, scaled_res);

            _mm_storeu_si128(add_vec_chunk.as_mut_ptr().cast(), accum_res);
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
