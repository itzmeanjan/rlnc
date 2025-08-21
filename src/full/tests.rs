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

        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder");
        let mut decoder = Decoder::new(encoder.get_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create Decoder");

        loop {
            let coded_piece = encoder.code(&mut rng);

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

        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder");
        let mut decoder = Decoder::new(encoder.get_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create Decoder");

        'OUTER: loop {
            let num_pieces_to_recode = rng.random_range(MIN_NUM_PIECES_TO_RECODE..=MAX_NUM_PIECES_TO_RECODE);

            let coded_pieces = (0..num_pieces_to_recode).flat_map(|_| encoder.code(&mut rng)).collect::<Vec<u8>>();

            let mut recoder = Recoder::new(coded_pieces, encoder.get_full_coded_piece_byte_len(), encoder.get_piece_count())
                .expect("Construction of RLNC recoder must not fail!");

            let num_recoded_pieces_to_use = rng.random_range(MIN_NUM_RECODED_PIECES_TO_USE..=MAX_NUM_RECODED_PIECES_TO_USE);
            let mut recoded_piece_idx = 0;

            while recoded_piece_idx < num_recoded_pieces_to_use {
                let single_recoded_piece = recoder.recode(&mut rng);

                match decoder.decode(&single_recoded_piece) {
                    Ok(_) => {}
                    Err(e) => match e {
                        RLNCError::ReceivedAllPieces => break 'OUTER,
                        RLNCError::PieceNotUseful => {}
                        _ => panic!("Did not expect this error during decoding: {e}"),
                    },
                };

                recoded_piece_idx += 1;
            }

            let coded_piece = encoder.code(&mut rng);
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
fn prop_test_rlnc_decoding_with_useless_pieces() {
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

        // Create Full RLNC Encoder
        let encoder = Encoder::new(data, piece_count).expect("Failed to create Encoder");
        // Create Full RLNC Decoder
        let mut decoder = Decoder::new(encoder.get_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create Decoder");

        // Reserve memory for holding coded pieces, which are to be used for recoding.
        let num_pieces_to_use_for_recoding = piece_count / 2;
        let mut coded_pieces_for_recoding = Vec::with_capacity(encoder.get_full_coded_piece_byte_len() * num_pieces_to_use_for_recoding);

        // Generate some coded pieces, push them into Decoder and keep their copy so that they can be used for recoding.
        (0..num_pieces_to_use_for_recoding).for_each(|_| {
            let coded_piece = encoder.code(&mut rng);

            match decoder.decode(&coded_piece) {
                Ok(_) => coded_pieces_for_recoding.extend_from_slice(&coded_piece),
                Err(e) => match e {
                    RLNCError::PieceNotUseful => {}
                    _ => panic!("Did not expect this error during decoding: {e}"),
                },
            };
        });

        // Build a Recoder with coded pieces produced in previous phase.
        //
        // Now notice, these coded pieces are already consumed by the Decoder, but anyway we'll use RLNC Recoder to produce
        // new pieces from previously consumed coded pieces. And those recoded pieces will all be useless, because the recoder will
        // just produce new linear combination of existing coded pieces, and they can't be linearly independent from all coded pieces
        // which were already seen by the Decoder.
        let mut recoder = Recoder::new(coded_pieces_for_recoding, encoder.get_full_coded_piece_byte_len(), encoder.get_piece_count())
            .expect("Must be able to build a Recoder");

        // Hence in following loop, decoding process won't progress, because all the recoded pieces will be useless.
        let num_recoded_pieces_to_use = num_pieces_to_use_for_recoding * 2;

        (0..num_recoded_pieces_to_use).for_each(|_| {
            let recoded_piece = recoder.recode(&mut rng);

            match decoder.decode(&recoded_piece) {
                Ok(_) => panic!("Decoding with linearly dependent coded piece must not succeed!"),
                Err(e) => match e {
                    RLNCError::PieceNotUseful => {}
                    _ => panic!("Did not expect this error during this phase of decoding: {e}"),
                },
            };
        });

        // Finally, we can grab new coded pieces from directly the Encoder to finalize the decoding process.
        while decoder.get_remaining_piece_count() > 0 {
            let coded_piece = encoder.code(&mut rng);

            match decoder.decode(&coded_piece) {
                Ok(_) => {}
                Err(e) => match e {
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
