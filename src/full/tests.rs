#![cfg(test)]

use super::{decoder::Decoder, encoder::Encoder, recoder::Recoder};
use crate::RLNCError;
use rand::Rng;

#[test]
fn prop_test_rlnc_encoder_decoder() {
    const NUM_TEST_ITERATIONS: usize = 10;

    const MIN_DATA_BYTE_LEN: usize = 1usize << 10;
    const MAX_DATA_BYTE_LEN: usize = 1usize << 16;

    const MIN_PIECE_COUNT: usize = 1usize << 5;
    const MAX_PIECE_COUNT: usize = 1usize << 11;

    let mut rng = rand::rng();

    (0..NUM_TEST_ITERATIONS).for_each(|_| {
        let data_byte_len = rng.random_range(MIN_DATA_BYTE_LEN..=MAX_DATA_BYTE_LEN);
        let piece_count = rng.random_range(MIN_PIECE_COUNT..=MAX_PIECE_COUNT);

        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let data_copy = data.clone();

        let (encoder, piece_byte_len) = Encoder::new(data, piece_count);
        let mut decoder = Decoder::new(piece_byte_len, piece_count);

        loop {
            let coded_piece = encoder.code(&mut rng).expect("Generating new RLNC coded piece must not fail!");

            match decoder.decode(&coded_piece) {
                Ok(_) => {}
                Err(e) => match e {
                    RLNCError::ReceivedAllPieces => break,
                    RLNCError::PieceNotUseful => continue,
                    _ => panic!("Did not expect this error during decoding: {e}"),
                },
            };
        }

        assert!(decoder.is_already_decoded());
        let decoded_data = decoder.get_decoded_data().expect("Extracting decoded data must not fail!");

        assert_eq!(data_copy, decoded_data);
    });
}

#[test]
fn prop_test_rlnc_encoder_recoder_decoder() {
    const NUM_TEST_ITERATIONS: usize = 10;

    const MIN_DATA_BYTE_LEN: usize = 1usize << 10;
    const MAX_DATA_BYTE_LEN: usize = 1usize << 16;

    const MIN_PIECE_COUNT: usize = 1usize << 5;
    const MAX_PIECE_COUNT: usize = 1usize << 11;

    const MIN_NUM_PIECES_TO_RECODE: usize = 2;
    const MAX_NUM_PIECES_TO_RECODE: usize = (MIN_PIECE_COUNT + MAX_PIECE_COUNT) / 2;

    const MIN_NUM_RECODED_PIECES_TO_USE: usize = 0;
    const MAX_NUM_RECODED_PIECES_TO_USE: usize = u8::MAX as usize * 10;

    let mut rng = rand::rng();

    (0..NUM_TEST_ITERATIONS).for_each(|_| {
        let data_byte_len = rng.random_range(MIN_DATA_BYTE_LEN..=MAX_DATA_BYTE_LEN);
        let piece_count = rng.random_range(MIN_PIECE_COUNT..=MAX_PIECE_COUNT);

        let data = (0..data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let data_copy = data.clone();

        let (encoder, piece_byte_len) = Encoder::new(data, piece_count);
        let mut decoder = Decoder::new(piece_byte_len, piece_count);

        'OUTER: loop {
            let num_pieces_to_recode = rng.random_range(MIN_NUM_PIECES_TO_RECODE..=MAX_NUM_PIECES_TO_RECODE);

            let coded_pieces = (0..num_pieces_to_recode)
                .flat_map(|_| encoder.code(&mut rng).expect("Generating new RLNC coded piece must not fail!"))
                .collect::<Vec<u8>>();

            let recoder = Recoder::new(coded_pieces, piece_byte_len, piece_count).expect("Construction of RLNC recoder must not fail!");

            let num_recoded_pieces_to_use = rng.random_range(MIN_NUM_RECODED_PIECES_TO_USE..=MAX_NUM_RECODED_PIECES_TO_USE);
            let mut recoded_piece_idx = 0;

            while recoded_piece_idx < num_recoded_pieces_to_use {
                let recoded_piece = recoder.recode(&mut rng).expect("Recoding a new piece from existing pieces must not fail!");

                match decoder.decode(&recoded_piece) {
                    Ok(_) => {}
                    Err(e) => match e {
                        RLNCError::ReceivedAllPieces => break 'OUTER,
                        RLNCError::PieceNotUseful => {}
                        _ => panic!("Did not expect this error during decoding: {e}"),
                    },
                };

                recoded_piece_idx += 1;
            }

            let coded_piece = encoder.code(&mut rng).expect("Generating new RLNC coded piece must not fail!");
            match decoder.decode(&coded_piece) {
                Ok(_) => {}
                Err(e) => match e {
                    RLNCError::ReceivedAllPieces => break,
                    RLNCError::PieceNotUseful => continue,
                    _ => panic!("Did not expect this error during decoding: {e}"),
                },
            };
        }

        assert!(decoder.is_already_decoded());
        let decoded_data = decoder.get_decoded_data().expect("Extracting decoded data must not fail!");

        assert_eq!(data_copy, decoded_data);
    });
}
