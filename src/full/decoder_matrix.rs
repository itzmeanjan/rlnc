use crate::{
    RLNCError,
    common::{
        gf256::Gf256,
        simd::{gf256_inplace_mul_vec_by_scalar, gf256_mul_vec_by_scalar_then_add_into_vec},
    },
};
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, PartialEq)]
pub struct DecoderMatrix {
    num_pieces_coded_together: usize,
    rows: usize,
    cols: usize,
    elements: Vec<u8>,
}

impl DecoderMatrix {
    /// Given RLNC encoding configuration, it sets up a decoder matrix.
    ///
    /// This decoder matrix can be used to add incoming erasure-coded pieces,
    /// and incrementally decode them using Gaussian Elimination, if it's a
    /// useful (i.e. linearly independent) piece.
    ///
    /// # Arguments
    /// * `num_pieces_coded_together` - The minimum number of useful coded pieces needed for decoding.
    /// * `piece_byte_length` - The byte length of each original data piece.
    ///
    /// # Returns
    /// An instance of decoder matrix - ready to use for decoding.
    pub fn new(num_pieces_coded_together: usize, piece_byte_length: usize) -> Self {
        let full_coded_piece_byte_len = num_pieces_coded_together + piece_byte_length;
        let total_byte_len = num_pieces_coded_together * full_coded_piece_byte_len;
        let elements = Vec::with_capacity(total_byte_len);

        Self {
            num_pieces_coded_together,
            rows: 0,
            cols: full_coded_piece_byte_len,
            elements,
        }
    }

    /// Adds a new row to the decoder matrix.
    ///
    /// # Arguments
    /// `row` - A byte slice, representing a full erasure-coded piece i.e. containing the coefficients followed by
    ///  the coded data for one piece. Its length must be `num_pieces_coded_together + piece_byte_length`.
    ///
    /// # Returns
    /// * Ok(&mut Self) - If full erasure-coded piece is of valid length.
    /// * Err(RLNCError::InvalidPieceLength) - If full erasure-coded piece length doesn't match expected value.
    pub fn add_row(&mut self, row: &[u8]) -> Result<&mut Self, RLNCError> {
        if row.len() != self.cols {
            return Err(RLNCError::InvalidPieceLength);
        }

        self.elements.extend_from_slice(row);
        self.rows += 1;

        Ok(self)
    }

    /// Swaps two rows in the decoder's matrix.
    ///
    /// # Arguments
    /// * `row1_idx` - The index of the first row.
    /// * `row2_idx` - The index of the second row.
    pub fn swap_rows(&mut self, row1_idx: usize, row2_idx: usize) -> &mut Self {
        if row1_idx == row2_idx {
            return self;
        }

        let (r1, r2) = if row1_idx < row2_idx { (row1_idx, row2_idx) } else { (row2_idx, row1_idx) };

        let start1 = r1 * self.cols;
        let end1 = start1 + self.cols;
        let start2 = r2 * self.cols;

        let (left, right) = self.elements.split_at_mut(start2);

        // row1 is in the first part
        let row1 = &mut left[start1..end1];
        // row2 is the beginning of the second part
        let row2 = &mut right[..self.cols];

        row1.swap_with_slice(row2);

        self
    }

    /// Computes the Reduced Row Echelon Form (RREF) of the matrix.
    ///
    /// This involves forward elimination (`Self::clean_forward`), backward elimination
    /// (`Self::clean_backward`), and removing any resulting zero rows (`Self::remove_zero_rows`).
    ///
    /// This function updates the number of rows to reflect the current rank of the matrix.
    /// It is safe to call `Self::rank` after calling this function.
    pub fn rref(&mut self) -> &mut Self {
        self.clean_forward().clean_backward().remove_zero_rows()
    }

    /// Returns the current rank of the matrix, which is same as the number
    /// of rows, after calling `Self::rref`.
    pub fn rank(&self) -> usize {
        self.rows
    }

    /// Returns underlying data i.e. `self.rows` many full erasure-coded pieces.
    /// Calling this function, consumes the decoder matrix instance.
    pub fn extract_data(self) -> Vec<u8> {
        self.elements
    }

    /// Performs the forward phase of Gaussian elimination (to row echelon form).
    ///
    /// Pivots are selected, rows are swapped if necessary to get a non-zero
    /// pivot, and rows below the pivot are cleared by subtracting a multiple
    /// of the pivot row.
    fn clean_forward(&mut self) -> &mut Self {
        let boundary = self.rows.min(self.cols);

        for i in 0..boundary {
            if self[(i, i)] == Gf256::zero() {
                let mut is_non_zero_col = false;
                let mut pivot_row_idx = i + 1;

                while pivot_row_idx < self.rows {
                    if self[(pivot_row_idx, i)] != Gf256::zero() {
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

            for j in (i + 1)..self.rows {
                if self[(j, i)] == Gf256::zero() {
                    continue;
                }

                let quotient = unsafe { (self[(j, i)] / self[(i, i)]).unwrap_unchecked().get() };

                let i_th_row_starts_at = i * self.cols;
                let i_th_row_ends_at = i_th_row_starts_at + self.cols;

                let j_th_row_starts_at = j * self.cols;
                let j_th_row_ends_at = j_th_row_starts_at + self.cols;

                let (left, right) = self.elements.split_at_mut(i_th_row_ends_at);

                let i_th_row = &left[(i_th_row_starts_at + i)..];
                let j_th_row = &mut right[(j_th_row_starts_at - i_th_row_ends_at + i)..(j_th_row_ends_at - i_th_row_ends_at)];

                gf256_mul_vec_by_scalar_then_add_into_vec(j_th_row, i_th_row, quotient);
            }
        }

        self
    }

    /// Performs the backward phase of Gaussian elimination (to reduced row echelon form).
    ///
    /// Clears entries above the pivots and normalizes pivots to 1.
    fn clean_backward(&mut self) -> &mut Self {
        let boundary = self.rows.min(self.cols);

        for i in (0..boundary).rev() {
            if self[(i, i)] == Gf256::zero() {
                continue;
            }

            for j in 0..i {
                if self[(j, i)] == Gf256::zero() {
                    continue;
                }

                let quotient = unsafe { (self[(j, i)] / self[(i, i)]).unwrap_unchecked().get() };

                let j_th_row_starts_at = j * self.cols;
                let j_th_row_ends_at = j_th_row_starts_at + self.cols;

                let i_th_row_starts_at = i * self.cols;
                let i_th_row_ends_at = i_th_row_starts_at + self.cols;

                let (left, right) = self.elements.split_at_mut(j_th_row_ends_at);

                let j_th_row = &mut left[(j_th_row_starts_at + i)..];
                let i_th_row = &right[(i_th_row_starts_at - j_th_row_ends_at + i)..(i_th_row_ends_at - j_th_row_ends_at)];

                gf256_mul_vec_by_scalar_then_add_into_vec(j_th_row, i_th_row, quotient);
            }

            if self[(i, i)] == Gf256::one() {
                continue;
            }

            let inv = unsafe { self[(i, i)].inv().unwrap_unchecked().get() };
            self[(i, i)] = Gf256::one();

            let i_th_row_starts_at = i * self.cols;
            let i_th_row_ends_at = i_th_row_starts_at + self.cols;

            let i_th_row = &mut self.elements[(i_th_row_starts_at + (i + 1))..i_th_row_ends_at];
            gf256_inplace_mul_vec_by_scalar(i_th_row, inv);
        }

        self
    }

    /// Removes zero rows from the matrix and updates `useful_piece_count`.
    ///
    /// A row is considered a zero row if all its coefficient columns are zero.
    /// This step is crucial after RREF to determine the true rank and compact
    /// the matrix to only the useful rows.
    fn remove_zero_rows(&mut self) -> &mut Self {
        let mut i = 0;
        while i < self.rows {
            let is_nonzero_row = (0..self.num_pieces_coded_together).any(|cidx| self[(i, cidx)] != Gf256::zero());
            if is_nonzero_row {
                i += 1;
                continue;
            }

            let start_idx_of_row_to_remove = i * self.cols;
            let start_idx_of_next_row = (i + 1) * self.cols;

            if start_idx_of_next_row < self.elements.len() {
                self.elements.copy_within(start_idx_of_next_row.., start_idx_of_row_to_remove);
            }
            self.rows -= 1;
        }

        let updated_num_elements = self.rows * self.cols;
        self.elements.truncate(updated_num_elements);

        self
    }
}

impl Index<(usize, usize)> for DecoderMatrix {
    type Output = Gf256;

    /// Returns an immutable reference to an element of matrix at the specified row and column,
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
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (row_idx, col_idx) = index;
        let lin_idx = row_idx * self.cols + col_idx;

        unsafe { std::mem::transmute(self.elements.get_unchecked(lin_idx)) }
    }
}

impl IndexMut<(usize, usize)> for DecoderMatrix {
    /// Returns a mutable reference to an element of matrix at the specified row and column,
    /// converting it to a `Gf256` element.
    ///
    /// # Arguments
    /// * `index` - A tuple `(row_index, col_index)` specifying the position.
    /// * `val` - The `Gf256` value to set.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (row_idx, col_idx) = index;
        let lin_idx = row_idx * self.cols + col_idx;

        unsafe { std::mem::transmute(self.elements.get_unchecked_mut(lin_idx)) }
    }
}

#[cfg(test)]
mod test {
    use crate::full::decoder_matrix::DecoderMatrix;
    use rand::Rng;

    fn make_random_matrix<R: Rng + ?Sized>(num_rows: usize, num_cols: usize, rng: &mut R) -> DecoderMatrix {
        let mut matrix = DecoderMatrix::new(num_cols, 0);

        (0..num_rows).for_each(|_| {
            let random_row = (0..num_cols).map(|_| rng.random()).collect::<Vec<u8>>();
            matrix.add_row(&random_row).expect("adding new must not fail");
        });

        matrix
    }

    #[test]
    fn prop_test_rref_is_idempotent() {
        const NUM_TEST_ITERATIONS: usize = 1000;

        const MIN_NUM_ROWS: usize = 1;
        const MAX_NUM_ROWS: usize = 1000;

        const MIN_NUM_COLS: usize = 1;
        const MAX_NUM_COLS: usize = 1000;

        let mut rng = rand::rng();

        (0..NUM_TEST_ITERATIONS).for_each(|_| {
            let num_rows = rng.random_range(MIN_NUM_ROWS..=MAX_NUM_ROWS);
            let num_cols = rng.random_range(MIN_NUM_COLS..=MAX_NUM_COLS);

            let mut matrix = make_random_matrix(num_rows, num_cols, &mut rng);
            let rrefed = matrix.rref().clone().rref().to_owned();

            assert_eq!(matrix, rrefed);
        });
    }
}
