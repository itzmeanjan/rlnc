use super::encoder::Encoder;
use crate::{RLNCError, common::gf256::Gf256};
use rand::Rng;

#[derive(Clone)]
pub struct Recoder {
    coding_vectors: Vec<Gf256>,
    encoder: Encoder,
    num_pieces_received: usize,
    piece_byte_len: usize,
    num_pieces_coded_together: usize,
}

impl Recoder {
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

    pub fn recode<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Vec<u8>, RLNCError> {
        let random_coding_vector = (0..self.num_pieces_received).map(|_| rng.random()).collect::<Vec<Gf256>>();
        let rhs_mat_cols = self.num_pieces_coded_together;

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
