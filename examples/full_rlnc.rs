use rand::Rng;
use rlnc::{
    RLNCError,
    full::{decoder::Decoder, encoder::Encoder, recoder::Recoder},
};

fn main() {
    let mut rng = rand::rng();

    // 1. Define original data parameters
    let original_data_len = 1024 * 10; // 10 KB
    let piece_count = 32; // Data will be split into 32 pieces
    let original_data: Vec<u8> = (0..original_data_len).map(|_| rng.random()).collect();
    let original_data_copy = original_data.clone();

    // 2. Initialize the Encoder
    let encoder = Encoder::new(original_data, piece_count).expect("Failed to create RLNC encoder");
    println!(
        "Initialized Encoder with {} bytes of data, split into {} pieces, each of {} bytes. Each coded piece will be of {} bytes.",
        original_data_len,
        piece_count,
        encoder.get_piece_byte_len(),
        encoder.get_full_coded_piece_byte_len()
    );

    // Show the overhead of encodng as a % on top of the original data size
    let overhead = (encoder.get_full_coded_piece_byte_len() * piece_count) as f64 / original_data_len as f64 * 100.0 - 100.0;
    println!("Overhead of encoding: {overhead:.2}%");

    // 3. Initialize the Decoder
    println!(
        "Initializing Decoder, expecting {} original pieces of {} bytes each.",
        encoder.get_piece_count(),
        encoder.get_piece_byte_len()
    );
    let mut decoder = Decoder::new(encoder.get_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create RLNC decoder");

    // 4. Simulate a sender generating initial coded pieces
    let num_initial_coded_pieces_from_sender = encoder.get_piece_count() / 2; // Send half directly
    println!("\nSender generating {num_initial_coded_pieces_from_sender} initial coded pieces...");
    let mut pieces_for_recoder = Vec::new();

    for i in 0..num_initial_coded_pieces_from_sender {
        let coded_piece = encoder.code(&mut rng);
        pieces_for_recoder.extend_from_slice(&coded_piece); // Collect the same coded piece for recoder

        match decoder.decode(&coded_piece) {
            Ok(_) => println!("  Decoded direct piece {}: Useful.", i + 1),
            Err(RLNCError::PieceNotUseful) => println!("  Decoded direct piece {}: Not useful.", i + 1),
            Err(RLNCError::ReceivedAllPieces) => {
                println!("  Decoded direct piece {}: All pieces received, breaking.", i + 1);
                break;
            }
            Err(e) => panic!("Unexpected error during direct decoding: {e:?}"),
        }
    }

    // 5. Initialize the Recoder with same coded pieces which were already used for decoding
    println!("\nInitializing Recoder with {} bytes of received coded pieces.", pieces_for_recoder.len());
    let mut recoder =
        Recoder::new(pieces_for_recoder, encoder.get_full_coded_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create RLNC recoder");

    // 6. Generate recoded pieces and feed them to the decoder, though all of the recoded pieces will be linearly dependent on the original pieces
    println!("\nRecoder active. Generating recoded pieces...");
    let num_recoded_pieces_to_send = encoder.get_piece_count() * 2; // Send many recoded pieces, though all of them will be useless

    for i in 0..num_recoded_pieces_to_send {
        // This condition will never be executed because the decoder will not see a single useful coded piece while executing inside this loop
        if decoder.is_already_decoded() {
            println!("  All necessary pieces received via recoding.");
            break;
        }

        let recoded_piece = recoder.recode(&mut rng);

        match decoder.decode(&recoded_piece) {
            Ok(_) => println!("  Decoded recoded piece {}: Useful.", i + 1),
            Err(RLNCError::PieceNotUseful) => println!("  Decoded recoded piece {}: Not useful.", i + 1),
            Err(RLNCError::ReceivedAllPieces) => {
                println!("  Decoded recoded piece {}: All pieces received, breaking.", i + 1);
                break;
            }
            Err(e) => panic!("Unexpected error during recoded piece decoding: {e:?}"),
        }
    }

    // 7. Generate new coded pieces for recoding, so that we can actually demonstrate the power of recoding in RLNC
    let mut pieces_for_new_recoder = Vec::new();
    for _ in 0..num_initial_coded_pieces_from_sender {
        let coded_piece = encoder.code(&mut rng);
        pieces_for_new_recoder.extend_from_slice(&coded_piece); // Collect for new recoder
    }

    println!(
        "\nInitializing a new Recoder with {} bytes of received coded pieces.",
        pieces_for_new_recoder.len()
    );
    let mut recoder =
        Recoder::new(pieces_for_new_recoder, encoder.get_full_coded_piece_byte_len(), encoder.get_piece_count()).expect("Must be able to build a new recoder");

    // 8. Generate new recoded pieces and feed them to the decoder. Now most of these recoded pieces will be useful, as these pieces were never seen by the decoder before.
    let num_recoded_pieces_to_send = num_initial_coded_pieces_from_sender / 2; // Send some recoded pieces for decoding
    for i in 0..num_recoded_pieces_to_send {
        if decoder.is_already_decoded() {
            println!("  All necessary pieces received via recoding.");
            break;
        }

        let recoded_piece = recoder.recode(&mut rng);

        match decoder.decode(&recoded_piece) {
            Ok(_) => println!("  Decoded recoded piece {}: Useful.", i + 1),
            Err(RLNCError::PieceNotUseful) => println!("  Decoded recoded piece {}: Not useful.", i + 1),
            Err(RLNCError::ReceivedAllPieces) => {
                println!("  Decoded recoded piece {}: All pieces received, breaking.", i + 1);
                break;
            }
            Err(e) => panic!("Unexpected error during recoded piece decoding: {e:?}"),
        }
    }

    // 9. If not yet decoded, continue generating direct coded pieces from encoder
    let mut direct_piece_count = num_initial_coded_pieces_from_sender;
    while !decoder.is_already_decoded() {
        println!("\nStill need more pieces. Generating direct piece {} from encoder...", direct_piece_count + 1);
        let coded_piece = encoder.code(&mut rng);

        match decoder.decode(&coded_piece) {
            Ok(_) => {
                direct_piece_count += 1;
                println!("  Decoded direct piece {direct_piece_count}: Useful.");
            }
            Err(RLNCError::PieceNotUseful) => {
                println!("  Decoded direct piece {}: Not useful.", direct_piece_count + 1);
            }
            Err(RLNCError::ReceivedAllPieces) => {
                println!("  All pieces received via direct encoding.");
                break;
            }
            Err(e) => panic!("Unexpected error during direct decoding (post-recoding): {e:?}"),
        }
    }

    // 8. Retrieve the decoded data
    println!("\nRetrieving decoded data...");
    let decoded_data = decoder.get_decoded_data().expect("Failed to retrieve decoded data after all pieces received");

    // 9. Verify that the decoded data matches the original data
    assert_eq!(original_data_copy, decoded_data);
    println!("\nRLNC workflow completed successfully! Original data matches decoded data.");
}
