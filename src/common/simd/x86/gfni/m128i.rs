use crate::common::gf256::{GF256_HALF_ORDER, Gf256};

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "gfni", enable = "avx512vl")]
pub unsafe fn mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    let mut iter = vec.chunks_exact_mut(2 * GF256_HALF_ORDER);

    unsafe {
        let scalar_simd = _mm_set1_epi8(scalar as i8);
        for chunk in iter.by_ref() {
            let chunk_simd = _mm_loadu_si128(chunk.as_ptr().cast());
            let res = _mm_gf2p8mul_epi8(chunk_simd, scalar_simd);

            _mm_storeu_si128(chunk.as_mut_ptr().cast(), res);
        }
    }

    iter.into_remainder().iter_mut().for_each(|symbol| {
        *symbol = Gf256::mul_const(*symbol, scalar);
    });
}

#[target_feature(enable = "gfni", enable = "avx512vl")]
pub unsafe fn mul_vec_by_scalar_then_add_into(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    let mut add_vec_iter = add_into_vec.chunks_exact_mut(2 * GF256_HALF_ORDER);
    let mut mul_vec_iter = mul_vec.chunks_exact(2 * GF256_HALF_ORDER);

    unsafe {
        let scalar_simd = _mm_set1_epi8(scalar as i8);

        for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
            let mul_vec_chunk_simd = _mm_loadu_si128(mul_vec_chunk.as_ptr().cast());
            let scaled_res = _mm_gf2p8mul_epi8(mul_vec_chunk_simd, scalar_simd);

            let add_vec_chunk_simd = _mm_loadu_si128(add_vec_chunk.as_ptr().cast());
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
