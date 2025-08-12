pub mod errors;
pub mod gf256;
pub mod simd;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
mod simd_mul_table;
