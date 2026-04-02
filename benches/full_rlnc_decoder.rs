use criterion::*;
use rand::Rng;
use rlnc::full::{Decoder, Encoder};
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

    format!("{:.1}{}", bytes, units[unit_index])
}

impl Debug for RLNCConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}/{}-pieces", &bytes_to_human_readable(self.data_byte_len), self.piece_count))
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

fn decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for rlnc_config in ARGS {
        let mut rng = rand::rng();

        let data = (0..rlnc_config.data_byte_len).map(|_| rng.random()).collect::<Vec<u8>>();
        let encoder = Encoder::new(data, rlnc_config.piece_count).expect("Failed to create RLNC encoder");

        let num_pieces_to_produce = rlnc_config.piece_count * 2;
        let coded_pieces = (0..num_pieces_to_produce).flat_map(|_| encoder.code(&mut rng)).collect::<Vec<u8>>();

        let decoder = Decoder::new(encoder.piece_byte_len(), encoder.piece_count());

        group.measurement_time(Duration::from_secs(20));
        group.sample_size(100);

        group.throughput(Throughput::Bytes(
            (decoder.full_coded_piece_byte_len() * decoder.piece_count().value()) as u64,
        ));
        group.bench_function(format!("{:?}", rlnc_config), move |b| {
            b.iter_batched(
                || decoder.clone(),
                |mut dec| {
                    let mut coded_pieces_iter = coded_pieces.chunks_exact(dec.full_coded_piece_byte_len());

                    while !black_box(&dec).is_already_decoded() {
                        let coded_piece = unsafe { coded_pieces_iter.next().unwrap_unchecked() };
                        let _ = black_box(&mut dec).decode(black_box(coded_piece));
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

criterion_group!(rlnc_decoder, decode);
criterion_main!(rlnc_decoder);
