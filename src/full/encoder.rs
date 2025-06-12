use super::consts::BOUNDARY_MARKER;
use crate::{RLNCError, common::gf256::Gf256};
use rand::Rng;

/// Represents an RLNC encoder, responsible for dividing data into pieces and
/// generating coded pieces based on random sampled coding vectors.
#[derive(Clone)]
pub struct Encoder {
    data: Vec<u8>,
    piece_count: usize,
}

impl Encoder {
    /// Creates a new `Encoder` without adding any padding to the input data.
    /// This is suitable if the input data length is already a multiple of the
    /// desired piece count.
    ///
    /// Returns the `Encoder` and the calculated byte length of each piece.
    /// Returns `RLNCError::DataLengthMismatch` if the data length is not a
    /// multiple of the piece count.
    pub(crate) fn without_padding(data: Vec<u8>, piece_count: usize) -> Result<(Encoder, usize), RLNCError> {
        let in_data_len = data.len();
        let piece_byte_len = in_data_len / piece_count;
        let computed_total_data_len = piece_byte_len * piece_count;

        if computed_total_data_len != in_data_len {
            return Err(RLNCError::DataLengthMismatch);
        }

        Ok((Encoder { data, piece_count }, piece_byte_len))
    }

    /// Creates a new `Encoder` while padding the input data.
    ///
    /// The input data is padded with zeros to ensure its length is a multiple
    /// of `piece_count * piece_byte_len`, where `piece_byte_len` is calculated
    /// such that the original data plus a boundary marker fits within
    /// `piece_count` pieces. A boundary marker (`BOUNDARY_MARKER`) is placed
    /// at the end of the original data before zero padding.
    ///
    /// Returns the `Encoder` and the calculated byte length of each piece
    /// after padding.
    pub fn new(mut data: Vec<u8>, piece_count: usize) -> (Encoder, usize) {
        let in_data_len = data.len();
        let boundary_marker_len = 1;
        let piece_byte_len = (in_data_len + boundary_marker_len).div_ceil(piece_count);
        let padded_data_len = piece_count * piece_byte_len;

        data.resize(padded_data_len, 0);
        data[in_data_len] = BOUNDARY_MARKER;

        (Encoder { data, piece_count }, piece_byte_len)
    }

    /// Encodes the data held by the encoder using a provided coding vector.
    ///
    /// The resulting coded piece is returned as a `Vec<u8>`, prefixed by the
    /// coding vector itself (as `u8` values). The total length of the returned
    /// vector is `piece_count + piece_byte_len`.
    ///
    /// Returns `RLNCError::CodingVectorLengthMismatch` if the length of the
    /// provided `coding_vector` does not match `self.piece_count`.
    pub fn code_with_coding_vector(&self, coding_vector: &[Gf256]) -> Result<Vec<u8>, RLNCError> {
        if coding_vector.len() != self.piece_count {
            return Err(RLNCError::CodingVectorLengthMismatch);
        }

        let piece_byte_len = self.data.len() / self.piece_count;

        let coded_piece = self
            .data
            .chunks_exact(piece_byte_len)
            .zip(coding_vector)
            .map(|(piece, &random_symbol)| piece.iter().map(|&symbol| Gf256::new(symbol) * random_symbol).collect::<Vec<Gf256>>())
            .fold(vec![Gf256::default(); piece_byte_len], |mut acc, cur| {
                acc.iter_mut().zip(cur).for_each(|(a, b)| {
                    *a += b;
                });

                acc
            })
            .iter()
            .map(|symbol| symbol.get())
            .collect::<Vec<u8>>();

        let mut full_coded_piece = vec![0u8; self.piece_count + piece_byte_len];

        full_coded_piece[..self.piece_count].iter_mut().enumerate().for_each(|(idx, symbol)| {
            *symbol = coding_vector[idx].get();
        });
        full_coded_piece[self.piece_count..].copy_from_slice(&coded_piece);

        Ok(full_coded_piece)
    }

    /// Encodes the data held by the encoder using a randomly sampled coding vector.
    ///
    /// A coding vector of `self.piece_count` random `Gf256` symbols is generated
    /// using the provided random number generator.
    ///
    /// Calls `code_with_coding_vector` internally.
    ///
    /// Returns the coded piece prefixed by the random coding vector.
    pub fn code<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Vec<u8>, RLNCError> {
        let random_coding_vector = (0..self.piece_count).map(|_| rng.random()).collect::<Vec<Gf256>>();

        self.code_with_coding_vector(&random_coding_vector)
    }
}
