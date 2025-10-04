use criterion::*;
use rand::Rng;
use rlnc::full::Encoder;
use std::{fmt::Debug, hint::black_box, time::Duration};

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
            "{} data split into {} pieces",
            &bytes_to_human_readable(self.data_byte_len),
            self.piece_count
        ))
    }
}

const ARGS: &[RLNCConfig] = &[
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 4,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 6,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 7,
    },
    RLNCConfig {
        data_byte_len: 1usize << 20,
        piece_count: 1usize << 8,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 4,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 6,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 7,
    },
    RLNCConfig {
        data_byte_len: 1usize << 24,
        piece_count: 1usize << 8,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 4,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 5,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 6,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 7,
    },
    RLNCConfig {
        data_byte_len: 1usize << 25,
        piece_count: 1usize << 8,
    },
];

fn encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for rlnc_config in ARGS {
        let mut rng = rand::rng();
        let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create RLNC encoder");

        group.measurement_time(Duration::from_secs(20));
        group.sample_size(100);

        // Number of bytes used as input to encoder + Number of bytes for each coded piece
        group.throughput(Throughput::Bytes(
            (encoder.get_piece_byte_len() * encoder.get_piece_count() + encoder.get_full_coded_piece_byte_len()) as u64,
        ));
        group.bench_function(format!("{:?}", rlnc_config), |b| {
            b.iter(|| black_box(&encoder).code(black_box(&mut rng)));
        });
    }

    group.finish();
}

fn encode_zero_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_zero_alloc");

    for rlnc_config in ARGS {
        let mut rng = rand::rng();
        let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();

        let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create RLNC encoder");
        let mut full_coded_piece = vec![0u8; encoder.get_full_coded_piece_byte_len()];

        group.measurement_time(Duration::from_secs(20));
        group.sample_size(100);

        // Number of bytes used as input to encoder + Number of bytes for each coded piece
        group.throughput(Throughput::Bytes(
            (encoder.get_piece_byte_len() * encoder.get_piece_count() + encoder.get_full_coded_piece_byte_len()) as u64,
        ));
        group.bench_function(format!("{:?}", rlnc_config), |b| {
            b.iter(|| black_box(&encoder).code_with_buf(black_box(&mut rng), black_box(&mut full_coded_piece)));
        });
    }

    group.finish();
}

criterion_group!(rlnc_encoder, encode, encode_zero_alloc);
criterion_main!(rlnc_encoder);
