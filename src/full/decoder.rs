use super::consts::BOUNDARY_MARKER;
use super::types::{PieceByteLen, PieceCount};
use crate::{RLNCError, full::decoder_matrix::DecoderMatrix};

/// Random Linear Network Coding (RLNC) Decoder.
///
/// This struct manages the received coded pieces and repeatedly performs Gaussian elimination
/// to recover the original data, validating padding.
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
    /// Number of pieces original data got split into and coded together.
    pub fn piece_count(&self) -> PieceCount {
        unsafe { PieceCount::new(self.required_piece_count).unwrap_unchecked() }
    }

    /// After padding the original data, it gets split into `self.piece_count()` many pieces, which results into these many bytes per piece.
    pub fn piece_byte_len(&self) -> PieceByteLen {
        unsafe { PieceByteLen::new(self.piece_byte_len).unwrap_unchecked() }
    }

    /// Each full coded piece consists of `self.piece_count()` random coefficients, appended by corresponding encoded piece of `self.piece_byte_len()` bytes.
    pub fn full_coded_piece_byte_len(&self) -> usize {
        self.required_piece_count + self.piece_byte_len
    }

    /// Total number of pieces received by the decoder so far.
    pub fn received_piece_count(&self) -> usize {
        self.received_piece_count
    }

    /// Number of useful pieces received by the decoder so far.
    pub fn useful_piece_count(&self) -> usize {
        self.useful_piece_count
    }

    /// Number of pieces remaining to be received by the decoder for successful decoding.
    pub fn remaining_piece_count(&self) -> usize {
        self.required_piece_count - self.useful_piece_count
    }

    /// Creates a new `Decoder` instance.
    ///
    /// Because `PieceByteLen` and `PieceCount` are guaranteed to be non-zero by
    /// construction, this function is infallible.
    ///
    /// # Arguments
    /// * `piece_byte_len` - The byte length of each original data piece.
    /// * `piece_count` - The minimum number of useful coded pieces
    ///   needed for decoding (equivalent to the number of original pieces).
    pub fn new(piece_byte_len: PieceByteLen, piece_count: PieceCount) -> Decoder {
        let pbl = piece_byte_len.value();
        let pc = piece_count.value();

        Decoder {
            matrix: DecoderMatrix::new(pc, pbl),
            piece_byte_len: pbl,
            required_piece_count: pc,
            received_piece_count: 0,
            useful_piece_count: 0,
        }
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
    /// * Returns `Ok(())` if the piece was useful and added successfully.
    /// * Returns `Err(RLNCError::ReceivedAllPieces)` if decoding is already complete.
    /// * Returns `Err(RLNCError::PieceNotUseful)` if the piece was linearly dependent on the already received useful pieces.
    /// * Returns `Err(RLNCError::InvalidPieceLength)` if the `full_coded_piece` has an unexpected length.
    pub fn decode(&mut self, full_coded_piece: &[u8]) -> Result<(), RLNCError> {
        if self.is_already_decoded() {
            return Err(RLNCError::ReceivedAllPieces);
        }
        if full_coded_piece.len() != self.full_coded_piece_byte_len() {
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

    /// Checks if the decoder has received enough linearly independent pieces to recover the original data.
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
    /// * Returns `Ok(Vec<u8>)` containing the decoded data if successful.
    /// * Returns `Err(RLNCError::NotAllPiecesReceivedYet)` if not enough useful pieces have been received.
    /// * Returns `Err(RLNCError::InvalidDecodedDataFormat)` if the extracted data does not follow the expected format (e.g., boundary marker issues).
    pub fn into_decoded_data(self) -> Result<Vec<u8>, RLNCError> {
        if !self.is_already_decoded() {
            return Err(RLNCError::NotAllPiecesReceivedYet);
        }

        let required_len = self.piece_byte_len * self.required_piece_count;
        let mut buf = vec![0u8; required_len];

        let mut current_pos = 0;
        let full_coded_piece_len = self.full_coded_piece_byte_len();

        // Write the decoded data piece by piece into the output buffer
        for chunk in self.matrix.extract_data().chunks_exact(full_coded_piece_len) {
            let payload = &chunk[self.required_piece_count..];
            let end_pos = current_pos + self.piece_byte_len;
            buf[current_pos..end_pos].copy_from_slice(payload);
            current_pos = end_pos;
        }

        let final_len = Self::get_final_data_len(&buf[..current_pos])?;

        buf.truncate(final_len);
        Ok(buf)
    }

    /// Helper to find the boundary marker, validate padding, and return the final length of the original data.
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
    use super::{Decoder, PieceByteLen, PieceCount, RLNCError};
    use crate::full::encoder::Encoder;
    use rand::Rng;

    #[test]
    fn test_decoder_new_type_safety() {
        // PieceByteLen and PieceCount reject zero at the type level
        assert!(PieceByteLen::new(0).is_none());
        assert!(PieceCount::new(0).is_none());

        // Valid input always succeeds (infallible constructor)
        let piece_byte_len = PieceByteLen::new(10).unwrap();
        let piece_count = PieceCount::new(5).unwrap();
        let decoder = Decoder::new(piece_byte_len, piece_count);
        assert_eq!(decoder.piece_count(), piece_count);
        assert_eq!(decoder.piece_byte_len(), piece_byte_len);
    }

    #[test]
    fn test_decoder_decode_invalid_piece_length() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize;
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder for decode invalid length test");

        let mut decoder = Decoder::new(encoder.piece_byte_len(), encoder.piece_count());

        // Test case 1: Piece length is shorter than expected
        let short_piece_len = encoder.full_coded_piece_byte_len() - 1;
        let short_coded_piece: Vec<u8> = (0..short_piece_len).map(|_| rng.random()).collect();
        let result_short = decoder.decode(&short_coded_piece);
        assert!(result_short.is_err());
        assert_eq!(
            result_short.expect_err("Expected InvalidPieceLength error for short piece"),
            RLNCError::InvalidPieceLength
        );

        // Test case 2: Piece length is longer than expected
        let long_piece_len = encoder.full_coded_piece_byte_len() + 1;
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
        assert_eq!(decoder.received_piece_count(), 0);
        assert_eq!(decoder.useful_piece_count(), 0);
        assert!(!decoder.is_already_decoded());

        // Test case 4: Valid coded piece - check if state changes
        let correct_coded_piece = encoder.code(&mut rng);
        let result_correct = decoder.decode(&correct_coded_piece);
        assert!(result_correct.is_ok() || matches!(result_correct, Err(RLNCError::PieceNotUseful)));

        // After a valid decode attempt, received_piece_count must increase
        assert_eq!(decoder.received_piece_count(), 1);

        // If the piece was useful, useful_piece_count will be 1. Otherwise, it remains 0.
        // Given the small piece_count in this test, it's very likely to be useful.
        if result_correct.is_ok() {
            assert_eq!(decoder.useful_piece_count(), 1);
            assert!(!decoder.is_already_decoded()); // Unless piece_count was 1
        } else {
            assert_eq!(decoder.useful_piece_count(), 0);
        }
    }

    #[test]
    fn test_decoder_getters() {
        let mut rng = rand::rng();

        let data_byte_len = 1024usize;
        let piece_count = 32usize;
        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data.clone(), piece_count).expect("Failed to create Encoder for getters test");

        let mut decoder = Decoder::new(encoder.piece_byte_len(), encoder.piece_count());

        assert_eq!(decoder.piece_count(), encoder.piece_count());
        assert_eq!(decoder.piece_byte_len(), encoder.piece_byte_len());
        assert_eq!(decoder.full_coded_piece_byte_len(), encoder.full_coded_piece_byte_len());
        assert_eq!(decoder.received_piece_count(), 0);
        assert_eq!(decoder.useful_piece_count(), 0);
        assert_eq!(decoder.remaining_piece_count(), encoder.piece_count().value());
        assert!(!decoder.is_already_decoded());

        // Add some pieces and track useful ones
        let num_pieces_to_decode_initially = encoder.piece_count().value() / 2;
        let mut expected_useful_pieces_after_initial = 0;

        for _ in 0..num_pieces_to_decode_initially {
            let coded_piece = encoder.code(&mut rng);
            match decoder.decode(&coded_piece) {
                Ok(_) => {
                    expected_useful_pieces_after_initial += 1;
                }
                Err(RLNCError::PieceNotUseful) => {}
                Err(e) => panic!("Unexpected error during initial decoding phase: {e:?}"),
            }
        }

        assert_eq!(decoder.received_piece_count(), num_pieces_to_decode_initially);
        assert_eq!(decoder.useful_piece_count(), expected_useful_pieces_after_initial);
        assert_eq!(decoder.remaining_piece_count(), encoder.piece_count().value() - expected_useful_pieces_after_initial);

        // Add remaining pieces to complete decoding
        let mut total_pieces_received = num_pieces_to_decode_initially;
        while !decoder.is_already_decoded() {
            let coded_piece = encoder.code(&mut rng);

            match decoder.decode(&coded_piece) {
                Ok(_) => {}
                Err(RLNCError::PieceNotUseful) => {}
                Err(RLNCError::ReceivedAllPieces) => break,
                Err(e) => panic!("Unexpected error during final decoding phase: {e:?}"),
            }

            total_pieces_received += 1;
        }

        assert_eq!(decoder.useful_piece_count(), encoder.piece_count().value());
        assert_eq!(decoder.remaining_piece_count(), 0);
        assert!(decoder.is_already_decoded());
        assert_eq!(decoder.received_piece_count(), total_pieces_received);
    }
}
