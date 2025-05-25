use rand::Rng;
use rlnc::{
    RLNCError,
    full::{decoder::Decoder, encoder::Encoder},
};
use std::{fmt::Debug, time::Duration};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::default().bytes_format(divan::counter::BytesFormat::Binary).main();
}

struct RLNCConfig {
    data_byte_len: usize,
    piece_count: usize,
}

fn bytes_to_human_readable(bytes: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut bytes = bytes as f64;
    let mut unit_index = 0;

    while bytes >= 1024.0 && unit_index < units.len() - 1 {
        bytes /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", bytes, units[unit_index])
}

impl Debug for RLNCConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Data: {}, Piece Count: {}",
            &bytes_to_human_readable(self.data_byte_len),
            self.piece_count
        ))
    }
}

const ARGS: &[RLNCConfig] = &[
    RLNCConfig {
        data_byte_len: 1usize << 10,
        piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 15,
        piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 10,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 10,
    },
    RLNCConfig {
        data_byte_len: 1usize << 30,
        piece_count: 1usize << 15,
    },
];

#[divan::bench(args = ARGS, max_time = Duration::from_secs(300), skip_ext_time = true)]
fn decode(bencher: divan::Bencher, rlnc_config: &RLNCConfig) {
    let mut rng = rand::rng();

    let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
    let (encoder, piece_byte_len) = Encoder::new(data, rlnc_config.piece_count);

    let num_pieces_to_produce = rlnc_config.piece_count * 2;
    let coded_pieces = (0..num_pieces_to_produce).flat_map(|_| encoder.code(&mut rng).unwrap()).collect::<Vec<u8>>();

    bencher
        .counter(divan::counter::BytesCount::new(rlnc_config.piece_count * rlnc_config.data_byte_len))
        .with_inputs(|| Decoder::new(piece_byte_len, rlnc_config.piece_count))
        .bench_refs(|mut decoder| {
            let mut piece_index = 0;
            let per_piece_byte_len = coded_pieces.len() / num_pieces_to_produce;

            while piece_index < num_pieces_to_produce {
                let coded_piece_begins_at = piece_index * per_piece_byte_len;
                let coded_piece_ends_at = coded_piece_begins_at + per_piece_byte_len;
                let coded_piece = &coded_pieces[coded_piece_begins_at..coded_piece_ends_at];

                match divan::black_box(&mut decoder).decode(divan::black_box(coded_piece)) {
                    Ok(_) => {}
                    Err(e) => match e {
                        RLNCError::ReceivedAllPieces => break,
                        RLNCError::PieceNotUseful => continue,
                        _ => panic!("Did not expect this error during decoding: {}", e),
                    },
                };

                piece_index += 1;
            }
        });
}
