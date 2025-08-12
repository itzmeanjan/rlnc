mod avx2;
mod ssse3;

pub(super) fn gf256_inplace_mul_vec_by_scalar(vec: &mut [u8], scalar: u8) -> bool {
    if is_x86_feature_detected!("avx2") {
        unsafe { avx2::mul_vec_by_scalar(vec, scalar) };
        return true;
    }

    if is_x86_feature_detected!("ssse3") {
        unsafe { ssse3::mul_vec_by_scalar(vec, scalar) };
        return true;
    }

    false
}

pub(super) fn gf256_inplace_add_vectors(vec_dst: &mut [u8], vec_src: &[u8]) -> bool {
    if is_x86_feature_detected!("avx2") {
        unsafe { avx2::add_vec_into(vec_dst, vec_src) };
        return true;
    }

    if is_x86_feature_detected!("ssse3") {
        unsafe { ssse3::add_vec_into(vec_dst, vec_src) };
        return true;
    }

    false
}

pub(super) fn gf256_mul_vec_by_scalar_then_add_into_vec(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) -> bool {
    if is_x86_feature_detected!("avx2") {
        unsafe { avx2::mul_vec_by_scalar_then_add_into(add_into_vec, mul_vec, scalar) };
        return true;
    }

    if is_x86_feature_detected!("ssse3") {
        unsafe { ssse3::mul_vec_by_scalar_then_add_into(add_into_vec, mul_vec, scalar) };
        return true;
    }

    false
}
