use super::consts::BOUNDARY_MARKER;
use crate::{RLNCError, full::decoder_matrix::DecoderMatrix};

/// Random Linear Network Code (RLNC) Decoder.
///
/// This struct manages the received coded pieces and performs Gaussian
/// elimination to recover the original data.
#[derive(Clone, Debug)]
pub struct Decoder {
    /// Stores the coefficient matrix and coded data rows concatenated.
    /// Each row is a coded piece: `[coefficients | data_piece]`.
    matrix: DecoderMatrix,
    /// The byte length of each original data piece.
    piece_byte_len: usize,
    /// The minimum number of useful coded pieces required to decode.
    required_piece_count: usize,
    /// The total number of coded pieces received so far.
    received_piece_count: usize,
    /// The number of linearly independent pieces received so far.
    useful_piece_count: usize,
}

impl Decoder {
    /// Number of pieces original data got splitted into and coded together.
    pub fn get_num_pieces_coded_together(&self) -> usize {
        self.required_piece_count
    }

    /// After padding the original data, it gets splitted into `self.get_num_pieces_coded_together()` many pieces, which results into these many bytes per piece.
    pub fn get_piece_byte_len(&self) -> usize {
        self.piece_byte_len
    }

    /// Each full coded piece consists of `self.get_num_pieces_coded_together()` random coefficients, appended by corresponding encoded piece of `self.get_piece_byte_len()` bytes.
    pub fn get_full_coded_piece_byte_len(&self) -> usize {
        self.get_num_pieces_coded_together() + self.get_piece_byte_len()
    }

    /// Total number of pieces received by the decoder so far.
    pub fn get_received_piece_count(&self) -> usize {
        self.received_piece_count
    }

    /// Number of useful pieces received by the decoder so far.
    pub fn get_useful_piece_count(&self) -> usize {
        self.useful_piece_count
    }

    /// Number of pieces remaining to be received by the decoder for successful decoding.
    pub fn get_remaining_piece_count(&self) -> usize {
        self.get_num_pieces_coded_together() - self.get_useful_piece_count()
    }

    /// Creates a new `Decoder` instance.
    ///
    /// # Arguments
    /// * `piece_byte_len` - The byte length of each original data piece.
    /// * `required_piece_count` - The minimum number of useful coded pieces
    ///   needed for decoding (equivalent to the number of original pieces).
    ///
    /// # Returns
    /// Returns `Ok(Decoder)` on successful creation.
    /// Returns `Err(RLNCError::PieceLengthZero)` if `piece_byte_len` is zero.
    /// Returns `Err(RLNCError::PieceCountZero)` if `required_piece_count` is zero.
    pub fn new(piece_byte_len: usize, required_piece_count: usize) -> Result<Decoder, RLNCError> {
        if piece_byte_len == 0 {
            return Err(RLNCError::PieceLengthZero);
        }
        if required_piece_count == 0 {
            return Err(RLNCError::PieceCountZero);
        }

        Ok(Decoder {
            matrix: DecoderMatrix::new(required_piece_count, piece_byte_len),
            piece_byte_len,
            required_piece_count,
            received_piece_count: 0,
            useful_piece_count: 0,
        })
    }

    /// Decodes a single full coded piece and adds it to the decoder's matrix.
    ///
    /// Performs Gaussian elimination to reduce the matrix and checks if the
    /// added piece was linearly independent of the existing ones.
    ///
    /// # Arguments
    /// * `full_coded_piece` - A slice containing the coefficients followed by
    ///   the coded data for one piece. Its length must be `required_piece_count + piece_byte_len`.
    ///
    /// # Returns
    /// Returns `Ok(())` if the piece was useful and added successfully.
    /// Returns `Err(RLNCError::ReceivedAllPieces)` if decoding is already complete.
    /// Returns `Err(RLNCError::PieceNotUseful)` if the piece was linearly
    /// dependent on the already received useful pieces.
    /// Returns `Err(RLNCError::InvalidPieceLength)` if the `full_coded_piece` has an unexpected length.
    pub fn decode(&mut self, full_coded_piece: &[u8]) -> Result<(), RLNCError> {
        if self.is_already_decoded() {
            return Err(RLNCError::ReceivedAllPieces);
        }
        if full_coded_piece.len() != self.get_full_coded_piece_byte_len() {
            return Err(RLNCError::InvalidPieceLength);
        }

        let rank_before = self.matrix.rank();

        unsafe { self.matrix.add_row(full_coded_piece).unwrap_unchecked().rref() };
        self.received_piece_count += 1;

        let rank_after = self.matrix.rank();

        // If the rank didn't increase, the piece was not useful.
        if rank_before == rank_after {
            Err(RLNCError::PieceNotUseful)
        } else {
            self.useful_piece_count = rank_after;
            Ok(())
        }
    }

    /// Checks if the decoder has received enough linearly independent pieces
    /// to recover the original data.
    pub fn is_already_decoded(&self) -> bool {
        self.matrix.rank() == self.required_piece_count
    }

    /// Recovers and returns the original data byte vector if decoding is complete.
    ///
    /// Assumes the matrix is in Reduced Row Echelon Form (RREF) and extracts
    /// the original data pieces corresponding to the identity matrix part
    /// of the coefficient matrix. It also handles the boundary marker to
    /// determine the original data length and trims padding.
    ///
    /// # Returns
    /// Returns `Ok(Vec<u8>)` containing the decoded data if successful.
    /// Returns `Err(RLNCError::NotAllPiecesReceivedYet)` if not enough useful
    /// pieces have been received.
    /// Returns `Err(RLNCError::InvalidDecodedDataFormat)` if the extracted data
    /// does not follow the expected format (e.g., boundary marker issues).
    pub fn get_decoded_data(self) -> Result<Vec<u8>, RLNCError> {
        let required_len = self.piece_byte_len * self.required_piece_count;
        let mut buf = vec![0u8; required_len];
        let final_len = self.get_decoded_data_into(&mut buf)?;

        buf.truncate(final_len);
        Ok(buf)
    }

    /// Recovers the original data and writes it into a pre-allocated buffer.
    ///
    /// Returns the final length of the decoded data after removing padding.
    ///
    /// Use this over `get_decoded_data` when you want to avoid heap allocation
    /// and have a pre-allocated buffer ready.
    pub fn get_decoded_data_into(self, out_buf: &mut [u8]) -> Result<usize, RLNCError> {
        if !self.is_already_decoded() {
            return Err(RLNCError::NotAllPiecesReceivedYet);
        }

        let required_len = self.piece_byte_len * self.required_piece_count;
        if out_buf.len() < required_len {
            return Err(RLNCError::InvalidOutputBuffer);
        }

        let mut current_pos = 0;
        let full_coded_piece_len = self.get_full_coded_piece_byte_len();

        // Write the decoded data piece by piece into the output buffer
        for chunk in self.matrix.extract_data().chunks_exact(full_coded_piece_len) {
            let payload = &chunk[self.required_piece_count..];
            let end_pos = current_pos + self.piece_byte_len;
            out_buf[current_pos..end_pos].copy_from_slice(payload);
            current_pos = end_pos;
        }

        let final_len = Self::get_final_data_len(&out_buf[..current_pos])?;

        // Instead of returning a Vec, we return the length of the valid data.
        Ok(final_len)
    }

    /// Helper to find the boundary marker, validate padding,
    /// and return the final length of the original data.
    fn get_final_data_len(padded_data: &[u8]) -> Result<usize, RLNCError> {
        let last_index = padded_data.len().saturating_sub(1);

        let boundary_marker_rev_index = padded_data.iter().rev().position(|&byte| byte == BOUNDARY_MARKER).unwrap_or(last_index);

        let boundary_marker_index = last_index - boundary_marker_rev_index;

        if boundary_marker_index == 0 {
            return Err(RLNCError::InvalidDecodedDataFormat);
        }
        if padded_data[(boundary_marker_index + 1)..].iter().any(|&byte| byte != 0) {
            return Err(RLNCError::InvalidDecodedDataFormat);
        }

        // On success, return the index of the marker, which is the new length.
        Ok(boundary_marker_index)
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoder, RLNCError};
    use crate::full::encoder::Encoder;
    use rand::Rng;

    #[test]
    fn test_decoder_new_invalid_inputs() {
        // Test case 1: piece_byte_len is zero
        let piece_byte_len_zero = 0;
        let required_piece_count_non_zero = 10;

        let result_piece_len_zero = Decoder::new(piece_byte_len_zero, required_piece_count_non_zero);
        assert!(result_piece_len_zero.is_err());
        assert_eq!(result_piece_len_zero.expect_err("Expected PieceLengthZero error"), RLNCError::PieceLengthZero);

        // Test case 2: required_piece_count is zero
        let piece_byte_len_non_zero = 10;
        let required_piece_count_zero = 0;

        let result_piece_count_zero = Decoder::new(piece_byte_len_non_zero, required_piece_count_zero);
        assert!(result_piece_count_zero.is_err());
        assert_eq!(result_piece_count_zero.expect_err("Expected PieceCountZero error"), RLNCError::PieceCountZero);

        // Test case 3: Both piece_byte_len and required_piece_count are zero
        let piece_byte_len_both_zero = 0;
        let required_piece_count_both_zero = 0;

        let result_both_zero = Decoder::new(piece_byte_len_both_zero, required_piece_count_both_zero);
        assert!(result_both_zero.is_err());
        assert_eq!(
            result_both_zero.expect_err("Expected PieceLengthZero error for both zero inputs"),
            RLNCError::PieceLengthZero
        );

        // Test case 4: Valid input
        let piece_byte_len_valid = 10;
        let required_piece_count_valid = 5;
        let result_valid = Decoder::new(piece_byte_len_valid, required_piece_count_valid);
        assert!(result_valid.is_ok());
    }

    #[test]
    fn test_decoder_decode_invalid_piece_length() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize;
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder for decode invalid length test");

        let piece_byte_len = encoder.get_piece_byte_len();
        let required_piece_count = encoder.get_piece_count();
        let full_coded_piece_byte_len = encoder.get_full_coded_piece_byte_len();

        let mut decoder = Decoder::new(piece_byte_len, required_piece_count).expect("Failed to create Decoder for decode invalid length test");

        // Test case 1: Piece length is shorter than expected
        let short_piece_len = full_coded_piece_byte_len - 1;
        let short_coded_piece: Vec<u8> = (0..short_piece_len).map(|_| rng.random()).collect();
        let result_short = decoder.decode(&short_coded_piece);
        assert!(result_short.is_err());
        assert_eq!(
            result_short.expect_err("Expected InvalidPieceLength error for short piece"),
            RLNCError::InvalidPieceLength
        );

        // Test case 2: Piece length is longer than expected
        let long_piece_len = full_coded_piece_byte_len + 1;
        let long_coded_piece: Vec<u8> = (0..long_piece_len).map(|_| rng.random()).collect();
        let result_long = decoder.decode(&long_coded_piece);
        assert!(result_long.is_err());
        assert_eq!(
            result_long.expect_err("Expected InvalidPieceLength error for long piece"),
            RLNCError::InvalidPieceLength
        );

        // Test case 3: Piece length is zero
        let zero_piece: Vec<u8> = Vec::new();
        let result_zero = decoder.decode(&zero_piece);
        assert!(result_zero.is_err());
        assert_eq!(
            result_zero.expect_err("Expected InvalidPieceLength error for zero-length piece"),
            RLNCError::InvalidPieceLength
        );

        // Ensure decoder state is unchanged after invalid decode attempts
        assert_eq!(decoder.get_received_piece_count(), 0);
        assert_eq!(decoder.get_useful_piece_count(), 0);
        assert!(!decoder.is_already_decoded());

        // Test case 4: Valid coded piece - check if state changes
        let mut coded_piece = vec![0u8; full_coded_piece_byte_len];
        encoder.code(&mut rng, &mut coded_piece);
        let result_correct = decoder.decode(&coded_piece);
        assert!(result_correct.is_ok() || matches!(result_correct, Err(RLNCError::PieceNotUseful)));

        // After a valid decode attempt, received_piece_count must increase
        assert_eq!(decoder.get_received_piece_count(), 1);

        // If the piece was useful, useful_piece_count will be 1. Otherwise, it remains 0.
        // Given the small piece_count in this test, it's very likely to be useful.
        if result_correct.is_ok() {
            assert_eq!(decoder.get_useful_piece_count(), 1);
            assert!(!decoder.is_already_decoded()); // Unless piece_count was 1
        } else {
            assert_eq!(decoder.get_useful_piece_count(), 0);
        }
    }

    #[test]
    fn test_decoder_getters() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize;
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data.clone(), piece_count).expect("Failed to create Encoder for getters test");

        let piece_byte_len = encoder.get_piece_byte_len();
        let required_piece_count = encoder.get_piece_count();
        let full_coded_piece_byte_len = encoder.get_full_coded_piece_byte_len();

        let mut decoder = Decoder::new(piece_byte_len, required_piece_count).expect("Failed to create Decoder for getters test");

        assert_eq!(decoder.get_num_pieces_coded_together(), required_piece_count);
        assert_eq!(decoder.get_piece_byte_len(), piece_byte_len);
        assert_eq!(decoder.get_full_coded_piece_byte_len(), full_coded_piece_byte_len);
        assert_eq!(decoder.get_received_piece_count(), 0);
        assert_eq!(decoder.get_useful_piece_count(), 0);
        assert_eq!(decoder.get_remaining_piece_count(), required_piece_count);
        assert!(!decoder.is_already_decoded());

        // Add some pieces and track useful ones
        let num_pieces_to_decode_initially = required_piece_count / 2;
        let mut expected_useful_pieces_after_initial = 0;
        let mut coded_piece = vec![0u8; full_coded_piece_byte_len];

        for _ in 0..num_pieces_to_decode_initially {
            encoder.code(&mut rng, &mut coded_piece);
            match decoder.decode(&coded_piece) {
                Ok(_) => {
                    expected_useful_pieces_after_initial += 1;
                }
                Err(RLNCError::PieceNotUseful) => {}
                Err(e) => panic!("Unexpected error during initial decoding phase: {e:?}"),
            }
        }

        assert_eq!(decoder.get_received_piece_count(), num_pieces_to_decode_initially);
        assert_eq!(decoder.get_useful_piece_count(), expected_useful_pieces_after_initial);
        assert_eq!(decoder.get_remaining_piece_count(), required_piece_count - expected_useful_pieces_after_initial);

        // Add remaining pieces to complete decoding
        let mut total_pieces_received = num_pieces_to_decode_initially;
        let mut coded_piece = vec![0u8; full_coded_piece_byte_len];
        while !decoder.is_already_decoded() {
            encoder.code(&mut rng, &mut coded_piece);

            match decoder.decode(&coded_piece) {
                Ok(_) => {}
                Err(RLNCError::PieceNotUseful) => {}
                Err(RLNCError::ReceivedAllPieces) => break,
                Err(e) => panic!("Unexpected error during final decoding phase: {e:?}"),
            }

            total_pieces_received += 1;
        }

        assert_eq!(decoder.get_useful_piece_count(), required_piece_count);
        assert_eq!(decoder.get_remaining_piece_count(), 0);
        assert!(decoder.is_already_decoded());
        assert_eq!(decoder.get_received_piece_count(), total_pieces_received);
    }
}
