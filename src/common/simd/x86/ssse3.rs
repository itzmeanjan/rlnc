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
    let mut iter = vec.chunks_exact_mut(4 * GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast());
        let h_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast());
        let l_mask = _mm_set1_epi8(0x0f);

        for chunk in iter.by_ref() {
            let (chunk0, chunk1, chunk2, chunk3) = {
                let (chunk0, rest) = chunk.split_at_mut_unchecked(GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_mut_unchecked(GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_mut_unchecked(GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let chunk0_simd = _mm_lddqu_si128(chunk0.as_ptr().cast());
            let chunk1_simd = _mm_lddqu_si128(chunk1.as_ptr().cast());
            let chunk2_simd = _mm_lddqu_si128(chunk2.as_ptr().cast());
            let chunk3_simd = _mm_lddqu_si128(chunk3.as_ptr().cast());

            let chunk0_simd_lo = _mm_and_si128(chunk0_simd, l_mask);
            let chunk1_simd_lo = _mm_and_si128(chunk1_simd, l_mask);
            let chunk2_simd_lo = _mm_and_si128(chunk2_simd, l_mask);
            let chunk3_simd_lo = _mm_and_si128(chunk3_simd, l_mask);

            let chunk0_simd_lo = _mm_shuffle_epi8(l_tbl, chunk0_simd_lo);
            let chunk1_simd_lo = _mm_shuffle_epi8(l_tbl, chunk1_simd_lo);
            let chunk2_simd_lo = _mm_shuffle_epi8(l_tbl, chunk2_simd_lo);
            let chunk3_simd_lo = _mm_shuffle_epi8(l_tbl, chunk3_simd_lo);

            let chunk0_simd_hi = _mm_srli_epi64(chunk0_simd, 4);
            let chunk1_simd_hi = _mm_srli_epi64(chunk1_simd, 4);
            let chunk2_simd_hi = _mm_srli_epi64(chunk2_simd, 4);
            let chunk3_simd_hi = _mm_srli_epi64(chunk3_simd, 4);

            let chunk0_simd_hi = _mm_and_si128(chunk0_simd_hi, l_mask);
            let chunk1_simd_hi = _mm_and_si128(chunk1_simd_hi, l_mask);
            let chunk2_simd_hi = _mm_and_si128(chunk2_simd_hi, l_mask);
            let chunk3_simd_hi = _mm_and_si128(chunk3_simd_hi, l_mask);

            let chunk0_simd_hi = _mm_shuffle_epi8(h_tbl, chunk0_simd_hi);
            let chunk1_simd_hi = _mm_shuffle_epi8(h_tbl, chunk1_simd_hi);
            let chunk2_simd_hi = _mm_shuffle_epi8(h_tbl, chunk2_simd_hi);
            let chunk3_simd_hi = _mm_shuffle_epi8(h_tbl, chunk3_simd_hi);

            let res0 = _mm_xor_si128(chunk0_simd_lo, chunk0_simd_hi);
            let res1 = _mm_xor_si128(chunk1_simd_lo, chunk1_simd_hi);
            let res2 = _mm_xor_si128(chunk2_simd_lo, chunk2_simd_hi);
            let res3 = _mm_xor_si128(chunk3_simd_lo, chunk3_simd_hi);

            _mm_storeu_si128(chunk0.as_mut_ptr().cast(), res0);
            _mm_storeu_si128(chunk1.as_mut_ptr().cast(), res1);
            _mm_storeu_si128(chunk2.as_mut_ptr().cast(), res2);
            _mm_storeu_si128(chunk3.as_mut_ptr().cast(), res3);
        }
    }

    iter.into_remainder().iter_mut().for_each(|symbol| {
        *symbol = Gf256::mul_const(*symbol, scalar);
    });
}

#[target_feature(enable = "ssse3")]
pub unsafe fn add_vec_into(vec_dst: &mut [u8], vec_src: &[u8]) {
    let mut iter_dst = vec_dst.chunks_exact_mut(4 * GF256_HALF_ORDER);
    let mut iter_src = vec_src.chunks_exact(4 * GF256_HALF_ORDER);

    unsafe {
        for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
            let (chunk0_dst, chunk1_dst, chunk2_dst, chunk3_dst) = {
                let (chunk0, rest) = chunk_dst.split_at_mut_unchecked(GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_mut_unchecked(GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_mut_unchecked(GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let (chunk0_src, chunk1_src, chunk2_src, chunk3_src) = {
                let (chunk0, rest) = chunk_src.split_at_unchecked(GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_unchecked(GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_unchecked(GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let chunk0_dst_simd = _mm_lddqu_si128(chunk0_dst.as_ptr().cast());
            let chunk0_src_simd = _mm_lddqu_si128(chunk0_src.as_ptr().cast());
            let chunk0_result = _mm_xor_si128(chunk0_dst_simd, chunk0_src_simd);
            _mm_storeu_si128(chunk0_dst.as_mut_ptr().cast(), chunk0_result);

            let chunk1_dst_simd = _mm_lddqu_si128(chunk1_dst.as_ptr().cast());
            let chunk1_src_simd = _mm_lddqu_si128(chunk1_src.as_ptr().cast());
            let chunk1_result = _mm_xor_si128(chunk1_dst_simd, chunk1_src_simd);
            _mm_storeu_si128(chunk1_dst.as_mut_ptr().cast(), chunk1_result);

            let chunk2_dst_simd = _mm_lddqu_si128(chunk2_dst.as_ptr().cast());
            let chunk2_src_simd = _mm_lddqu_si128(chunk2_src.as_ptr().cast());
            let chunk2_result = _mm_xor_si128(chunk2_dst_simd, chunk2_src_simd);
            _mm_storeu_si128(chunk2_dst.as_mut_ptr().cast(), chunk2_result);

            let chunk3_dst_simd = _mm_lddqu_si128(chunk3_dst.as_ptr().cast());
            let chunk3_src_simd = _mm_lddqu_si128(chunk3_src.as_ptr().cast());
            let chunk3_result = _mm_xor_si128(chunk3_dst_simd, chunk3_src_simd);
            _mm_storeu_si128(chunk3_dst.as_mut_ptr().cast(), chunk3_result);
        }
    }

    iter_dst.into_remainder().iter_mut().zip(iter_src.remainder()).for_each(|(a, b)| {
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
