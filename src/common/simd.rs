use super::gf256::Gf256;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
use super::{
    gf256::GF256_HALF_ORDER,
    simd_mul_table::{GF256_SIMD_MUL_TABLE_HIGH, GF256_SIMD_MUL_TABLE_LOW},
};

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    _mm_and_si128, _mm_lddqu_si128, _mm_set1_epi8, _mm_shuffle_epi8, _mm_srli_epi64, _mm_storeu_si128, _mm_xor_si128, _mm256_and_si256, _mm256_lddqu_si256,
    _mm256_set1_epi8, _mm256_shuffle_epi8, _mm256_srli_epi64, _mm256_storeu_si256, _mm256_xor_si256,
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    _mm_and_si128, _mm_lddqu_si128, _mm_set1_epi8, _mm_shuffle_epi8, _mm_srli_epi64, _mm_storeu_si128, _mm_xor_si128, _mm256_and_si256, _mm256_lddqu_si256,
    _mm256_set1_epi8, _mm256_shuffle_epi8, _mm256_srli_epi64, _mm256_storeu_si256, _mm256_xor_si256,
};

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::{vandq_u8, vdupq_n_u8, veorq_u8, vld1q_u8, vqtbl1q_u8, vshrq_n_u8, vst1q_u8};
#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

/// Given a byte array of arbitrary length, this function can be used to multiply each
/// byte element with a single specific scalar, over GF(2^8), mutating the input vector.
///
/// In case this function runs on `x86_64` with `avx2` or `ssse3` features, it can use
/// lookup-table assisted SIMD multiplication, inspired from https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1029-L1037.
///
/// You have to build with `RUSTFLAGS="-C target-cpu=native"` flag to enjoy full benefits of compiler optimization.
///
/// I originally discovered this technique in https://www.snia.org/sites/default/files/files2/files2/SDC2013/presentations/NewThinking/EthanMiller_Screaming_Fast_Galois_Field%20Arithmetic_SIMD%20Instructions.pdf.
pub fn gf256_inplace_mul_vec_by_scalar(vec: &mut [u8], scalar: u8) {
    if vec.is_empty() {
        return;
    }
    if scalar == 0 {
        vec.fill(0);
        return;
    }
    if scalar == 1 {
        return;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        unsafe {
            let l_tbl = _mm256_lddqu_si256(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
            let h_tbl = _mm256_lddqu_si256(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
            let l_mask = _mm256_set1_epi8(0x0f);

            let mut iter = vec.chunks_exact_mut(2 * GF256_HALF_ORDER);

            for chunk in iter.by_ref() {
                let chunk_simd = _mm256_lddqu_si256(chunk.as_ptr() as *const _);

                let chunk_simd_lo = _mm256_and_si256(chunk_simd, l_mask);
                let chunk_simd_lo = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo);

                let chunk_simd_hi = _mm256_srli_epi64(chunk_simd, 4);
                let chunk_simd_hi = _mm256_and_si256(chunk_simd_hi, l_mask);
                let chunk_simd_hi = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi);

                let res = _mm256_xor_si256(chunk_simd_lo, chunk_simd_hi);
                _mm256_storeu_si256(chunk.as_mut_ptr() as *mut _, res);
            }

            iter.into_remainder().iter_mut().for_each(|symbol| {
                *symbol = Gf256::mul_const(*symbol, scalar);
            });
        }

        return;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("ssse3") {
        unsafe {
            let l_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
            let h_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
            let l_mask = _mm_set1_epi8(0x0f);

            let mut iter = vec.chunks_exact_mut(GF256_HALF_ORDER);

            for chunk in iter.by_ref() {
                let chunk_simd = _mm_lddqu_si128(chunk.as_ptr() as *const _);

                let chunk_simd_lo = _mm_and_si128(chunk_simd, l_mask);
                let chunk_simd_lo = _mm_shuffle_epi8(l_tbl, chunk_simd_lo);

                let chunk_simd_hi = _mm_srli_epi64(chunk_simd, 4);
                let chunk_simd_hi = _mm_and_si128(chunk_simd_hi, l_mask);
                let chunk_simd_hi = _mm_shuffle_epi8(h_tbl, chunk_simd_hi);

                let res = _mm_xor_si128(chunk_simd_lo, chunk_simd_hi);
                _mm_storeu_si128(chunk.as_mut_ptr() as *mut _, res);
            }

            iter.into_remainder().iter_mut().for_each(|symbol| {
                *symbol = Gf256::mul_const(*symbol, scalar);
            });
        }

        return;
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        unsafe {
            let l_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
            let h_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
            let l_mask = vdupq_n_u8(0x0f);

            let mut iter = vec.chunks_exact_mut(GF256_HALF_ORDER);

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

            iter.into_remainder().iter_mut().for_each(|symbol| {
                *symbol = Gf256::mul_const(*symbol, scalar);
            });
        }

        return;
    }

    vec.iter_mut().for_each(|src_symbol| {
        *src_symbol = Gf256::mul_const(*src_symbol, scalar);
    });
}

/// Given two byte arrays of equal length, this routine performs element-wise
/// addition over GF(2^8), mutating one of the operand vectors.
///
/// Note, addition over GF(2^8) is nothing but XOR-ing two operands. If this function
/// runs on `x86_64` with `avx2` or `ssse3` features, it can perform fast SIMD addition
/// using vector intrinsics.
///
/// You have to compile with `RUSTFLAGS="-C target-cpu=native` flag to hint the compiler
/// so that it generates best code.
pub fn gf256_inplace_add_vectors(vec_dst: &mut [u8], vec_src: &[u8]) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        unsafe {
            let mut iter_dst = vec_dst.chunks_exact_mut(2 * GF256_HALF_ORDER);
            let mut iter_src = vec_src.chunks_exact(2 * GF256_HALF_ORDER);

            for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
                let chunk_dst_simd = _mm256_lddqu_si256(chunk_dst.as_ptr() as *const _);
                let chunk_src_simd = _mm256_lddqu_si256(chunk_src.as_ptr() as *const _);
                let chunk_result = _mm256_xor_si256(chunk_dst_simd, chunk_src_simd);

                _mm256_storeu_si256(chunk_dst.as_mut_ptr() as *mut _, chunk_result);
            }

            let remainder_dst = iter_dst.into_remainder();
            let remainder_src = iter_src.remainder();

            remainder_dst.iter_mut().zip(remainder_src).for_each(|(a, b)| {
                *a ^= b;
            });
        }

        return;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("ssse3") {
        unsafe {
            let mut iter_dst = vec_dst.chunks_exact_mut(GF256_HALF_ORDER);
            let mut iter_src = vec_src.chunks_exact(GF256_HALF_ORDER);

            for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
                let chunk_dst_simd = _mm_lddqu_si128(chunk_dst.as_ptr() as *const _);
                let chunk_src_simd = _mm_lddqu_si128(chunk_src.as_ptr() as *const _);
                let chunk_result = _mm_xor_si128(chunk_dst_simd, chunk_src_simd);

                _mm_storeu_si128(chunk_dst.as_mut_ptr() as *mut _, chunk_result);
            }

            let remainder_dst = iter_dst.into_remainder();
            let remainder_src = iter_src.remainder();

            remainder_dst.iter_mut().zip(remainder_src).for_each(|(a, b)| {
                *a ^= b;
            });
        }

        return;
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        unsafe {
            let mut iter_dst = vec_dst.chunks_exact_mut(GF256_HALF_ORDER);
            let mut iter_src = vec_src.chunks_exact(GF256_HALF_ORDER);

            for (chunk_dst, chunk_src) in iter_dst.by_ref().zip(iter_src.by_ref()) {
                let chunk_dst_simd = vld1q_u8(chunk_dst.as_ptr() as *const _);
                let chunk_src_simd = vld1q_u8(chunk_src.as_ptr() as *const _);
                let chunk_result = veorq_u8(chunk_dst_simd, chunk_src_simd);

                vst1q_u8(chunk_dst.as_mut_ptr() as *mut _, chunk_result);
            }

            let remainder_dst = iter_dst.into_remainder();
            let remainder_src = iter_src.remainder();

            remainder_dst.iter_mut().zip(remainder_src).for_each(|(a, b)| {
                *a ^= b;
            });
        }

        return;
    }

    vec_dst.iter_mut().zip(vec_src).for_each(|(a, b)| {
        *a ^= b;
    });
}

/// Given a byte array `mul_vec` of arbitrary length, this function can be used to multiply each
/// byte element with a single specific scalar, over GF(2^8), and then adding each scaled value
/// to corresponding value in sink vector `add_into_vec`.
///
/// In case this function runs on `x86_64` with `avx2` or `ssse3` features, it can use
/// lookup-table assisted SIMD multiplication, inspired from https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1029-L1037.
///
/// You have to build with `RUSTFLAGS="-C target-cpu=native"` flag to enjoy full benefits of compiler optimization.
///
/// This function can be thought of an optimization over, first applying `gf256_inplace_mul_vec_by_scalar`
/// and then applying `gf256_inplace_add_vectors`.
pub fn gf256_mul_vec_by_scalar_then_add_into_vec(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) {
    if add_into_vec.is_empty() {
        return;
    }
    if scalar == 0 {
        return;
    }
    if scalar == 1 {
        gf256_inplace_add_vectors(add_into_vec, mul_vec);
        return;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        unsafe {
            let l_tbl = _mm256_lddqu_si256(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
            let h_tbl = _mm256_lddqu_si256(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
            let l_mask = _mm256_set1_epi8(0x0f);

            let mut add_vec_iter = add_into_vec.chunks_exact_mut(2 * GF256_HALF_ORDER);
            let mut mul_vec_iter = mul_vec.chunks_exact(2 * GF256_HALF_ORDER);

            for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
                let mul_vec_chunk_simd = _mm256_lddqu_si256(mul_vec_chunk.as_ptr() as *const _);

                let chunk_simd_lo = _mm256_and_si256(mul_vec_chunk_simd, l_mask);
                let chunk_simd_lo = _mm256_shuffle_epi8(l_tbl, chunk_simd_lo);

                let chunk_simd_hi = _mm256_srli_epi64(mul_vec_chunk_simd, 4);
                let chunk_simd_hi = _mm256_and_si256(chunk_simd_hi, l_mask);
                let chunk_simd_hi = _mm256_shuffle_epi8(h_tbl, chunk_simd_hi);

                let scaled_res = _mm256_xor_si256(chunk_simd_lo, chunk_simd_hi);

                let add_vec_chunk_simd = _mm256_lddqu_si256(add_vec_chunk.as_ptr() as *const _);
                let accum_res = _mm256_xor_si256(add_vec_chunk_simd, scaled_res);

                _mm256_storeu_si256(add_vec_chunk.as_mut_ptr() as *mut _, accum_res);
            }

            add_vec_iter
                .into_remainder()
                .iter_mut()
                .zip(mul_vec_iter.remainder().iter().map(|&src_symbol| Gf256::mul_const(src_symbol, scalar)))
                .for_each(|(res, scaled)| {
                    *res ^= scaled;
                });
        }

        return;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("ssse3") {
        unsafe {
            let l_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
            let h_tbl = _mm_lddqu_si128(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
            let l_mask = _mm_set1_epi8(0x0f);

            let mut add_vec_iter = add_into_vec.chunks_exact_mut(GF256_HALF_ORDER);
            let mut mul_vec_iter = mul_vec.chunks_exact(GF256_HALF_ORDER);

            for (add_vec_chunk, mul_vec_chunk) in add_vec_iter.by_ref().zip(mul_vec_iter.by_ref()) {
                let mul_vec_chunk_simd = _mm_lddqu_si128(mul_vec_chunk.as_ptr() as *const _);

                let chunk_simd_lo = _mm_and_si128(mul_vec_chunk_simd, l_mask);
                let chunk_simd_lo = _mm_shuffle_epi8(l_tbl, chunk_simd_lo);

                let chunk_simd_hi = _mm_srli_epi64(mul_vec_chunk_simd, 4);
                let chunk_simd_hi = _mm_and_si128(chunk_simd_hi, l_mask);
                let chunk_simd_hi = _mm_shuffle_epi8(h_tbl, chunk_simd_hi);

                let scaled_res = _mm_xor_si128(chunk_simd_lo, chunk_simd_hi);

                let add_vec_chunk_simd = _mm_lddqu_si128(add_vec_chunk.as_ptr() as *const _);
                let accum_res = _mm_xor_si128(add_vec_chunk_simd, scaled_res);

                _mm_storeu_si128(add_vec_chunk.as_mut_ptr() as *mut _, accum_res);
            }

            add_vec_iter
                .into_remainder()
                .iter_mut()
                .zip(mul_vec_iter.remainder().iter().map(|&src_symbol| Gf256::mul_const(src_symbol, scalar)))
                .for_each(|(res, scaled)| {
                    *res ^= scaled;
                });
        }

        return;
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        unsafe {
            let l_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_LOW[scalar as usize].as_ptr() as *const _);
            let h_tbl = vld1q_u8(GF256_SIMD_MUL_TABLE_HIGH[scalar as usize].as_ptr() as *const _);
            let l_mask = vdupq_n_u8(0x0f);

            let mut add_vec_iter = add_into_vec.chunks_exact_mut(GF256_HALF_ORDER);
            let mut mul_vec_iter = mul_vec.chunks_exact(GF256_HALF_ORDER);

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

            add_vec_iter
                .into_remainder()
                .iter_mut()
                .zip(mul_vec_iter.remainder().iter().map(|&src_symbol| Gf256::mul_const(src_symbol, scalar)))
                .for_each(|(res, scaled)| {
                    *res ^= scaled;
                });
        }

        return;
    }

    add_into_vec
        .iter_mut()
        .zip(mul_vec.iter().map(|&src_symbol| Gf256::mul_const(src_symbol, scalar)))
        .for_each(|(res, scaled)| *res ^= scaled);
}
