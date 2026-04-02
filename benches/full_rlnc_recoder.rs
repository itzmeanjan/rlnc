use criterion::*;
use rand::Rng;
use rlnc::full::{Encoder, Recoder};
use std::{fmt::Debug, hint::black_box, time::Duration};

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

    format!("{:.1}{}", bytes, units[unit_index])
}

impl Debug for RLNCConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}/{}-pieces/{}-pieces",
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

fn recode(c: &mut Criterion) {
    let mut group = c.benchmark_group("recode");

    for rlnc_config in ARGS {
        let mut rng = rand::rng();

        let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create RLNC encoder");

        let coded_pieces = (0..rlnc_config.recoding_with_piece_count)
            .flat_map(|_| encoder.code(&mut rng))
            .collect::<Vec<u8>>();
        let mut recoder =
            Recoder::new(coded_pieces.clone(), encoder.piece_byte_len(), encoder.piece_count()).expect("Failed to create RLNC recoder");

        group.measurement_time(Duration::from_secs(20));
        group.sample_size(100);

        // Number of bytes used as input to recoder + Number of bytes for each recoded piece
        group.throughput(Throughput::Bytes(
            (recoder.full_coded_piece_byte_len() * recoder.recoded_piece_count() + recoder.full_coded_piece_byte_len()) as u64,
        ));
        group.bench_function(format!("{:?}", rlnc_config), move |b| {
            b.iter(|| black_box(&mut recoder).recode(black_box(&mut rng)));
        });
    }

    group.finish();
}

fn recode_zero_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("recode_zero_alloc");

    for rlnc_config in ARGS {
        let mut rng = rand::rng();

        let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create RLNC encoder");

        let coded_pieces = (0..rlnc_config.recoding_with_piece_count)
            .flat_map(|_| encoder.code(&mut rng))
            .collect::<Vec<u8>>();
        let mut recoder =
            Recoder::new(coded_pieces.clone(), encoder.piece_byte_len(), encoder.piece_count()).expect("Failed to create RLNC recoder");

        let mut full_recoded_piece = vec![0u8; recoder.full_coded_piece_byte_len()];

        group.measurement_time(Duration::from_secs(20));
        group.sample_size(100);

        // Number of bytes used as input to recoder + Number of bytes for each recoded piece
        group.throughput(Throughput::Bytes(
            (recoder.full_coded_piece_byte_len() * recoder.recoded_piece_count() + recoder.full_coded_piece_byte_len()) as u64,
        ));
        group.bench_function(format!("{:?}", rlnc_config), move |b| {
            b.iter(|| black_box(&mut recoder).recode_with_buf(black_box(&mut rng), black_box(&mut full_recoded_piece)));
        });
    }

    group.finish();
}

criterion_group!(rlnc_recoder, recode, recode_zero_alloc);
criterion_main!(rlnc_recoder);
