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
    let mut rng = rand::rng();
    let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

    let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create RLNC encoder");
    let coded_pieces = (0..rlnc_config.recoding_with_piece_count)
        .flat_map(|_| {
            let mut coded_piece = vec![0u8; encoder.get_full_coded_piece_byte_len()];
            encoder.code(&mut rng, &mut coded_piece);
            coded_piece
        })
        .collect::<Vec<u8>>();

    bencher
        .with_inputs(|| {
            // Pre-allocate the final output buffer ONCE.
            let output_buffer = vec![0u8; encoder.get_full_coded_piece_byte_len()];
            (
                rand::rng(),
                Recoder::new(coded_pieces.clone(), encoder.get_full_coded_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create RLNC recoder"),
                output_buffer,
            )
        })
        .input_counter(|(_, recoder, _)| {
            divan::counter::BytesCount::new(
                recoder.get_full_coded_piece_byte_len() * recoder.get_num_pieces_recoded_together() + // Number of bytes used as input to recoder
                recoder.get_full_coded_piece_byte_len(), // Number of bytes for each recoded piece
            )
        })
        .bench_local_refs(|(rng, recoder, output_buffer)| {
            recoder.recode(rng, output_buffer).unwrap();
            divan::black_box(output_buffer);
        });
}
