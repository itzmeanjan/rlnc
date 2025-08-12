use crate::common::gf256::Gf256;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(target_arch = "aarch64")]
mod aarch64;

/// Given a byte array of arbitrary length, this function can be used to multiply each
/// byte element with a single specific scalar, over GF(2^8), mutating the input vector.
///
/// In case this function runs on `x86_64` CPU with `avx2` or `ssse3` features or on `aarch64` CPU with `neon` features,
/// it can use lookup-table assisted SIMD multiplication, inspired from https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1029-L1037.
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
    {
        if x86::gf256_inplace_mul_vec_by_scalar(vec, scalar) {
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if aarch64::gf256_inplace_mul_vec_by_scalar(vec, scalar) {
            return;
        }
    }

    vec.iter_mut().for_each(|src_symbol| {
        *src_symbol = Gf256::mul_const(*src_symbol, scalar);
    });
}

/// Given two byte arrays of equal length, this routine performs element-wise
/// addition over GF(2^8), mutating one of the operand vectors.
///
/// Note, addition over GF(2^8) is nothing but XOR-ing two operands. If this function
/// runs on `x86_64` CPU with `avx2` or `ssse3` features or on `aarch64` CPU with `neon` features,
/// it can perform fast SIMD addition using vector intrinsics.
///
/// You have to compile with `RUSTFLAGS="-C target-cpu=native` flag to hint the compiler
/// so that it generates best code.
pub fn gf256_inplace_add_vectors(vec_dst: &mut [u8], vec_src: &[u8]) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if x86::gf256_inplace_add_vectors(vec_dst, vec_src) {
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if aarch64::gf256_inplace_add_vectors(vec_dst, vec_src) {
            return;
        }
    }

    vec_dst.iter_mut().zip(vec_src).for_each(|(a, b)| {
        *a ^= b;
    });
}

/// Given a byte array `mul_vec` of arbitrary length, this function can be used to multiply each
/// byte element with a single specific scalar, over GF(2^8), and then adding each scaled value
/// to corresponding value in sink vector `add_into_vec`.
///
/// In case this function runs on `x86_64` CPU with `avx2` or `ssse3` features or on `aarch64` CPU with `neon` features,
/// it can use lookup-table assisted SIMD multiplication, inspired from https://github.com/ceph/gf-complete/blob/a6862d10c9db467148f20eef2c6445ac9afd94d8/src/gf_w8.c#L1029-L1037.
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
    {
        if x86::gf256_mul_vec_by_scalar_then_add_into_vec(add_into_vec, mul_vec, scalar) {
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if aarch64::gf256_mul_vec_by_scalar_then_add_into_vec(add_into_vec, mul_vec, scalar) {
            return;
        }
    }

    add_into_vec
        .iter_mut()
        .zip(mul_vec.iter().map(|&src_symbol| Gf256::mul_const(src_symbol, scalar)))
        .for_each(|(res, scaled)| *res ^= scaled);
}
