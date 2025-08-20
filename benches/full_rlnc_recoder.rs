use rand::Rng;
use rlnc::full::{encoder::Encoder, recoder::Recoder};
use std::{fmt::Debug, time::Duration};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::default().bytes_format(divan::counter::BytesFormat::Binary).main();
}

struct RLNCConfig {
    data_byte_len: usize,
    piece_count: usize,
    recoding_with_piece_count: usize,
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
            "{} data splitted into {} pieces, recoding with {} pieces",
            &bytes_to_human_readable(self.data_byte_len),
            self.piece_count,
            self.recoding_with_piece_count
        ))
    }
}

const ARGS: &[RLNCConfig] = &[
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 4,
        recoding_with_piece_count: 1usize << 3,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 5,
        recoding_with_piece_count: 1usize << 4,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 6,
        recoding_with_piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 7,
        recoding_with_piece_count: 1usize << 6,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 8,
        recoding_with_piece_count: 1usize << 7,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 4,
        recoding_with_piece_count: 1usize << 3,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 5,
        recoding_with_piece_count: 1usize << 4,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 6,
        recoding_with_piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 7,
        recoding_with_piece_count: 1usize << 6,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 8,
        recoding_with_piece_count: 1usize << 7,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 4,
        recoding_with_piece_count: 1usize << 3,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 5,
        recoding_with_piece_count: 1usize << 4,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 6,
        recoding_with_piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 7,
        recoding_with_piece_count: 1usize << 6,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 8,
        recoding_with_piece_count: 1usize << 7,
    },
];

#[divan::bench(args = ARGS, max_time = Duration::from_secs(100), skip_ext_time = true)]
fn recode(bencher: divan::Bencher, rlnc_config: &RLNCConfig) {
    bencher
        // --- 1. SETUP ---
        // Create all the necessary inputs once.
        .with_inputs(|| {
            let mut rng = rand::rng();

            // We need an encoder to generate some pieces for the recoder to use.
            let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
            let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create encoder for recoder bench");

            // Generate some coded pieces to initialize the recoder.
            let num_pieces_for_recoder = rlnc_config.piece_count / 2;
            let coded_pieces: Vec<u8> = (0..num_pieces_for_recoder).flat_map(|_| encoder.code(&mut rng)).collect();

            // Create the Recoder instance. This now allocates its own internal workspace buffers.
            let recoder =
                Recoder::new(coded_pieces, encoder.get_full_coded_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create RLNC recoder");

            // Pre-allocate the final output buffer that will be reused in the hot loop.
            let output_buffer = vec![0u8; recoder.get_full_coded_piece_byte_len()];

            (rng, recoder, output_buffer)
        })
        // --- 2. THE BENCHMARK LOOP ---
        // Use `bench_local_refs` to get mutable access.
        .bench_local_refs(|(rng, recoder, output_buffer)| {
            // Call the new zero-copy recode function.
            recoder.recode(rng, output_buffer).unwrap();

            // Black-box the result to prevent optimizations.
            divan::black_box(output_buffer);
        });
}
