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
#[derive(Clone)]
pub struct Recoder {
    coding_vectors: Vec<Gf256>,
    encoder: Encoder,
    num_pieces_received: usize,
    piece_byte_len: usize,
    num_pieces_coded_together: usize,
}

impl Recoder {
    /// Creates a new `Recoder` instance from a vector of received coded pieces.
    ///
    /// Each coded piece in `data` is expected to be prepended with its coding
    /// vector. The length of each coded piece (including the coding vector)
    /// must be consistent.
    ///
    /// The `Recoder` extracts the coding vectors and coded pieces from the input
    /// data. It then initializes an internal `Encoder` that implicitly
    /// represents the source pieces extracted from the input.
    ///
    /// # Arguments
    ///
    /// * `data`: A vector of bytes containing the concatenated coded pieces, each
    ///   preceded by its coding vector. The total length must be a multiple
    ///   of `num_pieces_coded_together + piece_byte_len`.
    /// * `piece_byte_len`: The byte length of the original (uncoded) data pieces.
    /// * `num_pieces_coded_together`: The number of original pieces that were
    ///   linearly combined to create each coded piece. This is also the length
    ///   of the coding vector prepended to each coded piece.
    ///
    /// # Errors
    ///
    /// Returns `RLNCError::NotEnoughPiecesToRecode` if the input `data` is empty
    /// or does not contain at least one full coded piece.
    pub fn new(data: Vec<u8>, piece_byte_len: usize, num_pieces_coded_together: usize) -> Result<Recoder, RLNCError> {
        let full_coded_piece_len = num_pieces_coded_together + piece_byte_len;
        let num_pieces_received = data.len() / full_coded_piece_len;

        if num_pieces_received == 0 {
            return Err(RLNCError::NotEnoughPiecesToRecode);
        }

        let mut coding_vectors = Vec::with_capacity(num_pieces_received * num_pieces_coded_together);
        let mut coded_pieces = Vec::with_capacity(num_pieces_received * piece_byte_len);

        data.chunks_exact(full_coded_piece_len).for_each(|full_coded_piece| {
            let coding_vector = &full_coded_piece[..num_pieces_coded_together];
            let coded_piece = &full_coded_piece[num_pieces_coded_together..];

            coding_vectors.extend(coding_vector.iter().map(|&symbol| Gf256::new(symbol)));
            coded_pieces.extend_from_slice(coded_piece);
        });

        let (encoder, _) = Encoder::without_padding(coded_pieces, num_pieces_received)?;
        Ok(Recoder {
            coding_vectors,
            encoder,
            num_pieces_received,
            piece_byte_len,
            num_pieces_coded_together,
        })
    }

    /// Generates a new coded piece by recoding the source pieces using a randomly sampled coding vector.
    ///
    /// This method generates a random recoding vector (length `num_pieces_received`),
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
    /// Returns a `Result` containing a `Vec<u8>` representing the new coded
    /// piece prepended with its source coding vector on success.
    ///
    /// # Errors
    ///
    /// Returns an `RLNCError` if the internal `Encoder` fails to generate
    /// the coded piece (e.g., if the recoding vector is somehow invalid,
    /// though this is unlikely with a random vector).
    pub fn recode<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Vec<u8>, RLNCError> {
        let random_coding_vector = (0..self.num_pieces_received).map(|_| rng.random()).collect::<Vec<Gf256>>();
        let rhs_mat_cols = self.num_pieces_coded_together;

        // Compute the resulting coding vector for the original source pieces
        // by multiplying the random recoding vector by the matrix of received coding vectors.
        let mut computed_coding_vector = vec![0u8; rhs_mat_cols];
        computed_coding_vector.reserve(self.piece_byte_len);

        for (j, res_symbol) in computed_coding_vector.iter_mut().enumerate() {
            let mut local_res = Gf256::default();

            for (k, &random_symbol) in random_coding_vector.iter().enumerate() {
                local_res += random_symbol * self.coding_vectors[k * rhs_mat_cols + j];
            }

            *res_symbol = local_res.get();
        }

        let full_coded_piece = self.encoder.code_with_coding_vector(&random_coding_vector)?;
        let coded_piece = &full_coded_piece[self.num_pieces_received..];

        computed_coding_vector.extend_from_slice(coded_piece);
        Ok(computed_coding_vector)
    }
}
