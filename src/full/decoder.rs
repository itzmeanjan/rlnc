use super::consts::BOUNDARY_MARKER;
use crate::{RLNCError, common::gf256::Gf256};

#[derive(Clone)]
pub struct Decoder {
    data: Vec<u8>,
    piece_byte_len: usize,
    required_piece_count: usize,
    received_piece_count: usize,
    useful_piece_count: usize,
}

impl Decoder {
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

    pub fn decode(&mut self, full_coded_piece: &[u8]) -> Result<(), RLNCError> {
        if self.is_already_decoded() {
            return Err(RLNCError::ReceivedAllPieces);
        }

        let rank_before = self.rank();

        self.data.extend_from_slice(full_coded_piece);
        self.received_piece_count += 1;
        self.useful_piece_count += 1;
        self.rref();

        let rank_after = self.rank();

        if rank_before == rank_after {
            return Err(RLNCError::PieceNotUseful);
        }

        Ok(())
    }

    pub fn is_already_decoded(&self) -> bool {
        self.rank() == self.required_piece_count
    }

    pub fn get_decoded_data(self) -> Result<Vec<u8>, RLNCError> {
        if !self.is_already_decoded() {
            return Err(RLNCError::NotAllPiecesReceivedYet);
        }

        let full_coded_piece_len = self.required_piece_count + self.piece_byte_len;
        let mut decoded_data = Vec::with_capacity(self.piece_byte_len * self.required_piece_count);

        self.data.chunks_exact(full_coded_piece_len).for_each(|full_decoded_piece| {
            let decoded_piece = &full_decoded_piece[self.required_piece_count..];
            decoded_data.extend_from_slice(decoded_piece);
        });

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

    fn get(&self, index: (usize, usize)) -> Gf256 {
        let (r_index, c_index) = index;
        let cols = self.required_piece_count + self.piece_byte_len;

        Gf256::new(self.data[r_index * cols + c_index])
    }

    fn set(&mut self, index: (usize, usize), val: Gf256) {
        let (r_index, c_index) = index;
        let cols = self.required_piece_count + self.piece_byte_len;

        self.data[r_index * cols + c_index] = val.get();
    }

    fn swap_rows(&mut self, row1: usize, row2: usize) {
        let cols = self.required_piece_count + self.piece_byte_len;

        let row1_begins_at = row1 * cols;
        let row2_begins_at = row2 * cols;

        (0..cols).for_each(|cidx| {
            self.data.swap(row1_begins_at + cidx, row2_begins_at + cidx);
        });
    }

    fn clean_forward(&mut self) {
        let rows = self.useful_piece_count;
        let cols = self.required_piece_count + self.piece_byte_len;
        let boundary = rows.min(cols);

        for i in 0..boundary {
            if self.get((i, i)) == Gf256::zero() {
                let mut is_non_zero_col = false;
                let mut pivot = i + 1;

                while pivot < rows {
                    if self.get((pivot, i)) != Gf256::zero() {
                        is_non_zero_col = true;
                        break;
                    }

                    pivot += 1;
                }

                if !is_non_zero_col {
                    continue;
                }

                self.swap_rows(i, pivot);
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

    fn rref(&mut self) {
        self.clean_forward();
        self.clean_backward();
        self.remove_zero_rows();
    }

    fn rank(&self) -> usize {
        self.useful_piece_count
    }
}
