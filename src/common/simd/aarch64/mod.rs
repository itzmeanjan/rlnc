use neon;

pub(super) unsafe fn gf256_inplace_mul_vec_by_scalar(vec: &mut [u8], scalar: u8) -> bool {
    if is_aarch64_feature_detected!("neon") {
        unsafe { neon::mul_vec_by_scalar(vec, scalar) };
        return true;
    }

    false
}

pub(super) fn gf256_inplace_add_vectors(vec_dst: &mut [u8], vec_src: &[u8]) -> bool {
    if is_aarch64_feature_detected!("neon") {
        unsafe { neon::add_vec_into(vec_dst, vec_src) };
        return true;
    }

    false
}

pub(super) fn gf256_mul_vec_by_scalar_then_add_into_vec(add_into_vec: &mut [u8], mul_vec: &[u8], scalar: u8) -> bool {
    if is_aarch64_feature_detected!("neon") {
        unsafe { neon::mul_vec_by_scalar_then_add_into(add_into_vec, mul_vec, scalar) };
        return true;
    }

    false
}
