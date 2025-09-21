//! Following GF(2**8) logarithm and exponentiation tables are generated using
//! Python script @ <https://gist.github.com/itzmeanjan/0b2ec3f378de2c2e911bd4bb5505d45a>.

use rand::Rng;
use rand::distr::{Distribution, StandardUniform};
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

pub const GF256_ORDER: usize = u8::MAX as usize + 1;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub const GF256_BIT_WIDTH: usize = u8::BITS as usize;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub const GF256_HALF_ORDER: usize = 1usize << (GF256_BIT_WIDTH / 2);

const GF256_LOG_TABLE: [u8; GF256_ORDER] = [
    0, 0, 25, 1, 50, 2, 26, 198, 75, 199, 27, 104, 51, 238, 223, 3, 100, 4, 224, 14, 52, 141, 129, 239, 76, 113, 8, 200, 248, 105, 28, 193, 125, 194, 29, 181,
    249, 185, 39, 106, 77, 228, 166, 114, 154, 201, 9, 120, 101, 47, 138, 5, 33, 15, 225, 36, 18, 240, 130, 69, 53, 147, 218, 142, 150, 143, 219, 189, 54, 208,
    206, 148, 19, 92, 210, 241, 64, 70, 131, 56, 102, 221, 253, 48, 191, 6, 139, 98, 179, 37, 226, 152, 34, 136, 145, 16, 126, 110, 72, 195, 163, 182, 30, 66,
    58, 107, 40, 84, 250, 133, 61, 186, 43, 121, 10, 21, 155, 159, 94, 202, 78, 212, 172, 229, 243, 115, 167, 87, 175, 88, 168, 80, 244, 234, 214, 116, 79,
    174, 233, 213, 231, 230, 173, 232, 44, 215, 117, 122, 235, 22, 11, 245, 89, 203, 95, 176, 156, 169, 81, 160, 127, 12, 246, 111, 23, 196, 73, 236, 216, 67,
    31, 45, 164, 118, 123, 183, 204, 187, 62, 90, 251, 96, 177, 134, 59, 82, 161, 108, 170, 85, 41, 157, 151, 178, 135, 144, 97, 190, 220, 252, 188, 149, 207,
    205, 55, 63, 91, 209, 83, 57, 132, 60, 65, 162, 109, 71, 20, 42, 158, 93, 86, 242, 211, 171, 68, 17, 146, 217, 35, 32, 46, 137, 180, 124, 184, 38, 119,
    153, 227, 165, 103, 74, 237, 222, 197, 49, 254, 24, 13, 99, 140, 128, 192, 247, 112, 7,
];

const GF256_EXP_TABLE: [u8; 2 * GF256_ORDER - 2] = [
    1, 3, 5, 15, 17, 51, 85, 255, 26, 46, 114, 150, 161, 248, 19, 53, 95, 225, 56, 72, 216, 115, 149, 164, 247, 2, 6, 10, 30, 34, 102, 170, 229, 52, 92, 228,
    55, 89, 235, 38, 106, 190, 217, 112, 144, 171, 230, 49, 83, 245, 4, 12, 20, 60, 68, 204, 79, 209, 104, 184, 211, 110, 178, 205, 76, 212, 103, 169, 224, 59,
    77, 215, 98, 166, 241, 8, 24, 40, 120, 136, 131, 158, 185, 208, 107, 189, 220, 127, 129, 152, 179, 206, 73, 219, 118, 154, 181, 196, 87, 249, 16, 48, 80,
    240, 11, 29, 39, 105, 187, 214, 97, 163, 254, 25, 43, 125, 135, 146, 173, 236, 47, 113, 147, 174, 233, 32, 96, 160, 251, 22, 58, 78, 210, 109, 183, 194,
    93, 231, 50, 86, 250, 21, 63, 65, 195, 94, 226, 61, 71, 201, 64, 192, 91, 237, 44, 116, 156, 191, 218, 117, 159, 186, 213, 100, 172, 239, 42, 126, 130,
    157, 188, 223, 122, 142, 137, 128, 155, 182, 193, 88, 232, 35, 101, 175, 234, 37, 111, 177, 200, 67, 197, 84, 252, 31, 33, 99, 165, 244, 7, 9, 27, 45, 119,
    153, 176, 203, 70, 202, 69, 207, 74, 222, 121, 139, 134, 145, 168, 227, 62, 66, 198, 81, 243, 14, 18, 54, 90, 238, 41, 123, 141, 140, 143, 138, 133, 148,
    167, 242, 13, 23, 57, 75, 221, 124, 132, 151, 162, 253, 28, 36, 108, 180, 199, 82, 246, 1, 3, 5, 15, 17, 51, 85, 255, 26, 46, 114, 150, 161, 248, 19, 53,
    95, 225, 56, 72, 216, 115, 149, 164, 247, 2, 6, 10, 30, 34, 102, 170, 229, 52, 92, 228, 55, 89, 235, 38, 106, 190, 217, 112, 144, 171, 230, 49, 83, 245, 4,
    12, 20, 60, 68, 204, 79, 209, 104, 184, 211, 110, 178, 205, 76, 212, 103, 169, 224, 59, 77, 215, 98, 166, 241, 8, 24, 40, 120, 136, 131, 158, 185, 208,
    107, 189, 220, 127, 129, 152, 179, 206, 73, 219, 118, 154, 181, 196, 87, 249, 16, 48, 80, 240, 11, 29, 39, 105, 187, 214, 97, 163, 254, 25, 43, 125, 135,
    146, 173, 236, 47, 113, 147, 174, 233, 32, 96, 160, 251, 22, 58, 78, 210, 109, 183, 194, 93, 231, 50, 86, 250, 21, 63, 65, 195, 94, 226, 61, 71, 201, 64,
    192, 91, 237, 44, 116, 156, 191, 218, 117, 159, 186, 213, 100, 172, 239, 42, 126, 130, 157, 188, 223, 122, 142, 137, 128, 155, 182, 193, 88, 232, 35, 101,
    175, 234, 37, 111, 177, 200, 67, 197, 84, 252, 31, 33, 99, 165, 244, 7, 9, 27, 45, 119, 153, 176, 203, 70, 202, 69, 207, 74, 222, 121, 139, 134, 145, 168,
    227, 62, 66, 198, 81, 243, 14, 18, 54, 90, 238, 41, 123, 141, 140, 143, 138, 133, 148, 167, 242, 13, 23, 57, 75, 221, 124, 132, 151, 162, 253, 28, 36, 108,
    180, 199, 82, 246,
];

/// Galois Field GF(2^8) wrapper type.
///
/// This type represents elements of the Galois Field GF(2^8), which is commonly used in coding theory, cryptography, and error correction codes.
/// It supports basic arithmetic operations such as addition, subtraction, multiplication, and division.
/// The operations are defined over the finite field GF(2^8) with the irreducible polynomial x^8 + x^4 + x^3 + x^2 + 1
/// and the primitive element x = 2.
///
/// We assign the `transparent` attribute to ensure that the Rust compiler representation of `Gf256` is the same as its underlying `u8` value,
/// providing a guarantee that it can be used interchangeably with `u8` in contexts where the underlying value is needed.
#[repr(transparent)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Gf256 {
    val: u8,
}

impl Gf256 {
    /// Creates a new Gf256 element from a u8 value.
    pub const fn new(val: u8) -> Self {
        Gf256 { val }
    }

    /// Returns the raw u8 value of the Gf256 element.
    pub const fn get(&self) -> u8 {
        self.val
    }

    /// Returns the additive identity element (0).
    pub const fn zero() -> Self {
        Gf256::new(0)
    }

    /// Returns the multiplicative identity element (1).
    pub const fn one() -> Self {
        Gf256::new(1)
    }

    /// Returns primitive element x + 1, for GF(2^8) field with irreducible polynomial x^8 + x^4 + x^3 + x + 1.
    pub const fn primitive_element() -> Self {
        Gf256::new(3)
    }

    /// Compile-time executable multiplication of two bytes, over GF(2^8).
    pub const fn mul_const(a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }

        let l = GF256_LOG_TABLE[a as usize] as usize;
        let r = GF256_LOG_TABLE[b as usize] as usize;

        GF256_EXP_TABLE[l + r]
    }

    /// Computes the multiplicative inverse of the element. Returns `None` for the zero element.
    pub const fn inv(self) -> Option<Self> {
        if self.val == 0 {
            return None;
        }

        Some(Gf256 {
            val: GF256_EXP_TABLE[(GF256_ORDER - 1) - GF256_LOG_TABLE[self.val as usize] as usize],
        })
    }
}

impl Add for Gf256 {
    type Output = Self;

    /// Performs addition (XOR) of two Gf256 elements.
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Gf256 { val: self.val ^ rhs.val }
    }
}

impl AddAssign for Gf256 {
    /// Performs in-place addition i.e. compound addition operation (XOR) of two Gf256 elements.
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Self) {
        self.val ^= rhs.val;
    }
}

impl Neg for Gf256 {
    type Output = Self;

    /// Computes the additive inverse (itself, as XOR is self-inverse).
    fn neg(self) -> Self::Output {
        Gf256 { val: self.val }
    }
}

impl Sub for Gf256 {
    type Output = Self;

    /// Performs subtraction (XOR) of two Gf256 elements.
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        Gf256 { val: self.val ^ rhs.val }
    }
}

impl Mul for Gf256 {
    type Output = Self;

    /// Performs multiplication of two Gf256 elements using logarithm and exponentiation tables.
    fn mul(self, rhs: Self) -> Self::Output {
        Gf256 {
            val: Self::mul_const(self.val, rhs.val),
        }
    }
}

impl Div for Gf256 {
    type Output = Option<Self>;

    /// Performs division of two Gf256 elements using multiplicative inverse. Returns `None` if dividing by zero.
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        rhs.inv().map(|rhs_inv| self * rhs_inv)
    }
}

impl PartialEq for Gf256 {
    /// Checks for equality between two Gf256 elements.
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl Distribution<Gf256> for StandardUniform {
    /// Samples a random Gf256 element.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gf256 {
        Gf256 { val: rng.random() }
    }
}

#[cfg(test)]
mod test {
    use super::Gf256;
    use rand::Rng;

    #[test]
    fn prop_test_gf256_operations() {
        const NUM_TEST_ITERATIONS: usize = 100_000;

        let mut rng = rand::rng();

        (0..NUM_TEST_ITERATIONS).for_each(|_| {
            let a: Gf256 = rng.random();
            let b: Gf256 = rng.random();

            // Addition, Subtraction, Negation
            let sum = a + b;
            let diff = sum - b;

            assert_eq!(diff, a);

            // Multiplication, Division, Inversion
            let mul = a * b;
            let div = mul / b;

            if b == Gf256::zero() {
                assert_eq!(div, None);
                assert_eq!(mul, Gf256::zero());
            } else {
                assert_eq!(a, div.unwrap());
            }
        });
    }
}
