use super::consts::BOUNDARY_MARKER;
use crate::RLNCError;
use rand::Rng;

#[cfg(all(feature = "parallel", not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))))]
use crate::common::gf256::Gf256;
#[cfg(not(feature = "parallel"))]
use crate::common::simd::gf256_mul_vec_by_scalar_then_add_into_vec;
#[cfg(all(feature = "parallel", any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
use crate::common::simd::{gf256_inplace_add_vectors, gf256_inplace_mul_vec_by_scalar};
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Represents an RLNC encoder, responsible for dividing data into pieces and
/// generating coded pieces based on random sampled coding vectors.
#[derive(Clone, Debug)]
pub struct Encoder {
    data: Vec<u8>,
    piece_count: usize,
    piece_byte_len: usize,
}

impl Encoder {
    /// Number of pieces original data got splitted into and being coded together.
    pub fn get_piece_count(&self) -> usize {
        self.piece_count
    }

    /// After padding the original data, it gets splitted into `self.get_piece_count()` many pieces, which results into these many bytes per piece.
    pub fn get_piece_byte_len(&self) -> usize {
        self.piece_byte_len
    }

    /// Each full coded piece consists of `self.get_piece_count()` random coefficients, appended by corresponding encoded piece of `self.get_piece_byte_len()` bytes.
    pub fn get_full_coded_piece_byte_len(&self) -> usize {
        self.get_piece_count() + self.get_piece_byte_len()
    }

    /// Creates a new `Encoder` without adding any padding to the input data.
    /// This is suitable if the input data length is already a multiple of the
    /// desired piece count. This interface is used by Recoder.
    ///
    /// Returns `Ok(Encoder)` on success.
    /// Returns `Err(RLNCError::DataLengthZero)` if `data` is empty.
    /// Returns `Err(RLNCError::PieceCountZero)` if `piece_count` is zero.
    /// Returns `Err(RLNCError::DataLengthMismatch)` if the data length is not a
    /// multiple of the piece count.
    pub(crate) fn without_padding(data: Vec<u8>, piece_count: usize) -> Result<Encoder, RLNCError> {
        if data.is_empty() {
            return Err(RLNCError::DataLengthZero);
        }
        if piece_count == 0 {
            return Err(RLNCError::PieceCountZero);
        }

        let in_data_len = data.len();
        let piece_byte_len = in_data_len / piece_count;
        let computed_total_data_len = piece_byte_len * piece_count;

        if computed_total_data_len != in_data_len {
            return Err(RLNCError::DataLengthMismatch);
        }

        Ok(Encoder {
            data,
            piece_count,
            piece_byte_len,
        })
    }

    /// Creates a new `Encoder` while padding the input data.
    ///
    /// The input data is padded with zeros to ensure its length is a multiple
    /// of `piece_count * piece_byte_len`, where `piece_byte_len` is calculated
    /// such that the original data plus a boundary marker fits within
    /// `piece_count` pieces. A boundary marker (`BOUNDARY_MARKER`) is placed
    /// at the end of the original data before zero padding.
    ///
    /// # Returns
    /// Returns `Ok(Encoder)` on success.
    /// Returns `Err(RLNCError::DataLengthZero)` if `data` is empty.
    /// Returns `Err(RLNCError::PieceCountZero)` if `piece_count` is zero.
    pub fn new(mut data: Vec<u8>, piece_count: usize) -> Result<Encoder, RLNCError> {
        if data.is_empty() {
            return Err(RLNCError::DataLengthZero);
        }
        if piece_count == 0 {
            return Err(RLNCError::PieceCountZero);
        }

        let in_data_len = data.len();
        let boundary_marker_len = 1;
        let piece_byte_len = (in_data_len + boundary_marker_len).div_ceil(piece_count);
        let padded_data_len = piece_count * piece_byte_len;

        data.resize(padded_data_len, 0);
        data[in_data_len] = BOUNDARY_MARKER;

        Ok(Encoder {
            data,
            piece_count,
            piece_byte_len,
        })
    }

    /// Encodes the data held by the encoder using a provided coding vector.
    ///
    /// The resulting coded piece is returned as a `Vec<u8>`, prefixed by the
    /// coding vector itself (as `u8` values). The total length of the returned
    /// vector is `self.get_full_coded_piece_byte_len()`.
    ///
    /// Returns `RLNCError::CodingVectorLengthMismatch` if the length of the
    /// provided `coding_vector` does not match `self.piece_count`.
    ///
    /// # Deprecated
    /// Use the allocation friendly `code_with_buf_coding_vector` instead.
    /// This method is deprecated due to performance and memory usage concerns.
    /// It uses a vector allocation for each coded piece, which can be inefficient.
    #[deprecated(since = "0.9.0", note = "Use `code_with_buf_coding_vector` instead for better performance and memory usage.")]
    #[cfg(not(feature = "parallel"))]
    pub fn code_with_coding_vector(&self, coding_vector: &[u8]) -> Result<Vec<u8>, RLNCError> {
        if coding_vector.len() != self.piece_count {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        let mut full_coded_piece = vec![0u8; self.get_full_coded_piece_byte_len()];
        full_coded_piece[..self.piece_count].copy_from_slice(coding_vector);

        let coded_piece = &mut full_coded_piece[self.piece_count..];
        self.data
            .chunks_exact(self.piece_byte_len)
            .zip(coding_vector)
            .for_each(|(piece, &random_symbol)| gf256_mul_vec_by_scalar_then_add_into_vec(coded_piece, piece, random_symbol));

        Ok(full_coded_piece)
    }

    /// Encodes the data held by the encoder using a provided coding vector.
    ///
    /// The resulting coded piece is written into the provided output buffer `out_buf`,
    /// coding vector itself (as `u8` values). The total length of the returned
    /// vector is `self.get_full_coded_piece_byte_len()`.
    ///
    /// Returns `RLNCError::CodingVectorLengthMismatch` if the length of the
    /// provided `coding_vector` does not match `self.piece_count`.
    #[cfg(not(feature = "parallel"))]
    pub fn code_with_buf_coding_vector(&self, coding_vector: &[u8], out_buf: &mut [u8]) -> Result<(), RLNCError> {
        if coding_vector.len() != self.piece_count {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        if out_buf.len() < self.get_full_coded_piece_byte_len() {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        // We will use the out_buf by using `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
        let (coding_vector_buf, coded_piece_buf) = out_buf.split_at_mut(self.piece_count);
        coding_vector_buf[..self.piece_count].copy_from_slice(coding_vector);
        coded_piece_buf.fill(0); // Initialize the coded piece buffer to zero

        self.data
            .chunks_exact(self.piece_byte_len)
            .zip(coding_vector)
            .for_each(|(piece, &random_symbol)| gf256_mul_vec_by_scalar_then_add_into_vec(coded_piece_buf, piece, random_symbol));

        Ok(())
    }

    /// This function is deprecated.
    ///
    /// Encodes the data held by the encoder using a provided coding vector.
    ///
    /// The resulting coded piece is returned as a `Vec<u8>`, prefixed by the
    /// coding vector itself (as `u8` values). The total length of the returned
    /// vector is `self.get_complete_coded_piece_byte_len()`.
    ///
    /// Returns `RLNCError::CodingVectorLengthMismatch` if the length of the
    /// provided `coding_vector` does not match `self.piece_count`.
    ///
    /// # Deprecated
    /// Use the allocation friendly `code_with_buf_coding_vector` instead.
    /// This method is deprecated due to performance and memory usage concerns.
    /// It uses a vector allocation for each coded piece, which can be inefficient.
    #[deprecated(since = "0.9.0", note = "Use `code_with_buf_coding_vector` instead for better performance and memory usage.")]
    #[cfg(feature = "parallel")]
    pub fn code_with_coding_vector(&self, coding_vector: &[u8]) -> Result<Vec<u8>, RLNCError> {
        if coding_vector.len() != self.piece_count {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        let coded_piece = self
            .data
            .par_chunks_exact(self.piece_byte_len)
            .zip(coding_vector)
            .map(|(piece, &random_symbol)| {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                {
                    let mut scalar_x_piece = piece.to_vec();
                    gf256_inplace_mul_vec_by_scalar(&mut scalar_x_piece, random_symbol);

                    scalar_x_piece
                }

                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    piece.iter().map(move |&symbol| (Gf256::new(symbol) * Gf256::new(random_symbol)).get())
                }
            })
            .fold(
                || vec![0u8; self.piece_byte_len],
                |mut acc, cur| {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                    gf256_inplace_add_vectors(&mut acc, &cur);

                    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                    acc.iter_mut().zip(cur).for_each(|(a, b)| {
                        *a ^= b;
                    });

                    acc
                },
            )
            .reduce(
                || vec![0u8; self.piece_byte_len],
                |mut acc, cur| {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                    gf256_inplace_add_vectors(&mut acc, &cur);

                    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                    acc.iter_mut().zip(cur).for_each(|(a, b)| {
                        *a ^= b;
                    });

                    acc
                },
            );

        let mut full_coded_piece = vec![0u8; self.get_full_coded_piece_byte_len()];
        full_coded_piece[..self.piece_count].copy_from_slice(coding_vector);
        full_coded_piece[self.piece_count..].copy_from_slice(&coded_piece);

        Ok(full_coded_piece)
    }
    /// Encodes the data held by the encoder using a provided coding vector.
    ///
    /// The resulting coded piece is written into the provided output buffer `out_buf`,
    /// prefixed by the coding vector itself (as `u8` values). The total
    /// length of the output buffer is `self.get_complete_coded_piece_byte_len()`.
    ///
    /// Returns `RLNCError::CodingVectorLengthMismatch` if the length of the
    /// provided `coding_vector` does not match `self.piece_count`.
    #[cfg(feature = "parallel")]
    pub fn code_with_buf_coding_vector(&self, coding_vector: &[u8], out_buf: &mut [u8]) -> Result<(), RLNCError> {
        if coding_vector.len() != self.piece_count {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        let coded_piece = self
            .data
            .par_chunks_exact(self.piece_byte_len)
            .zip(coding_vector)
            .map(|(piece, &random_symbol)| {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                {
                    let mut scalar_x_piece = piece.to_vec();
                    gf256_inplace_mul_vec_by_scalar(&mut scalar_x_piece, random_symbol);

                    scalar_x_piece
                }

                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    piece.iter().map(move |&symbol| (Gf256::new(symbol) * Gf256::new(random_symbol)).get())
                }
            })
            .fold(
                || vec![0u8; self.piece_byte_len],
                |mut acc, cur| {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                    gf256_inplace_add_vectors(&mut acc, &cur);

                    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                    acc.iter_mut().zip(cur).for_each(|(a, b)| {
                        *a ^= b;
                    });

                    acc
                },
            )
            .reduce(
                || vec![0u8; self.piece_byte_len],
                |mut acc, cur| {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
                    gf256_inplace_add_vectors(&mut acc, &cur);

                    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
                    acc.iter_mut().zip(cur).for_each(|(a, b)| {
                        *a ^= b;
                    });

                    acc
                },
            );

        // Use the out_buf to store the full coded piece
        if out_buf.len() < self.get_full_coded_piece_byte_len() {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        out_buf[..self.piece_count].copy_from_slice(coding_vector);
        out_buf[self.piece_count..].copy_from_slice(&coded_piece);

        Ok(())
    }

    /// Encodes the data held by the encoder using a randomly sampled coding vector.
    ///
    /// A coding vector of `self.piece_count` random `Gf256` symbols is generated
    /// using the provided random number generator.
    ///
    /// Calls `code_with_buf_coding_vector` internally.
    ///
    /// Returns the coded piece prefixed by the random coding vector.
    pub fn code<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<u8> {
        let random_coding_vector = (0..self.piece_count).map(|_| rng.random()).collect::<Vec<u8>>();

        // Allocate once outside the hot loop so we don't have to reallocate on each call
        let mut coded_piece = vec![0u8; self.get_full_coded_piece_byte_len()];
        unsafe { self.code_with_buf_coding_vector(&random_coding_vector, &mut coded_piece).unwrap_unchecked() }
        coded_piece
    }
}

#[cfg(test)]
mod tests {
    use super::{Encoder, RLNCError};
    use rand::Rng;

    #[test]
    fn test_encoder_without_padding_invalid_data() {
        let mut rng = rand::rng();

        // Test case: Data length is 0
        let data_byte_len_zero = 0usize;
        let piece_count_non_zero = 10usize;
        let data_zero: Vec<u8> = (0..data_byte_len_zero).map(|_| rng.random()).collect();

        let result_data_zero = Encoder::without_padding(data_zero, piece_count_non_zero);
        assert!(result_data_zero.is_err());
        assert_eq!(result_data_zero.expect_err("Expected DataLengthZero error"), RLNCError::DataLengthZero);

        // Test case: Piece count is 0
        let data_byte_len_non_zero = 100usize;
        let piece_count_zero = 0usize;
        let data_non_zero: Vec<u8> = (0..data_byte_len_non_zero).map(|_| rng.random()).collect();

        let result_piece_count_zero = Encoder::without_padding(data_non_zero, piece_count_zero);
        assert!(result_piece_count_zero.is_err());
        assert_eq!(result_piece_count_zero.expect_err("Expected PieceCountZero error"), RLNCError::PieceCountZero);

        // Test case: Data length not a multiple of piece_count
        let data_byte_len = 1001usize; // Not a multiple of 32
        let piece_count = 32usize;
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

        let result = Encoder::without_padding(data, piece_count);
        assert!(result.is_err());
        assert_eq!(result.expect_err("Expected DataLengthMismatch error"), RLNCError::DataLengthMismatch);

        // Test case: Valid input
        let data_byte_len_valid = 100usize;
        let piece_count_valid = 10usize;
        let data_valid = (0..data_byte_len_valid).map(|_| rng.random()).collect::<Vec<u8>>();

        let result_valid = Encoder::without_padding(data_valid, piece_count_valid);
        assert!(result_valid.is_ok());
    }

    #[test]
    fn test_encoder_new_invalid_inputs() {
        let mut rng = rand::rng();

        // Test case: Data length is 0
        let data_byte_len_zero = 0;
        let piece_count_non_zero = 5;
        let data_zero: Vec<u8> = (0..data_byte_len_zero).map(|_| rng.random()).collect();

        let result_data_zero = Encoder::new(data_zero, piece_count_non_zero);
        assert!(result_data_zero.is_err());
        assert_eq!(result_data_zero.expect_err("Expected DataLengthZero error"), RLNCError::DataLengthZero);

        // Test case: Piece count is 0
        let data_byte_len_non_zero = 100;
        let piece_count_zero = 0;
        let data_non_zero: Vec<u8> = (0..data_byte_len_non_zero).map(|_| rng.random()).collect();

        let result_piece_count_zero = Encoder::new(data_non_zero, piece_count_zero);
        assert!(result_piece_count_zero.is_err());
        assert_eq!(result_piece_count_zero.expect_err("Expected PieceCountZero error"), RLNCError::PieceCountZero);

        // Test case: Both data length and piece count are 0
        let data_byte_len_both_zero = 0;
        let piece_count_both_zero = 0;
        let data_both_zero: Vec<u8> = (0..data_byte_len_both_zero).map(|_| rng.random()).collect();

        let result_both_zero = Encoder::new(data_both_zero, piece_count_both_zero);
        assert!(result_both_zero.is_err());
        assert_eq!(
            result_both_zero.expect_err("Expected DataLengthZero error for both zero inputs"),
            RLNCError::DataLengthZero
        );

        // Test case 4: Valid input
        let data_byte_len_valid = 1024;
        let piece_count_valid = 32;
        let data_valid = (0..data_byte_len_valid).map(|_| rng.random()).collect::<Vec<u8>>();

        let result_valid = Encoder::new(data_valid, piece_count_valid);
        assert!(result_valid.is_ok());
    }

    #[test]
    fn test_encoder_code_with_coding_vector_invalid_inputs() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize;
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder for invalid inputs test");

        // Test case 1: Coding vector is shorter than expected
        let short_coding_vector_len = piece_count - 1;
        let short_coding_vector: Vec<u8> = (0..short_coding_vector_len).map(|_| rng.random()).collect();
        let mut buf_short_coding_vector = vec![0u8; encoder.get_full_coded_piece_byte_len()];
        let result_short = encoder.code_with_buf_coding_vector(&short_coding_vector, &mut buf_short_coding_vector);

        assert!(result_short.is_err());
        assert_eq!(
            result_short.expect_err("Expected CodingVectorLengthMismatch error for short vector"),
            RLNCError::CodingVectorLengthMismatch
        );

        // Test case 2: Coding vector is longer than expected
        let long_coding_vector_len = piece_count + 1;
        let long_coding_vector: Vec<u8> = (0..long_coding_vector_len).map(|_| rng.random()).collect();
        let result_long = encoder.code_with_buf_coding_vector(&long_coding_vector, &mut vec![0u8; encoder.get_full_coded_piece_byte_len()]);

        assert!(result_long.is_err());
        assert_eq!(
            result_long.expect_err("Expected CodingVectorLengthMismatch error for long vector"),
            RLNCError::CodingVectorLengthMismatch
        );

        // Test case 3: Empty coding vector
        let empty_coding_vector: Vec<u8> = Vec::new();
        let result_empty = encoder.code_with_buf_coding_vector(&empty_coding_vector, &mut vec![0u8; encoder.get_full_coded_piece_byte_len()]);

        assert!(result_empty.is_err());
        assert_eq!(
            result_empty.expect_err("Expected CodingVectorLengthMismatch error for empty vector"),
            RLNCError::CodingVectorLengthMismatch
        );

        // Test case 4: Valid coding vector
        let valid_coding_vector: Vec<u8> = (0..piece_count).map(|_| rng.random()).collect();
        let mut buf_valid_coding_vector = vec![0u8; encoder.get_full_coded_piece_byte_len()];
        let result_valid = encoder.code_with_buf_coding_vector(&valid_coding_vector, &mut buf_valid_coding_vector);

        assert!(result_valid.is_ok());
        assert_eq!(buf_valid_coding_vector.len(), encoder.get_full_coded_piece_byte_len());
    }

    #[test]
    fn test_encoder_getters() {
        let mut rng = rand::rng();

        // Test case 1: Single piece (minimum piece count)
        let data_byte_len_single = 100usize;
        let piece_count_single = 1usize;
        let data_single = (0..data_byte_len_single).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder_single = Encoder::new(data_single.clone(), piece_count_single).expect("Failed to create Encoder (single piece)");

        assert_eq!(encoder_single.get_piece_count(), piece_count_single);
        assert_eq!(encoder_single.get_piece_byte_len(), (data_byte_len_single + 1).div_ceil(piece_count_single));
        assert_eq!(
            encoder_single.get_full_coded_piece_byte_len(),
            piece_count_single + (data_byte_len_single + 1).div_ceil(piece_count_single)
        );

        // Test case 2: Single byte data with single piece
        let piece_count_min = 1usize;
        let data_min = vec![42u8];
        let encoder_min = Encoder::new(data_min, piece_count_min).expect("Failed to create Encoder (min data)");

        assert_eq!(encoder_min.get_piece_count(), piece_count_min);
        assert_eq!(encoder_min.get_piece_byte_len(), 2); // 1 byte data + 1 boundary marker
        assert_eq!(encoder_min.get_full_coded_piece_byte_len(), 3); // 1 coeff + 2 data bytes

        // Test case 3: Data length equals piece count (each piece gets 1 data byte + padding)
        let data_byte_len_eq = 10usize;
        let piece_count_eq = 10usize;
        let data_eq = (0..data_byte_len_eq).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder_eq = Encoder::new(data_eq, piece_count_eq).expect("Failed to create Encoder (equal length)");

        assert_eq!(encoder_eq.get_piece_count(), piece_count_eq);
        assert_eq!(encoder_eq.get_piece_byte_len(), (data_byte_len_eq + 1).div_ceil(piece_count_eq)); // 2 bytes per piece
        assert_eq!(encoder_eq.get_full_coded_piece_byte_len(), piece_count_eq + 2);

        // Test case 4: Large piece count (many small pieces)
        let data_byte_len_large = 100usize;
        let piece_count_large = 50usize;
        let data_large = (0..data_byte_len_large).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder_large = Encoder::new(data_large, piece_count_large).expect("Failed to create Encoder (large piece count)");

        assert_eq!(encoder_large.get_piece_count(), piece_count_large);
        assert_eq!(encoder_large.get_piece_byte_len(), (data_byte_len_large + 1).div_ceil(piece_count_large));
        assert_eq!(
            encoder_large.get_full_coded_piece_byte_len(),
            piece_count_large + (data_byte_len_large + 1).div_ceil(piece_count_large)
        );
    }
}
