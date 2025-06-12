use rand::Rng;
use rlnc::full::encoder::Encoder;
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
            "{} data splitted into {} pieces",
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
fn encode(bencher: divan::Bencher, rlnc_config: &RLNCConfig) {
    let mut rng = rand::rng();
    let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

    let (encoder, _) = Encoder::new(data, rlnc_config.piece_count);

    bencher
        .counter(divan::counter::BytesCount::new(rlnc_config.piece_count + rlnc_config.data_byte_len))
        .with_inputs(rand::rng)
        .bench_refs(|rng| divan::black_box(&encoder).code(divan::black_box(rng)));
}
