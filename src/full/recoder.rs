use super::encoder::Encoder;
use crate::{RLNCError, common::gf256::Gf256};
use rand::Rng;

/// `Recoder` takes already coded pieces and recodes these coded pieces using
/// a new random sampled coding vector. This is useful for distributing coded
/// pieces more widely without needing to decode back to original data.
///
/// A `Recoder` essentially acts as a new encoder, but it operates on the *encoded* source pieces.
///
/// The `Recoder` stores the coding vectors and coded pieces from the input.
/// Internally, it uses an `Encoder` initialized with the coded pieces.
#[derive(Clone, Debug)]
pub struct Recoder {
    coding_vectors: Vec<Gf256>,
    encoder: Encoder,
    num_pieces_received: usize,
    full_coded_piece_byte_len: usize,
    num_pieces_coded_together: usize,
    /// A temporary buffer to hold the random recoding vector during the recoding process.
    /// This avoids repeated allocations on each recoding operation.
    random_recoding_vector: Vec<u8>,
    /// A temporary buffer to hold the output of the encoder during the recoding process.
    /// This avoids repeated allocations on each recoding operation.
    encoder_output_buf: Vec<u8>,
}

impl Recoder {
    /// Number of pieces original data got splitted into to be coded together.
    pub fn get_original_num_pieces_coded_together(&self) -> usize {
        self.num_pieces_coded_together
    }

    /// Number of pieces received by Recoder, which is getting recoded together, producing new pieces.
    pub fn get_num_pieces_recoded_together(&self) -> usize {
        self.num_pieces_received
    }

    /// After padding the original data, it gets splitted into `self.get_original_num_pieces_coded_together()` many pieces, which results into these many bytes per piece.
    pub fn get_piece_byte_len(&self) -> usize {
        self.full_coded_piece_byte_len - self.num_pieces_coded_together
    }

    /// Each full coded piece consists of `self.get_original_num_pieces_coded_together()` random coefficients, appended by corresponding encoded piece of `self.get_piece_byte_len()` bytes.
    pub fn get_full_coded_piece_byte_len(&self) -> usize {
        self.full_coded_piece_byte_len
    }

    /// Creates a new `Recoder` instance from a vector of received coded pieces.
    ///
    /// Each full coded piece in `data` is of `full_coded_piece_byte_len` bytes.
    /// A full coded piece = coding vector ++ coded piece
    ///
    /// The `Recoder` extracts the coding vectors and coded pieces from the input
    /// data. It then initializes an internal `Encoder` that implicitly
    /// represents the source pieces extracted from the input.
    ///
    /// # Arguments
    ///
    /// * `data`: A vector of bytes containing the concatenated full coded pieces, each of
    ///   `full_coded_piece_byte_len` bytes length.
    /// * `full_coded_piece_byte_len`: The byte length of a full coded piece.
    /// * `num_pieces_coded_together`: The number of original pieces that were
    ///   linearly combined to create each coded piece. This is also the length
    ///   of the coding vector prepended to each full coded piece.
    ///
    /// # Returns
    /// Returns `Ok(Recoder)` on successful creation.
    /// Returns `Err(RLNCError::NotEnoughPiecesToRecode)` if the input `data` is empty
    /// or does not contain at least one full coded piece.
    /// Returns `Err(RLNCError::PieceLengthZero)` if `full_coded_piece_byte_len` is zero.
    /// Returns `Err(RLNCError::PieceCountZero)` if `num_pieces_coded_together` is zero.
    /// Returns `Err(RLNCError::PieceLengthTooShort)` if `full_coded_piece_byte_len` is not greater than `num_pieces_coded_together`.
    pub fn new(data: Vec<u8>, full_coded_piece_byte_len: usize, num_pieces_coded_together: usize) -> Result<Recoder, RLNCError> {
        if data.is_empty() {
            return Err(RLNCError::NotEnoughPiecesToRecode);
        }
        if full_coded_piece_byte_len == 0 {
            return Err(RLNCError::PieceLengthZero);
        }
        if num_pieces_coded_together == 0 {
            return Err(RLNCError::PieceCountZero);
        }
        if full_coded_piece_byte_len <= num_pieces_coded_together {
            return Err(RLNCError::PieceLengthTooShort);
        }

        let piece_byte_len = full_coded_piece_byte_len - num_pieces_coded_together;
        let num_pieces_received = data.len() / full_coded_piece_byte_len;

        let mut coding_vectors = Vec::with_capacity(num_pieces_received * num_pieces_coded_together);
        let mut coded_pieces = Vec::with_capacity(num_pieces_received * piece_byte_len);

        data.chunks_exact(full_coded_piece_byte_len).for_each(|full_coded_piece| {
            let coding_vector = &full_coded_piece[..num_pieces_coded_together];
            let coded_piece = &full_coded_piece[num_pieces_coded_together..];

            coding_vectors.extend(coding_vector.iter().map(|&symbol| Gf256::new(symbol)));
            coded_pieces.extend_from_slice(coded_piece);
        });

        let encoder = unsafe { Encoder::without_padding(coded_pieces, num_pieces_received).unwrap_unchecked() };

        // Pre-allocate internal workspace buffers to avoid repeated allocations during recoding.
        let random_recoding_vector = vec![0u8; num_pieces_received];
        let encoder_output_buf = vec![0u8; encoder.get_full_coded_piece_byte_len()];

        Ok(Recoder {
            coding_vectors,
            encoder,
            num_pieces_received,
            full_coded_piece_byte_len,
            num_pieces_coded_together,
            random_recoding_vector,
            encoder_output_buf,
        })
    }

    /// Generates a new coded piece by recoding the source pieces using a randomly sampled coding vector.
    ///
    /// This method generates a random recoding vector (length `self.get_num_pieces_recoded_together()`),
    /// computes the resulting coding vector for the original source pieces by
    /// multiplying the random vector by the matrix of received coding vectors,
    /// and then uses the internal `Encoder` to produce a new coded piece based
    /// on this computed coding vector.
    ///
    /// The output is a vector containing the computed source coding vector
    /// prepended to the newly generated coded piece.
    ///
    /// # Arguments
    ///
    /// * `rng`: Used to sample the random recoding vector.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<u8>` representing the new coded piece prepended with its
    /// source coding vector. The length of the returned vector is
    /// `self.get_full_coded_piece_byte_len()`.
    pub fn recode<R: Rng + ?Sized>(&mut self, rng: &mut R, full_recoded_piece: &mut [u8]) -> Result<(), RLNCError> {
        if full_recoded_piece.len() != self.full_coded_piece_byte_len {
            return Err(RLNCError::InvalidOutputBuffer);
        }

        rng.fill_bytes(&mut self.random_recoding_vector);

        // Compute the resulting coding vector for the original source pieces
        // by multiplying the random sampled recoding vector by the matrix of received coding vectors.
        let (computed_cv_slice, _) = full_recoded_piece.split_at_mut(self.num_pieces_coded_together);
        for (coeff_idx, item) in computed_cv_slice.iter_mut().enumerate().take(self.num_pieces_coded_together) {
            let computed_coeff = self
                .random_recoding_vector
                .iter()
                .enumerate()
                .fold(Gf256::default(), |acc, (recoding_vec_idx, &cur)| {
                    let row_begins_at = recoding_vec_idx * self.num_pieces_coded_together;
                    acc + Gf256::new(cur) * self.coding_vectors[row_begins_at + coeff_idx]
                });
            *item = computed_coeff.get();
        }

        unsafe {
            self.encoder
                .code_with_buf_coding_vector(&self.random_recoding_vector, &mut self.encoder_output_buf)
                .unwrap_unchecked()
        };
        let coded_piece = &self.encoder_output_buf[self.num_pieces_received..];

        full_recoded_piece[self.num_pieces_coded_together..].copy_from_slice(coded_piece);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{RLNCError, Recoder};
    use crate::full::encoder::Encoder;
    use rand::Rng;

    #[test]
    fn test_recoder_new_invalid_inputs() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize;
        let encoder = Encoder::new((0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>(), piece_count)
            .expect("Failed to create Encoder for recoder new invalid inputs test");
        let full_coded_piece_byte_len = encoder.get_full_coded_piece_byte_len();
        let num_pieces_coded_together = encoder.get_piece_count();

        // Test case 1: Empty `data` vector
        let empty_data: Vec<u8> = Vec::new();
        let result_empty_data = Recoder::new(empty_data, full_coded_piece_byte_len, num_pieces_coded_together);
        assert!(result_empty_data.is_err());
        assert_eq!(
            result_empty_data.expect_err("Expected NotEnoughPiecesToRecode error for empty data"),
            RLNCError::NotEnoughPiecesToRecode
        );

        // Test case 2: `full_coded_piece_byte_len` is zero
        let data_non_empty = vec![1, 2, 3]; // Needs at least one piece worth of data for non-empty input
        let result_zero_full_len = Recoder::new(data_non_empty.clone(), 0, num_pieces_coded_together);
        assert!(result_zero_full_len.is_err());
        assert_eq!(
            result_zero_full_len.expect_err("Expected PieceLengthZero error for zero full coded piece length"),
            RLNCError::PieceLengthZero
        );

        // Test case 3: `num_pieces_coded_together` is zero
        let result_zero_piece_count = Recoder::new(data_non_empty.clone(), full_coded_piece_byte_len, 0);
        assert!(result_zero_piece_count.is_err());
        assert_eq!(
            result_zero_piece_count.expect_err("Expected PieceCountZero error for zero pieces coded together"),
            RLNCError::PieceCountZero
        );

        // Test case 4: `full_coded_piece_byte_len` is not greater than `num_pieces_coded_together`
        // Case 4.1: Equal
        let result_equal_len = Recoder::new(
            data_non_empty.clone(),
            num_pieces_coded_together, // full_coded_piece_byte_len = num_pieces_coded_together
            num_pieces_coded_together,
        );
        assert!(result_equal_len.is_err());
        assert_eq!(
            result_equal_len.expect_err("Expected PieceLengthTooShort error when full length equals piece count"),
            RLNCError::PieceLengthTooShort
        );

        // Case 4.2: Less than
        let result_less_len = Recoder::new(
            data_non_empty.clone(),
            num_pieces_coded_together - 1, // full_coded_piece_byte_len < num_pieces_coded_together
            num_pieces_coded_together,
        );
        assert!(result_less_len.is_err());
        assert_eq!(
            result_less_len.expect_err("Expected PieceLengthTooShort error when full length is less than piece count"),
            RLNCError::PieceLengthTooShort
        );

        // Test case 5: Valid input (using existing encoder setup to generate valid data)
        let num_pieces_to_recode_with = 5;
        let coded_pieces_for_recoder: Vec<u8> = (0..num_pieces_to_recode_with).flat_map(|_| encoder.code(&mut rng)).collect();

        let result_valid = Recoder::new(coded_pieces_for_recoder, full_coded_piece_byte_len, num_pieces_coded_together);
        assert!(result_valid.is_ok());
        let recoder = result_valid.expect("Expected Recoder to be created successfully with valid inputs");
        assert_eq!(recoder.get_original_num_pieces_coded_together(), num_pieces_coded_together);
        assert_eq!(recoder.get_num_pieces_recoded_together(), num_pieces_to_recode_with);
    }

    #[test]
    fn test_recoder_getters() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize; // Original number of pieces
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder for recoder getters test");

        let num_pieces_to_recode_with = 10; // Number of coded pieces given to recoder
        let full_coded_piece_byte_len = encoder.get_full_coded_piece_byte_len();
        let original_piece_byte_len = encoder.get_piece_byte_len();

        let coded_pieces_for_recoder: Vec<u8> = (0..num_pieces_to_recode_with).flat_map(|_| encoder.code(&mut rng)).collect();

        let recoder = Recoder::new(
            coded_pieces_for_recoder,
            full_coded_piece_byte_len,
            piece_count, // `num_pieces_coded_together` from original encoder
        )
        .expect("Recoder creation failed");

        assert_eq!(recoder.get_original_num_pieces_coded_together(), piece_count);
        assert_eq!(recoder.get_num_pieces_recoded_together(), num_pieces_to_recode_with);
        assert_eq!(recoder.get_piece_byte_len(), original_piece_byte_len);
        assert_eq!(recoder.get_full_coded_piece_byte_len(), full_coded_piece_byte_len);
    }
}
