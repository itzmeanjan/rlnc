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
        data_byte_len: 1usize << 10,
        piece_count: 1usize << 4,
        recoding_with_piece_count: 1usize << 3,
    },
    RLNCConfig {
        data_byte_len: 1usize << 14,
        piece_count: 1usize << 6,
        recoding_with_piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 18,
        piece_count: 1usize << 8,
        recoding_with_piece_count: 1usize << 7,
    },
    RLNCConfig {
        data_byte_len: 1usize << 22,
        piece_count: 1usize << 10,
        recoding_with_piece_count: 1usize << 9,
    },
    RLNCConfig {
        data_byte_len: 1usize << 26,
        piece_count: 1usize << 12,
        recoding_with_piece_count: 1usize << 11,
    },
];

#[divan::bench(args = ARGS, max_time = Duration::from_secs(300), skip_ext_time = true)]
fn recode(bencher: divan::Bencher, rlnc_config: &RLNCConfig) {
    let mut rng = rand::rng();
    let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

    let (encoder, piece_byte_len) = Encoder::new(data, rlnc_config.piece_count);
    let coded_pieces = (0..rlnc_config.recoding_with_piece_count)
        .flat_map(|_| encoder.code(&mut rng).unwrap())
        .collect::<Vec<u8>>();

    bencher
        .counter(divan::counter::BytesCount::new(rlnc_config.recoding_with_piece_count + coded_pieces.len()))
        .with_inputs(|| {
            let rng = rand::rng();
            let recoder = Recoder::new(coded_pieces.clone(), piece_byte_len, rlnc_config.piece_count).unwrap();

            (rng, recoder)
        })
        .bench_refs(|(rng, recoder)| divan::black_box(&recoder).recode(divan::black_box(rng)));
}
