use crate::common::gf256::{GF256_HALF_ORDER, Gf256};

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "gfni", enable = "avx512f")]
pub unsafe fn mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    let mut iter = vec.chunks_exact_mut(4 * 4 * GF256_HALF_ORDER);

    unsafe {
        let scalar_simd = _mm512_set1_epi8(scalar as i8);
        for chunk in iter.by_ref() {
            let (chunk0, chunk1, chunk2, chunk3) = {
                let (chunk0, rest) = chunk.split_at_mut_unchecked(4 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_mut_unchecked(4 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_mut_unchecked(4 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let chunk0_simd = _mm512_loadu_si512(chunk0.as_ptr().cast());
            let chunk1_simd = _mm512_loadu_si512(chunk1.as_ptr().cast());
            let chunk2_simd = _mm512_loadu_si512(chunk2.as_ptr().cast());
            let chunk3_simd = _mm512_loadu_si512(chunk3.as_ptr().cast());

            let res0 = _mm512_gf2p8mul_epi8(chunk0_simd, scalar_simd);
            let res1 = _mm512_gf2p8mul_epi8(chunk1_simd, scalar_simd);
            let res2 = _mm512_gf2p8mul_epi8(chunk2_simd, scalar_simd);
            let res3 = _mm512_gf2p8mul_epi8(chunk3_simd, scalar_simd);

            _mm512_storeu_si512(chunk0.as_mut_ptr().cast(), res0);
            _mm512_storeu_si512(chunk1.as_mut_ptr().cast(), res1);
            _mm512_storeu_si512(chunk2.as_mut_ptr().cast(), res2);
            _mm512_storeu_si512(chunk3.as_mut_ptr().cast(), res3);
        }
    }

    iter.into_remainder().iter_mut().for_each(|symbol| {
        *symbol = Gf256::mul_const(*symbol, scalar);
    });
}

#[target_feature(enable = "gfni", enable = "avx512f")]
pub unsafe fn mul_vec_by_scalar_then_add_into(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    let mut add_vec_iter = add_into_vec.chunks_exact_mut(4 * 4 * GF256_HALF_ORDER);
    let mut mul_vec_iter = mul_vec.chunks_exact(4 * 4 * GF256_HALF_ORDER);

    unsafe {
        let scalar_simd = _mm512_set1_epi8(scalar as i8);

        for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
            let (mul_vec_chunk0, mul_vec_chunk1, mul_vec_chunk2, mul_vec_chunk3) = {
                let (chunk0, rest) = mul_vec_chunk.split_at_unchecked(4 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_unchecked(4 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_unchecked(4 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let mul_vec_chunk0_simd = _mm512_loadu_si512(mul_vec_chunk0.as_ptr().cast());
            let mul_vec_chunk1_simd = _mm512_loadu_si512(mul_vec_chunk1.as_ptr().cast());
            let mul_vec_chunk2_simd = _mm512_loadu_si512(mul_vec_chunk2.as_ptr().cast());
            let mul_vec_chunk3_simd = _mm512_loadu_si512(mul_vec_chunk3.as_ptr().cast());

            let scaled_res0 = _mm512_gf2p8mul_epi8(mul_vec_chunk0_simd, scalar_simd);
            let scaled_res1 = _mm512_gf2p8mul_epi8(mul_vec_chunk1_simd, scalar_simd);
            let scaled_res2 = _mm512_gf2p8mul_epi8(mul_vec_chunk2_simd, scalar_simd);
            let scaled_res3 = _mm512_gf2p8mul_epi8(mul_vec_chunk3_simd, scalar_simd);

            let (add_vec_chunk0, add_vec_chunk1, add_vec_chunk2, add_vec_chunk3) = {
                let (chunk0, rest) = add_vec_chunk.split_at_mut_unchecked(4 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_mut_unchecked(4 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_mut_unchecked(4 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let add_vec_chunk0_simd = _mm512_loadu_si512(add_vec_chunk0.as_ptr().cast());
            let add_vec_chunk1_simd = _mm512_loadu_si512(add_vec_chunk1.as_ptr().cast());
            let add_vec_chunk2_simd = _mm512_loadu_si512(add_vec_chunk2.as_ptr().cast());
            let add_vec_chunk3_simd = _mm512_loadu_si512(add_vec_chunk3.as_ptr().cast());

            let accum_res0 = _mm512_xor_si512(add_vec_chunk0_simd, scaled_res0);
            let accum_res1 = _mm512_xor_si512(add_vec_chunk1_simd, scaled_res1);
            let accum_res2 = _mm512_xor_si512(add_vec_chunk2_simd, scaled_res2);
            let accum_res3 = _mm512_xor_si512(add_vec_chunk3_simd, scaled_res3);

            _mm512_storeu_si512(add_vec_chunk0.as_mut_ptr().cast(), accum_res0);
            _mm512_storeu_si512(add_vec_chunk1.as_mut_ptr().cast(), accum_res1);
            _mm512_storeu_si512(add_vec_chunk2.as_mut_ptr().cast(), accum_res2);
            _mm512_storeu_si512(add_vec_chunk3.as_mut_ptr().cast(), accum_res3);
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
