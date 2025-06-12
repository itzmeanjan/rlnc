use super::consts::BOUNDARY_MARKER;
use crate::{RLNCError, common::gf256::Gf256};

/// Random Linear Network Code (RLNC) Decoder.
///
/// This struct manages the received coded pieces and performs Gaussian
/// elimination to recover the original data.
#[derive(Clone)]
pub struct Decoder {
    /// Stores the coefficient matrix and coded data rows concatenated.
    /// Each row is a coded piece: `[coefficients | data_piece]`.
    data: Vec<u8>,
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
    /// Creates a new `Decoder` instance.
    ///
    /// # Arguments
    /// * `piece_byte_len` - The byte length of each original data piece.
    /// * `required_piece_count` - The minimum number of useful coded pieces
    ///   needed for decoding (equivalent to the number of original pieces).
    pub fn new(piece_byte_len: usize, required_piece_count: usize) -> Decoder {
        let full_coded_piece_byte_len = required_piece_count + piece_byte_len;
        let total_byte_len = required_piece_count * full_coded_piece_byte_len;
        let data = Vec::with_capacity(total_byte_len);

        Decoder {
            data,
            piece_byte_len,
            required_piece_count,
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
    /// Returns `Ok(())` if the piece was useful and added successfully.
    /// Returns `Err(RLNCError::ReceivedAllPieces)` if decoding is already complete.
    /// Returns `Err(RLNCError::PieceNotUseful)` if the piece was linearly
    /// dependent on the already received useful pieces.
    pub fn decode(&mut self, full_coded_piece: &[u8]) -> Result<(), RLNCError> {
        if self.is_already_decoded() {
            return Err(RLNCError::ReceivedAllPieces);
        }

        let rank_before = self.rank();

        self.data.extend_from_slice(full_coded_piece);
        self.received_piece_count += 1;
        self.useful_piece_count += 1;
        self.rref(); // Perform Gaussian elimination.

        let rank_after = self.rank();

        // If the rank didn't increase, the piece was not useful.
        if rank_before == rank_after {
            // The `rref` call will have already removed the zero row that resulted
            // from adding this linearly dependent piece, so `useful_piece_count`
            // is already back to `rank_before`.
            return Err(RLNCError::PieceNotUseful);
        }

        Ok(())
    }

    /// Checks if the decoder has received enough linearly independent pieces
    /// to recover the original data.
    pub fn is_already_decoded(&self) -> bool {
        self.rank() == self.required_piece_count
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
        if !self.is_already_decoded() {
            return Err(RLNCError::NotAllPiecesReceivedYet);
        }

        let full_coded_piece_len = self.required_piece_count + self.piece_byte_len;
        let mut decoded_data = Vec::with_capacity(self.piece_byte_len * self.required_piece_count);

        // Iterate over the useful rows (which should be the decoded original pieces)
        // and extract the data part from each row.
        self.data.chunks_exact(full_coded_piece_len).for_each(|full_decoded_piece| {
            // The data part of the row starts after the coefficient columns.
            let decoded_piece = &full_decoded_piece[self.required_piece_count..];
            decoded_data.extend_from_slice(decoded_piece);
        });

        // Find the boundary marker to trim padding.
        let last_index_of_decoded_data = decoded_data.len() - 1;
        let boundary_marker_rev_index = decoded_data
            .iter()
            .rev()
            .position(|&byte| byte == BOUNDARY_MARKER)
            .unwrap_or(last_index_of_decoded_data);
        let boundary_marker_index = last_index_of_decoded_data - boundary_marker_rev_index;

        if boundary_marker_index == 0 {
            return Err(RLNCError::InvalidDecodedDataFormat);
        }
        if decoded_data[(boundary_marker_index + 1)..].iter().any(|&byte| byte != 0) {
            return Err(RLNCError::InvalidDecodedDataFormat);
        }

        decoded_data.truncate(boundary_marker_index);
        Ok(decoded_data)
    }

    /// Gets a byte from the decoder's matrix at the specified row and column,
    /// converting it to a `Gf256` element.
    ///
    /// # Arguments
    /// * `index` - A tuple `(row_index, col_index)` specifying the position.
    ///
    /// # Returns
    /// Returns the element as a `Gf256`.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    fn get(&self, index: (usize, usize)) -> Gf256 {
        let (r_index, c_index) = index;
        let cols = self.required_piece_count + self.piece_byte_len;

        Gf256::new(self.data[r_index * cols + c_index])
    }

    /// Sets a byte in the decoder's matrix at the specified row and column
    /// from a `Gf256` element.
    ///
    /// # Arguments
    /// * `index` - A tuple `(row_index, col_index)` specifying the position.
    /// * `val` - The `Gf256` value to set.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    fn set(&mut self, index: (usize, usize), val: Gf256) {
        let (r_index, c_index) = index;
        let cols = self.required_piece_count + self.piece_byte_len;

        self.data[r_index * cols + c_index] = val.get();
    }

    /// Swaps two rows in the decoder's matrix.
    ///
    /// # Arguments
    /// * `row1` - The index of the first row.
    /// * `row2` - The index of the second row.
    ///
    /// # Panics
    /// Panics if either row index is out of bounds for the current number of useful rows.
    fn swap_rows(&mut self, row1: usize, row2: usize) {
        let cols = self.required_piece_count + self.piece_byte_len;

        let row1_begins_at = row1 * cols;
        let row2_begins_at = row2 * cols;

        // Swap each element in the two rows.
        (0..cols).for_each(|cidx| {
            self.data.swap(row1_begins_at + cidx, row2_begins_at + cidx);
        });
    }

    /// Performs the forward phase of Gaussian elimination (to row echelon form).
    ///
    /// Pivots are selected, rows are swapped if necessary to get a non-zero
    /// pivot, and rows below the pivot are cleared by subtracting a multiple
    /// of the pivot row.
    fn clean_forward(&mut self) {
        let rows = self.useful_piece_count;
        let cols = self.required_piece_count + self.piece_byte_len;
        let boundary = rows.min(cols);

        for i in 0..boundary {
            if self.get((i, i)) == Gf256::zero() {
                let mut is_non_zero_col = false;
                let mut pivot_row_idx = i + 1;

                while pivot_row_idx < rows {
                    if self.get((pivot_row_idx, i)) != Gf256::zero() {
                        is_non_zero_col = true;
                        break;
                    }
                    pivot_row_idx += 1;
                }

                if !is_non_zero_col {
                    continue;
                }

                self.swap_rows(i, pivot_row_idx);
            }

            for j in (i + 1)..rows {
                if self.get((j, i)) == Gf256::zero() {
                    continue;
                }

                let quotient = (self.get((j, i)) / self.get((i, i))).unwrap();
                for k in i..cols {
                    self.set((j, k), self.get((j, k)) + self.get((i, k)) * quotient);
                }
            }
        }
    }

    /// Performs the backward phase of Gaussian elimination (to reduced row echelon form).
    ///
    /// Clears entries above the pivots and normalizes pivots to 1.
    fn clean_backward(&mut self) {
        let rows = self.useful_piece_count;
        let cols = self.required_piece_count + self.piece_byte_len;
        let boundary = rows.min(cols);

        for i in (0..boundary).rev() {
            if self.get((i, i)) == Gf256::zero() {
                continue;
            }

            for j in 0..i {
                if self.get((j, i)) == Gf256::zero() {
                    continue;
                }

                let quotient = (self.get((j, i)) / self.get((i, i))).unwrap();
                for k in i..cols {
                    self.set((j, k), self.get((j, k)) + self.get((i, k)) * quotient);
                }
            }

            if self.get((i, i)) == Gf256::one() {
                continue;
            }

            let inv = self.get((i, i)).inv().unwrap();
            self.set((i, i), Gf256::one());

            for j in (i + 1)..cols {
                if self.get((i, j)) == Gf256::zero() {
                    continue;
                }
                self.set((i, j), self.get((i, j)) * inv);
            }
        }
    }

    /// Removes zero rows from the matrix and updates `useful_piece_count`.
    ///
    /// A row is considered a zero row if all its coefficient columns are zero.
    /// This step is crucial after RREF to determine the true rank and compact
    /// the matrix to only the useful rows.
    fn remove_zero_rows(&mut self) {
        let mut rows = self.useful_piece_count;
        let cols = self.required_piece_count + self.piece_byte_len;
        let coeff_cols = self.required_piece_count;

        let mut i = 0;
        while i < rows {
            let is_nonzero_row = (0..coeff_cols).any(|cidx| (self.get((i, cidx)) != Gf256::zero()));
            if is_nonzero_row {
                i += 1;
                continue;
            }

            let start_index_of_row_to_remove = i * cols;
            let start_index_of_next_row = (i + 1) * cols;
            let end_index_of_useful_data = self.useful_piece_count * cols;

            if start_index_of_next_row < end_index_of_useful_data {
                self.data
                    .copy_within(start_index_of_next_row..end_index_of_useful_data, start_index_of_row_to_remove);
            }

            rows -= 1;
        }

        self.useful_piece_count = rows;

        let total_byte_len = rows * cols;
        self.data.truncate(total_byte_len);
    }

    /// Computes the Reduced Row Echelon Form (RREF) of the matrix.
    ///
    /// This involves forward elimination (`clean_forward`), backward elimination
    /// (`clean_backward`), and removing any resulting zero rows (`remove_zero_rows`).
    /// The `useful_piece_count` is updated to reflect the rank of the matrix.
    fn rref(&mut self) {
        self.clean_forward();
        self.clean_backward();
        self.remove_zero_rows();
    }

    /// Returns the current rank of the matrix, which is the number of
    /// linearly independent (useful) pieces received so far.
    fn rank(&self) -> usize {
        self.useful_piece_count
    }
}
