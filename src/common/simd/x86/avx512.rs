use crate::common::{
    gf256::{GF256_HALF_ORDER, Gf256},
    simd_mul_table::{GF256_SIMD_MUL_TABLE_HIGH, GF256_SIMD_MUL_TABLE_LOW},
};

#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "avx512bw")]
pub unsafe fn mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    let mut iter = vec.chunks_exact_mut(4 * 4 * GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm512_broadcast_i32x4(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast()));
        let h_tbl = _mm512_broadcast_i32x4(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast()));
        let l_mask = _mm512_set1_epi8(0x0f);

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

            let chunk0_simd_lo = _mm512_and_si512(chunk0_simd, l_mask);
            let chunk1_simd_lo = _mm512_and_si512(chunk1_simd, l_mask);
            let chunk2_simd_lo = _mm512_and_si512(chunk2_simd, l_mask);
            let chunk3_simd_lo = _mm512_and_si512(chunk3_simd, l_mask);

            let chunk0_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk0_simd_lo);
            let chunk1_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk1_simd_lo);
            let chunk2_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk2_simd_lo);
            let chunk3_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk3_simd_lo);

            let chunk0_simd_hi = _mm512_srli_epi64(chunk0_simd, 4);
            let chunk1_simd_hi = _mm512_srli_epi64(chunk1_simd, 4);
            let chunk2_simd_hi = _mm512_srli_epi64(chunk2_simd, 4);
            let chunk3_simd_hi = _mm512_srli_epi64(chunk3_simd, 4);

            let chunk0_simd_hi = _mm512_and_si512(chunk0_simd_hi, l_mask);
            let chunk1_simd_hi = _mm512_and_si512(chunk1_simd_hi, l_mask);
            let chunk2_simd_hi = _mm512_and_si512(chunk2_simd_hi, l_mask);
            let chunk3_simd_hi = _mm512_and_si512(chunk3_simd_hi, l_mask);

            let chunk0_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk0_simd_hi);
            let chunk1_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk1_simd_hi);
            let chunk2_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk2_simd_hi);
            let chunk3_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk3_simd_hi);

            let res0 = _mm512_xor_si512(chunk0_simd_lo, chunk0_simd_hi);
            let res1 = _mm512_xor_si512(chunk1_simd_lo, chunk1_simd_hi);
            let res2 = _mm512_xor_si512(chunk2_simd_lo, chunk2_simd_hi);
            let res3 = _mm512_xor_si512(chunk3_simd_lo, chunk3_simd_hi);

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

#[target_feature(enable = "avx512f")]
pub unsafe fn add_vec_into(vec_dst: &mut [u8], vec_src: &[u8]) {
    let mut iter_dst = vec_dst.chunks_exact_mut(4 * 4 * GF256_HALF_ORDER);
    let mut iter_src = vec_src.chunks_exact(4 * 4 * GF256_HALF_ORDER);

    unsafe {
        for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
            let (chunk0_dst, chunk1_dst, chunk2_dst, chunk3_dst) = {
                let (chunk0, rest) = chunk_dst.split_at_mut_unchecked(4 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_mut_unchecked(4 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_mut_unchecked(4 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let (chunk0_src, chunk1_src, chunk2_src, chunk3_src) = {
                let (chunk0, rest) = chunk_src.split_at_unchecked(4 * GF256_HALF_ORDER);
                let (chunk1, rest) = rest.split_at_unchecked(4 * GF256_HALF_ORDER);
                let (chunk2, chunk3) = rest.split_at_unchecked(4 * GF256_HALF_ORDER);

                (chunk0, chunk1, chunk2, chunk3)
            };

            let chunk0_dst_simd = _mm512_loadu_si512(chunk0_dst.as_ptr().cast());
            let chunk0_src_simd = _mm512_loadu_si512(chunk0_src.as_ptr().cast());
            let chunk0_result = _mm512_xor_si512(chunk0_dst_simd, chunk0_src_simd);
            _mm512_storeu_si512(chunk0_dst.as_mut_ptr().cast(), chunk0_result);

            let chunk1_dst_simd = _mm512_loadu_si512(chunk1_dst.as_ptr().cast());
            let chunk1_src_simd = _mm512_loadu_si512(chunk1_src.as_ptr().cast());
            let chunk1_result = _mm512_xor_si512(chunk1_dst_simd, chunk1_src_simd);
            _mm512_storeu_si512(chunk1_dst.as_mut_ptr().cast(), chunk1_result);

            let chunk2_dst_simd = _mm512_loadu_si512(chunk2_dst.as_ptr().cast());
            let chunk2_src_simd = _mm512_loadu_si512(chunk2_src.as_ptr().cast());
            let chunk2_result = _mm512_xor_si512(chunk2_dst_simd, chunk2_src_simd);
            _mm512_storeu_si512(chunk2_dst.as_mut_ptr().cast(), chunk2_result);

            let chunk3_dst_simd = _mm512_loadu_si512(chunk3_dst.as_ptr().cast());
            let chunk3_src_simd = _mm512_loadu_si512(chunk3_src.as_ptr().cast());
            let chunk3_result = _mm512_xor_si512(chunk3_dst_simd, chunk3_src_simd);
            _mm512_storeu_si512(chunk3_dst.as_mut_ptr().cast(), chunk3_result);
        }
    }

    iter_dst.into_remainder().iter_mut().zip(iter_src.remainder()).for_each(|(a, b)| {
        *a ^= b;
    });
}

#[target_feature(enable = "avx512bw")]
pub unsafe fn mul_vec_by_scalar_then_add_into(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    let mut add_vec_iter = add_into_vec.chunks_exact_mut(4 * 4 * GF256_HALF_ORDER);
    let mut mul_vec_iter = mul_vec.chunks_exact(4 * 4 * GF256_HALF_ORDER);

    unsafe {
        let l_tbl = _mm512_broadcast_i32x4(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr().cast()));
        let h_tbl = _mm512_broadcast_i32x4(_mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr().cast()));
        let l_mask = _mm512_set1_epi8(0x0f);

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

            let chunk0_simd_lo = _mm512_and_si512(mul_vec_chunk0_simd, l_mask);
            let chunk1_simd_lo = _mm512_and_si512(mul_vec_chunk1_simd, l_mask);
            let chunk2_simd_lo = _mm512_and_si512(mul_vec_chunk2_simd, l_mask);
            let chunk3_simd_lo = _mm512_and_si512(mul_vec_chunk3_simd, l_mask);

            let chunk0_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk0_simd_lo);
            let chunk1_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk1_simd_lo);
            let chunk2_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk2_simd_lo);
            let chunk3_simd_lo = _mm512_shuffle_epi8(l_tbl, chunk3_simd_lo);

            let chunk0_simd_hi = _mm512_srli_epi64(mul_vec_chunk0_simd, 4);
            let chunk1_simd_hi = _mm512_srli_epi64(mul_vec_chunk1_simd, 4);
            let chunk2_simd_hi = _mm512_srli_epi64(mul_vec_chunk2_simd, 4);
            let chunk3_simd_hi = _mm512_srli_epi64(mul_vec_chunk3_simd, 4);

            let chunk0_simd_hi = _mm512_and_si512(chunk0_simd_hi, l_mask);
            let chunk1_simd_hi = _mm512_and_si512(chunk1_simd_hi, l_mask);
            let chunk2_simd_hi = _mm512_and_si512(chunk2_simd_hi, l_mask);
            let chunk3_simd_hi = _mm512_and_si512(chunk3_simd_hi, l_mask);

            let chunk0_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk0_simd_hi);
            let chunk1_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk1_simd_hi);
            let chunk2_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk2_simd_hi);
            let chunk3_simd_hi = _mm512_shuffle_epi8(h_tbl, chunk3_simd_hi);

            let scaled_res0 = _mm512_xor_si512(chunk0_simd_lo, chunk0_simd_hi);
            let scaled_res1 = _mm512_xor_si512(chunk1_simd_lo, chunk1_simd_hi);
            let scaled_res2 = _mm512_xor_si512(chunk2_simd_lo, chunk2_simd_hi);
            let scaled_res3 = _mm512_xor_si512(chunk3_simd_lo, chunk3_simd_hi);

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
