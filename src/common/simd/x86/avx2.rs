use crate::common::{
    gf256::{GF256_HALF_ORDER, Gf256},
    simd_mul_table::{GF256_SIMD_MUL_TABLE_HIGH, GF256_SIMD_MUL_TABLE_LOW},
};

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
pub unsafe fn mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    let mut iter = vec.chunks_exact_mut(2 * GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm256_broadcastsi128_si256(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast()));
        let h_tbl = _mm256_broadcastsi128_si256(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast()));
        let l_mask = _mm256_set1_epi8(0x0f);

        for chunk in iter.by_ref() {
            let chunk_simd = _mm256_lddqu_si256(chunk.as_ptr().cast());

            let chunk_simd_lo = _mm256_and_si256(chunk_simd, l_mask);
            let chunk_simd_lo = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo);

            let chunk_simd_hi = _mm256_srli_epi64(chunk_simd, 4);
            let chunk_simd_hi = _mm256_and_si256(chunk_simd_hi, l_mask);
            let chunk_simd_hi = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi);

            let res = _mm256_xor_si256(chunk_simd_lo, chunk_simd_hi);
            _mm256_storeu_si256(chunk.as_mut_ptr().cast(), res);
        }
    }

    iter.into_remainder().iter_mut().for_each(|symbol| {
        *symbol = Gf256::mul_const(*symbol, scalar);
    });
}

#[target_feature(enable = "avx2")]
pub unsafe fn add_vec_into(vec_dst: &mut [u8], vec_src: &[u8]) {
    let mut iter_dst = vec_dst.chunks_exact_mut(2 * GF256_HALF_ORDER);
    let mut iter_src = vec_src.chunks_exact(2 * GF256_HALF_ORDER);

    unsafe {
        for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
            let chunk_dst_simd = _mm256_lddqu_si256(chunk_dst.as_ptr().cast());
            let chunk_src_simd = _mm256_lddqu_si256(chunk_src.as_ptr().cast());
            let chunk_result = _mm256_xor_si256(chunk_dst_simd, chunk_src_simd);

            _mm256_storeu_si256(chunk_dst.as_mut_ptr().cast(), chunk_result);
        }
    }

    let remainder_dst = iter_dst.into_remainder();
    let remainder_src = iter_src.remainder();

    remainder_dst.iter_mut().zip(remainder_src).for_each(|(a, b)| {
        *a ^= b;
    });
}

#[target_feature(enable = "avx2")]
pub unsafe fn mul_vec_by_scalar_then_add_into(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    let mut add_vec_iter = add_into_vec.chunks_exact_mut(4 * 2 * GF256_HALF_ORDER);
    let mut mul_vec_iter = mul_vec.chunks_exact(4 * 2 * GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm256_broadcastsi128_si256(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast()));
        let h_tbl = _mm256_broadcastsi128_si256(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast()));
        let l_mask = _mm256_set1_epi8(0x0f);

        for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
            let (mul_vec_chunk0, mul_vec_chunk1, mul_vec_chunk2, mul_vec_chunk3) = {
                let (chunk0, rest) = mul_vec_chunk.split_at_unchecked(2 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_unchecked(2 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_unchecked(2 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let mul_vec_chunk0_simd = _mm256_lddqu_si256(mul_vec_chunk0.as_ptr().cast());
            let mul_vec_chunk1_simd = _mm256_lddqu_si256(mul_vec_chunk1.as_ptr().cast());
            let mul_vec_chunk2_simd = _mm256_lddqu_si256(mul_vec_chunk2.as_ptr().cast());
            let mul_vec_chunk3_simd = _mm256_lddqu_si256(mul_vec_chunk3.as_ptr().cast());

            let chunk_simd_lo0 = _mm256_and_si256(mul_vec_chunk0_simd, l_mask);
            let chunk_simd_lo1 = _mm256_and_si256(mul_vec_chunk1_simd, l_mask);
            let chunk_simd_lo2 = _mm256_and_si256(mul_vec_chunk2_simd, l_mask);
            let chunk_simd_lo3 = _mm256_and_si256(mul_vec_chunk3_simd, l_mask);

            let chunk_simd_hi0 = _mm256_and_si256(_mm256_srli_epi64(mul_vec_chunk0_simd, 4), l_mask);
            let chunk_simd_hi1 = _mm256_and_si256(_mm256_srli_epi64(mul_vec_chunk1_simd, 4), l_mask);
            let chunk_simd_hi2 = _mm256_and_si256(_mm256_srli_epi64(mul_vec_chunk2_simd, 4), l_mask);
            let chunk_simd_hi3 = _mm256_and_si256(_mm256_srli_epi64(mul_vec_chunk3_simd, 4), l_mask);

            let chunk_simd_lo0 = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo0);
            let chunk_simd_lo1 = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo1);
            let chunk_simd_lo2 = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo2);
            let chunk_simd_lo3 = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo3);

            let chunk_simd_hi0 = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi0);
            let chunk_simd_hi1 = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi1);
            let chunk_simd_hi2 = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi2);
            let chunk_simd_hi3 = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi3);

            let scaled_res0 = _mm256_xor_si256(chunk_simd_lo0, chunk_simd_hi0);
            let scaled_res1 = _mm256_xor_si256(chunk_simd_lo1, chunk_simd_hi1);
            let scaled_res2 = _mm256_xor_si256(chunk_simd_lo2, chunk_simd_hi2);
            let scaled_res3 = _mm256_xor_si256(chunk_simd_lo3, chunk_simd_hi3);

            let (add_vec_chunk0, add_vec_chunk1, add_vec_chunk2, add_vec_chunk3) = {
                let (chunk0, rest) = add_vec_chunk.split_at_mut_unchecked(2 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_mut_unchecked(2 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_mut_unchecked(2 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let add_vec_chunk0_simd = _mm256_lddqu_si256(add_vec_chunk0.as_ptr().cast());
            let add_vec_chunk1_simd = _mm256_lddqu_si256(add_vec_chunk1.as_ptr().cast());
            let add_vec_chunk2_simd = _mm256_lddqu_si256(add_vec_chunk2.as_ptr().cast());
            let add_vec_chunk3_simd = _mm256_lddqu_si256(add_vec_chunk3.as_ptr().cast());

            let accum_res0 = _mm256_xor_si256(add_vec_chunk0_simd, scaled_res0);
            let accum_res1 = _mm256_xor_si256(add_vec_chunk1_simd, scaled_res1);
            let accum_res2 = _mm256_xor_si256(add_vec_chunk2_simd, scaled_res2);
            let accum_res3 = _mm256_xor_si256(add_vec_chunk3_simd, scaled_res3);

            _mm256_storeu_si256(add_vec_chunk0.as_mut_ptr().cast(), accum_res0);
            _mm256_storeu_si256(add_vec_chunk1.as_mut_ptr().cast(), accum_res1);
            _mm256_storeu_si256(add_vec_chunk2.as_mut_ptr().cast(), accum_res2);
            _mm256_storeu_si256(add_vec_chunk3.as_mut_ptr().cast(), accum_res3);
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
