# rlnc
Random Linear Network Coding

## Introduction
`rlnc` is a Rust library crate that implements Random Linear Network Coding (RLNC) over $GF(2^8)$ with primitive polynomial $x^8 + x^4 + x^3 + x^2 + 1$. This library provides functionalities for erasure-coding data, reconstructing original data from coded pieces, and recoding existing coded pieces to new erasure-coded pieces, without ever decoding it back to original data.

For a quick understanding of RLNC, have a look at my blog post @ https://itzmeanjan.in/pages/rlnc-in-depth.html.

Random Linear Network Coding (RLNC) excels in highly dynamic and lossy environments like multicast, peer-to-peer networks, and distributed storage, due to interesting properties such as encoding with random-sampled coefficients, any `k` out of `n` coded-pieces are sufficient to recover and recoding new pieces with existing erasure-coded pieces. Unlike Reed-Solomon, which requires specific symbols for deterministic recovery, RLNC allows decoding from *any* set of linearly independent packets. Compared to Fountain Codes, RLNC offers robust algebraic linearity with coding vector overhead, whereas Fountain codes prioritize very low decoding complexity and indefinite symbol generation, often for large-scale broadcasts.

## Features
For now this crate implements only **Full RLNC** scheme.

- **Encoder**: Splits original data into fixed-size pieces and generates new coded pieces by linearly combining these original pieces with random coefficients, sampled from $GF(2^8)$.
- **Decoder**: Receives coded pieces, applies Gaussian elimination to recover the original data, and handles linearly dependent pieces gracefully.
- **Recoder**: Takes already coded pieces and generates new coded pieces from them, facilitating multi-hop data distribution without requiring intermediate decoding.
- **Error Handling**: Defines a custom `RLNCError` enum to provide clear error messages for various operational failures.

## Prerequisites
Rust stable toolchain; see https://rustup.rs for installation guide. MSRV for this crate is 1.85.0.

 ```bash
# While developing this library, I was using
$ rustc --version
rustc 1.88.0 (6b00bc388 2025-06-23)
```

## Testing
For ensuring functional correctness of RLNC operations, the library includes a comprehensive test suite. Run all the tests by running following commands.

```bash
# Testing on host, first with `default` feature, then with `parallel` feature enabled.
make test

# Testing on web assembly target, using `wasmtime`.
rustup target add wasm32-wasip1
rustup target add wasm32-wasip2
cargo install wasmtime-cli@35.0.0 --locked

make test-wasm
```

```bash
running 14 tests
test full::decoder::tests::test_decoder_decode_invalid_piece_length ... ok
test full::decoder::tests::test_decoder_new_invalid_inputs ... ok
test full::encoder::tests::test_encoder_code_with_coding_vector_invalid_inputs ... ok
test full::decoder::tests::test_decoder_getters ... ok
test full::encoder::tests::test_encoder_getters ... ok
test full::encoder::tests::test_encoder_without_padding_invalid_data ... ok
test full::encoder::tests::test_encoder_new_invalid_inputs ... ok
test full::recoder::tests::test_recoder_getters ... ok
test full::recoder::tests::test_recoder_new_invalid_inputs ... ok
test common::gf256::test::prop_test_gf256_operations ... ok
test full::tests::prop_test_rlnc_encoder_decoder ... ok
test full::decoder_matrix::test::prop_test_rref_is_idempotent ... ok
test full::tests::prop_test_rlnc_encoder_recoder_decoder ... ok
test full::tests::prop_test_rlnc_decoding_with_useless_pieces has been running for over 60 seconds
test full::tests::prop_test_rlnc_decoding_with_useless_pieces ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 63.59s

   Doc-tests rlnc

running 3 tests
test src/common/simd_mul_table.rs - common::simd_mul_table (line 22) ... ignored
test src/common/simd_mul_table.rs - common::simd_mul_table (line 8) ... ignored
test src/lib.rs - (line 49) ... ok

test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Code Coverage
To generate a detailed code coverage report in HTML format, use [cargo-tarpaulin](https://github.com/xd009642/tarpaulin):

```bash
# Install cargo-tarpaulin if not already installed
cargo install cargo-tarpaulin
make coverage
```

```bash
Coverage Results:
|| Tested/Total Lines:
|| src/common/errors.rs: 0/1
|| src/common/gf256.rs: 9/11
|| src/common/simd.rs: 42/64
|| src/full/decoder.rs: 29/34
|| src/full/decoder_matrix.rs: 50/55
|| src/full/encoder.rs: 28/28
|| src/full/recoder.rs: 30/36
||
82.10% coverage, 188/229 lines covered
```

This will create an HTML coverage report at `tarpaulin-report.html` that you can open in your web browser to view detailed line-by-line coverage information for all source files.

> [!NOTE]
> There is a help menu, which introduces you to all available commands; just run `$ make` from the root directory of this project.

## Benchmarking
Performance benchmarks for several input configurations are included to evaluate the efficiency of this RLNC implementation.

To run the benchmarks, execute the following command from the root of the project:

```bash
make bench # First with `default` feature, then with `parallel` feature enabled.
```

> [!WARNING]
> When benchmarking make sure you've disabled CPU frequency scaling, otherwise numbers you see can be misleading. I find https://github.com/google/benchmark/blob/b40db869/docs/reducing_variance.md helpful.

### On 12th Gen Intel(R) Core(TM) i7-1260P

Running benchmarks on `Linux 6.14.0-27-generic x86_64`, compiled with `rustc 1.88.0 (6b00bc388 2025-06-23)`.

Component | Peak Median Throughput (`default` feature) | Peak Median Throughput (`parallel` feature) | Impact of number of pieces on performance
--- | --- | --- | ---
Full RLNC Encoder | **30.14 GiB/s** | **23.39 GiB/s** | The number of pieces original data got split into has a **minimal** impact on the encoding speed.
Full RLNC Recoder | **27.26 GiB/s** | **12.63 GiB/s** | Similar to the encoder, the recoder's performance remains largely consistent regardless of how many pieces the original data is split into.
Full RLNC Decoder | **1.59 GiB/s** | **Doesn't yet implement a parallel decoding mode** | As the number of pieces increases, the decoding time increases substantially, leading to a considerable drop in throughput. This indicates that decoding is the most computationally intensive part of the full RLNC scheme, and its performance is inversely proportional to the number of pieces.

In summary, the full RLNC implementation demonstrates excellent encoding and recoding speeds, consistently achieving GiB/s throughputs with minimal sensitivity to the number of data pieces. The `parallel` feature, leveraging Rust `rayon` data-parallelism framework, also provides good performance for both encoding and recoding. Whether you want to use that feature, completely depends on your usecase. However, decoding remains a much slower operation, with its performance significantly diminishing as the data is split into a greater number of pieces, and currently does **not** implement a parallel decoding algorithm.

<details>
<summary>Click to view detailed benchmark results 👇</summary>

#### Full RLNC Encoder

```bash
# Encoding without `rayon` data-parallelism, instead just using AVX2-powered SIMD vector x scalar multiplication

Timer precision: 22 ns
full_rlnc_encoder                             fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ encode                                                   │               │               │               │         │
   ├─ 1.00 MB data splitted into 16 pieces    32.89 µs      │ 127.4 µs      │ 40.52 µs      │ 41.51 µs      │ 100     │ 100
   │                                          31.54 GiB/s   │ 8.141 GiB/s   │ 25.6 GiB/s    │ 24.99 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            64.03 KiB   │ 64.03 KiB     │ 64.03 KiB     │ 64.03 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            64.03 KiB   │ 64.03 KiB     │ 64.03 KiB     │ 64.03 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            16 B        │ 16 B          │ 16 B          │ 16 B          │         │
   ├─ 1.00 MB data splitted into 32 pieces    41.24 µs      │ 56.61 µs      │ 43.1 µs       │ 43.33 µs      │ 100     │ 100
   │                                          24.41 GiB/s   │ 17.78 GiB/s   │ 23.36 GiB/s   │ 23.24 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            32.06 KiB   │ 32.06 KiB     │ 32.06 KiB     │ 32.06 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            32.06 KiB   │ 32.06 KiB     │ 32.06 KiB     │ 32.06 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            32 B        │ 32 B          │ 32 B          │ 32 B          │         │
   ├─ 1.00 MB data splitted into 64 pieces    31.63 µs      │ 43.55 µs      │ 32.9 µs       │ 33.61 µs      │ 100     │ 100
   │                                          31.36 GiB/s   │ 22.77 GiB/s   │ 30.14 GiB/s   │ 29.51 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            16.12 KiB   │ 16.12 KiB     │ 16.12 KiB     │ 16.12 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            16.12 KiB   │ 16.12 KiB     │ 16.12 KiB     │ 16.12 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            64 B        │ 64 B          │ 64 B          │ 64 B          │         │
   ├─ 1.00 MB data splitted into 128 pieces   35.49 µs      │ 60.33 µs      │ 36.4 µs       │ 37.23 µs      │ 100     │ 100
   │                                          27.73 GiB/s   │ 16.31 GiB/s   │ 27.04 GiB/s   │ 26.43 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            8.25 KiB    │ 8.25 KiB      │ 8.25 KiB      │ 8.25 KiB      │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            8.25 KiB    │ 8.25 KiB      │ 8.25 KiB      │ 8.25 KiB      │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            128 B       │ 128 B         │ 128 B         │ 128 B         │         │
   ├─ 1.00 MB data splitted into 256 pieces   33.35 µs      │ 41.93 µs      │ 36.74 µs      │ 36.01 µs      │ 100     │ 100
   │                                          29.4 GiB/s    │ 23.39 GiB/s   │ 26.69 GiB/s   │ 27.23 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            4.5 KiB     │ 4.5 KiB       │ 4.5 KiB       │ 4.5 KiB       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            4.5 KiB     │ 4.5 KiB       │ 4.5 KiB       │ 4.5 KiB       │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            256 B       │ 256 B         │ 256 B         │ 256 B         │         │
   ├─ 16.00 MB data splitted into 16 pieces   1.034 ms      │ 2.273 ms      │ 1.094 ms      │ 1.173 ms      │ 100     │ 100
   │                                          16.04 GiB/s   │ 7.301 GiB/s   │ 15.17 GiB/s   │ 14.14 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            16 B        │ 16 B          │ 16 B          │ 16 B          │         │
   ├─ 16.00 MB data splitted into 32 pieces   941.6 µs      │ 1.658 ms      │ 1.009 ms      │ 1.027 ms      │ 100     │ 100
   │                                          17.11 GiB/s   │ 9.713 GiB/s   │ 15.96 GiB/s   │ 15.67 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            512 KiB     │ 512 KiB       │ 512 KiB       │ 512 KiB       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            512 KiB     │ 512 KiB       │ 512 KiB       │ 512 KiB       │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            32 B        │ 32 B          │ 32 B          │ 32 B          │         │
   ├─ 16.00 MB data splitted into 64 pieces   965.5 µs      │ 1.552 ms      │ 1.009 ms      │ 1.021 ms      │ 100     │ 100
   │                                          16.43 GiB/s   │ 10.22 GiB/s   │ 15.72 GiB/s   │ 15.52 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            256.1 KiB   │ 256.1 KiB     │ 256.1 KiB     │ 256.1 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            256.1 KiB   │ 256.1 KiB     │ 256.1 KiB     │ 256.1 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            64 B        │ 64 B          │ 64 B          │ 64 B          │         │
   ├─ 16.00 MB data splitted into 128 pieces  923.9 µs      │ 1.534 ms      │ 940.6 µs      │ 956.9 µs      │ 100     │ 100
   │                                          17.04 GiB/s   │ 10.26 GiB/s   │ 16.74 GiB/s   │ 16.45 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            128.2 KiB   │ 128.2 KiB     │ 128.2 KiB     │ 128.2 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            128.2 KiB   │ 128.2 KiB     │ 128.2 KiB     │ 128.2 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            128 B       │ 128 B         │ 128 B         │ 128 B         │         │
   ├─ 16.00 MB data splitted into 256 pieces  926 µs        │ 1.843 ms      │ 946.9 µs      │ 990.5 µs      │ 100     │ 100
   │                                          16.93 GiB/s   │ 8.507 GiB/s   │ 16.56 GiB/s   │ 15.83 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            64.5 KiB    │ 64.5 KiB      │ 64.5 KiB      │ 64.5 KiB      │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            64.5 KiB    │ 64.5 KiB      │ 64.5 KiB      │ 64.5 KiB      │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            256 B       │ 256 B         │ 256 B         │ 256 B         │         │
   ├─ 32.00 MB data splitted into 16 pieces   2.665 ms      │ 3.902 ms      │ 2.914 ms      │ 2.927 ms      │ 100     │ 100
   │                                          12.45 GiB/s   │ 8.507 GiB/s   │ 11.39 GiB/s   │ 11.34 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            16 B        │ 16 B          │ 16 B          │ 16 B          │         │
   ├─ 32.00 MB data splitted into 32 pieces   1.934 ms      │ 2.755 ms      │ 2.049 ms      │ 2.076 ms      │ 100     │ 100
   │                                          16.66 GiB/s   │ 11.69 GiB/s   │ 15.72 GiB/s   │ 15.52 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            32 B        │ 32 B          │ 32 B          │ 32 B          │         │
   ├─ 32.00 MB data splitted into 64 pieces   1.845 ms      │ 2.32 ms       │ 1.94 ms       │ 1.944 ms      │ 100     │ 100
   │                                          17.19 GiB/s   │ 13.67 GiB/s   │ 16.35 GiB/s   │ 16.31 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            64 B        │ 64 B          │ 64 B          │ 64 B          │         │
   ├─ 32.00 MB data splitted into 128 pieces  1.847 ms      │ 2.436 ms      │ 1.914 ms      │ 1.942 ms      │ 100     │ 100
   │                                          17.05 GiB/s   │ 12.92 GiB/s   │ 16.45 GiB/s   │ 16.21 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            256.2 KiB   │ 256.2 KiB     │ 256.2 KiB     │ 256.2 KiB     │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2             │         │
   │                                            256.2 KiB   │ 256.2 KiB     │ 256.2 KiB     │ 256.2 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1             │         │
   │                                            128 B       │ 128 B         │ 128 B         │ 128 B         │         │
   ╰─ 32.00 MB data splitted into 256 pieces  1.777 ms      │ 2.306 ms      │ 1.834 ms      │ 1.841 ms      │ 100     │ 100
                                              17.65 GiB/s   │ 13.6 GiB/s    │ 17.1 GiB/s    │ 17.04 GiB/s   │         │
                                              max alloc:    │               │               │               │         │
                                                2           │ 2             │ 2             │ 2             │         │
                                                128.5 KiB   │ 128.5 KiB     │ 128.5 KiB     │ 128.5 KiB     │         │
                                              alloc:        │               │               │               │         │
                                                2           │ 2             │ 2             │ 2             │         │
                                                128.5 KiB   │ 128.5 KiB     │ 128.5 KiB     │ 128.5 KiB     │         │
                                              dealloc:      │               │               │               │         │
                                                1           │ 1             │ 1             │ 1             │         │
                                                256 B       │ 256 B         │ 256 B         │ 256 B         │         │

# ---------------------------------------------------------------------------------------------------------------------------
# Encoding with `rayon` data-parallelism, also using AVX2 intrinsics for faster vector x scalar multiplication

Timer precision: 23 ns
full_rlnc_encoder                             fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ encode                                                   │               │               │               │         │
   ├─ 1.00 MB data splitted into 16 pieces    185.7 µs      │ 1.688 ms      │ 226.5 µs      │ 277.7 µs      │ 100     │ 100
   │                                          5.584 GiB/s   │ 629.2 MiB/s   │ 4.579 GiB/s   │ 3.736 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 2.68          │         │
   │                                            32 B        │ 32 B          │ 32 B          │ 607.5 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 3.73          │         │
   │                                            64.03 KiB   │ 64.03 KiB     │ 64.03 KiB     │ 64.6 KiB      │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3.07          │         │
   │                                            128 KiB     │ 128 KiB       │ 128 KiB       │ 128 KiB       │         │
   │                                          grow:         │               │               │               │         │
   │                                            0           │ 0             │ 0             │ 0.02          │         │
   │                                            0 B         │ 0 B           │ 0 B           │ 2.56 B        │         │
   ├─ 1.00 MB data splitted into 32 pieces    54.8 µs       │ 169.4 µs      │ 93.37 µs      │ 96.09 µs      │ 100     │ 100
   │                                          18.37 GiB/s   │ 5.942 GiB/s   │ 10.78 GiB/s   │ 10.48 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            64 B        │ 64 B          │ 64 B          │ 94.4 B        │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            32.06 KiB   │ 32.06 KiB     │ 32.06 KiB     │ 32.09 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            64.03 KiB   │ 64.03 KiB     │ 64.03 KiB     │ 64.03 KiB     │         │
   ├─ 1.00 MB data splitted into 64 pieces    54.04 µs      │ 220.4 µs      │ 96.38 µs      │ 102.1 µs      │ 100     │ 100
   │                                          18.35 GiB/s   │ 4.499 GiB/s   │ 10.29 GiB/s   │ 9.706 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.01          │         │
   │                                            128 B       │ 128 B         │ 128 B         │ 143.2 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.01          │         │
   │                                            16.12 KiB   │ 16.12 KiB     │ 16.12 KiB     │ 16.14 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            32.06 KiB   │ 32.06 KiB     │ 32.06 KiB     │ 32.06 KiB     │         │
   ├─ 1.00 MB data splitted into 128 pieces   44.13 µs      │ 667.3 µs      │ 103 µs        │ 107.2 µs      │ 100     │ 100
   │                                          22.3 GiB/s    │ 1.475 GiB/s   │ 9.554 GiB/s   │ 9.175 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            256 B       │ 256 B         │ 256 B         │ 286.4 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            8.25 KiB    │ 8.25 KiB      │ 8.25 KiB      │ 8.28 KiB      │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            16.12 KiB   │ 16.12 KiB     │ 16.12 KiB     │ 16.12 KiB     │         │
   ├─ 1.00 MB data splitted into 256 pieces   45.19 µs      │ 792.5 µs      │ 109.4 µs      │ 132 µs        │ 100     │ 100
   │                                          21.7 GiB/s    │ 1.237 GiB/s   │ 8.959 GiB/s   │ 7.429 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.01          │         │
   │                                            512 B       │ 512 B         │ 512 B         │ 527.2 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.01          │         │
   │                                            4.5 KiB     │ 4.5 KiB       │ 4.5 KiB       │ 4.515 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            8.251 KiB   │ 8.251 KiB     │ 8.251 KiB     │ 8.251 KiB     │         │
   ├─ 16.00 MB data splitted into 16 pieces   4.305 ms      │ 8.177 ms      │ 4.703 ms      │ 4.741 ms      │ 100     │ 100
   │                                          3.855 GiB/s   │ 2.03 GiB/s    │ 3.529 GiB/s   │ 3.501 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            32 B        │ 32 B          │ 32 B          │ 62.4 B        │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   ├─ 16.00 MB data splitted into 32 pieces   2.63 ms       │ 3.415 ms      │ 2.963 ms      │ 2.982 ms      │ 100     │ 100
   │                                          6.126 GiB/s   │ 4.718 GiB/s   │ 5.437 GiB/s   │ 5.403 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            64 B        │ 64 B          │ 64 B          │ 94.4 B        │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            512 KiB     │ 512 KiB       │ 512 KiB       │ 512 KiB       │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   ├─ 16.00 MB data splitted into 64 pieces   1.153 ms      │ 1.697 ms      │ 1.273 ms      │ 1.273 ms      │ 100     │ 100
   │                                          13.76 GiB/s   │ 9.347 GiB/s   │ 12.46 GiB/s   │ 12.45 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.01          │         │
   │                                            128 B       │ 128 B         │ 128 B         │ 143.2 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.01          │         │
   │                                            256.1 KiB   │ 256.1 KiB     │ 256.1 KiB     │ 256.1 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            512 KiB     │ 512 KiB       │ 512 KiB       │ 512 KiB       │         │
   ├─ 16.00 MB data splitted into 128 pieces  743.5 µs      │ 1.294 ms      │ 862 µs        │ 880.4 µs      │ 100     │ 100
   │                                          21.17 GiB/s   │ 12.16 GiB/s   │ 18.26 GiB/s   │ 17.88 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            256 B       │ 256 B         │ 256 B         │ 286.4 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            128.2 KiB   │ 128.2 KiB     │ 128.2 KiB     │ 128.2 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            256.1 KiB   │ 256.1 KiB     │ 256.1 KiB     │ 256.1 KiB     │         │
   ├─ 16.00 MB data splitted into 256 pieces  586.7 µs      │ 1.195 ms      │ 670.4 µs      │ 684.4 µs      │ 100     │ 100
   │                                          26.73 GiB/s   │ 13.12 GiB/s   │ 23.39 GiB/s   │ 22.91 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.01          │         │
   │                                            512 B       │ 512 B         │ 512 B         │ 527.2 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.01          │         │
   │                                            64.5 KiB    │ 64.5 KiB      │ 64.5 KiB      │ 64.51 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            128.2 KiB   │ 128.2 KiB     │ 128.2 KiB     │ 128.2 KiB     │         │
   ├─ 32.00 MB data splitted into 16 pieces   12.81 ms      │ 14.98 ms      │ 13.78 ms      │ 13.75 ms      │ 100     │ 100
   │                                          2.591 GiB/s   │ 2.216 GiB/s   │ 2.409 GiB/s   │ 2.414 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            32 B        │ 32 B          │ 32 B          │ 62.4 B        │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            4 MiB       │ 4 MiB         │ 4 MiB         │ 4 MiB         │         │
   ├─ 32.00 MB data splitted into 32 pieces   8.681 ms      │ 11.02 ms      │ 9.191 ms      │ 9.226 ms      │ 100     │ 100
   │                                          3.712 GiB/s   │ 2.924 GiB/s   │ 3.506 GiB/s   │ 3.492 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            64 B        │ 64 B          │ 64 B          │ 94.4 B        │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   ├─ 32.00 MB data splitted into 64 pieces   4.778 ms      │ 6.972 ms      │ 5.617 ms      │ 5.632 ms      │ 100     │ 100
   │                                          6.642 GiB/s   │ 4.552 GiB/s   │ 5.65 GiB/s    │ 5.635 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.01          │         │
   │                                            128 B       │ 128 B         │ 128 B         │ 143.2 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.01          │         │
   │                                            512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   ├─ 32.00 MB data splitted into 128 pieces  2.21 ms       │ 3.055 ms      │ 2.433 ms      │ 2.444 ms      │ 100     │ 100
   │                                          14.24 GiB/s   │ 10.3 GiB/s    │ 12.94 GiB/s   │ 12.88 GiB/s   │         │
   │                                          max alloc:    │               │               │               │         │
   │                                            1           │ 1             │ 1             │ 1.02          │         │
   │                                            256 B       │ 256 B         │ 256 B         │ 286.4 B       │         │
   │                                          alloc:        │               │               │               │         │
   │                                            2           │ 2             │ 2             │ 2.02          │         │
   │                                            256.2 KiB   │ 256.2 KiB     │ 256.2 KiB     │ 256.2 KiB     │         │
   │                                          dealloc:      │               │               │               │         │
   │                                            3           │ 3             │ 3             │ 3             │         │
   │                                            512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   ╰─ 32.00 MB data splitted into 256 pieces  1.386 ms      │ 2.209 ms      │ 1.629 ms      │ 1.619 ms      │ 100     │ 100
                                              22.62 GiB/s   │ 14.2 GiB/s    │ 19.25 GiB/s   │ 19.37 GiB/s   │         │
                                              max alloc:    │               │               │               │         │
                                                1           │ 1             │ 1             │ 1.01          │         │
                                                512 B       │ 512 B         │ 512 B         │ 527.2 B       │         │
                                              alloc:        │               │               │               │         │
                                                2           │ 2             │ 2             │ 2.01          │         │
                                                128.5 KiB   │ 128.5 KiB     │ 128.5 KiB     │ 128.5 KiB     │         │
                                              dealloc:      │               │               │               │         │
                                                3           │ 3             │ 3             │ 3             │         │
                                                256.2 KiB   │ 256.2 KiB     │ 256.2 KiB     │ 256.2 KiB     │         │
```

#### Full RLNC Recoder

```bash
# Recoding without `rayon` data-parallelism, instead just using AVX2-powered SIMD vector x scalar multiplication

Timer precision: 14 ns
full_rlnc_recoder                                                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ recode                                                                             │               │               │               │         │
   ├─ 1.00 MB data splitted into 16 pieces, recoding with 8 pieces      22.06 µs      │ 37.4 µs       │ 25.77 µs      │ 26.23 µs      │ 100     │ 100
   │                                                                    24.89 GiB/s   │ 14.69 GiB/s   │ 21.32 GiB/s   │ 20.94 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      128 KiB     │ 128 KiB       │ 128 KiB       │ 128 KiB       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      128 KiB     │ 128 KiB       │ 128 KiB       │ 128 KiB       │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      64.03 KiB   │ 64.03 KiB     │ 64.03 KiB     │ 64.03 KiB     │         │
   ├─ 1.00 MB data splitted into 32 pieces, recoding with 16 pieces     19.39 µs      │ 27.08 µs      │ 22.65 µs      │ 22.32 µs      │ 100     │ 100
   │                                                                    26.77 GiB/s   │ 19.17 GiB/s   │ 22.92 GiB/s   │ 23.26 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      64.09 KiB   │ 64.09 KiB     │ 64.09 KiB     │ 64.09 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      64.09 KiB   │ 64.09 KiB     │ 64.09 KiB     │ 64.09 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      32.06 KiB   │ 32.06 KiB     │ 32.06 KiB     │ 32.06 KiB     │         │
   ├─ 1.00 MB data splitted into 64 pieces, recoding with 32 pieces     17.73 µs      │ 22.41 µs      │ 18.54 µs      │ 18.53 µs      │ 100     │ 100
   │                                                                    28.5 GiB/s    │ 22.55 GiB/s   │ 27.26 GiB/s   │ 27.28 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      32.18 KiB   │ 32.18 KiB     │ 32.18 KiB     │ 32.18 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      32.18 KiB   │ 32.18 KiB     │ 32.18 KiB     │ 32.18 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      16.12 KiB   │ 16.12 KiB     │ 16.12 KiB     │ 16.12 KiB     │         │
   ├─ 1.00 MB data splitted into 128 pieces, recoding with 64 pieces    23.28 µs      │ 38.67 µs      │ 24.12 µs      │ 24.57 µs      │ 100     │ 100
   │                                                                    21.63 GiB/s   │ 13.02 GiB/s   │ 20.87 GiB/s   │ 20.49 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      16.37 KiB   │ 16.37 KiB     │ 16.37 KiB     │ 16.37 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      16.37 KiB   │ 16.37 KiB     │ 16.37 KiB     │ 16.37 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      8.25 KiB    │ 8.25 KiB      │ 8.25 KiB      │ 8.25 KiB      │         │
   ├─ 1.00 MB data splitted into 256 pieces, recoding with 128 pieces   44.62 µs      │ 60.58 µs      │ 46.58 µs      │ 47.26 µs      │ 100     │ 100
   │                                                                    11.71 GiB/s   │ 8.631 GiB/s   │ 11.22 GiB/s   │ 11.06 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      8.751 KiB   │ 8.751 KiB     │ 8.751 KiB     │ 8.751 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      8.751 KiB   │ 8.751 KiB     │ 8.751 KiB     │ 8.751 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      4.5 KiB     │ 4.5 KiB       │ 4.5 KiB       │ 4.5 KiB       │         │
   ├─ 16.00 MB data splitted into 16 pieces, recoding with 8 pieces     544.1 µs      │ 852.5 µs      │ 609.4 µs      │ 609.9 µs      │ 100     │ 100
   │                                                                    16.15 GiB/s   │ 10.3 GiB/s    │ 14.42 GiB/s   │ 14.41 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   ├─ 16.00 MB data splitted into 32 pieces, recoding with 16 pieces    471.7 µs      │ 616.5 µs      │ 519.6 µs      │ 522.2 µs      │ 100     │ 100
   │                                                                    17.59 GiB/s   │ 13.46 GiB/s   │ 15.97 GiB/s   │ 15.89 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      512 KiB     │ 512 KiB       │ 512 KiB       │ 512 KiB       │         │
   ├─ 16.00 MB data splitted into 64 pieces, recoding with 32 pieces    398.5 µs      │ 587.5 µs      │ 464.5 µs      │ 466.3 µs      │ 100     │ 100
   │                                                                    20.22 GiB/s   │ 13.71 GiB/s   │ 17.34 GiB/s   │ 17.28 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      256.1 KiB   │ 256.1 KiB     │ 256.1 KiB     │ 256.1 KiB     │         │
   ├─ 16.00 MB data splitted into 128 pieces, recoding with 64 pieces   427.6 µs      │ 567.9 µs      │ 465.8 µs      │ 467.7 µs      │ 100     │ 100
   │                                                                    18.57 GiB/s   │ 13.98 GiB/s   │ 17.04 GiB/s   │ 16.97 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      256.3 KiB   │ 256.3 KiB     │ 256.3 KiB     │ 256.3 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      256.3 KiB   │ 256.3 KiB     │ 256.3 KiB     │ 256.3 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      128.2 KiB   │ 128.2 KiB     │ 128.2 KiB     │ 128.2 KiB     │         │
   ├─ 16.00 MB data splitted into 256 pieces, recoding with 128 pieces  423.1 µs      │ 718.2 µs      │ 490.6 µs      │ 494.2 µs      │ 100     │ 100
   │                                                                    18.67 GiB/s   │ 11 GiB/s      │ 16.1 GiB/s    │ 15.99 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      128.7 KiB   │ 128.7 KiB     │ 128.7 KiB     │ 128.7 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      128.7 KiB   │ 128.7 KiB     │ 128.7 KiB     │ 128.7 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      64.5 KiB    │ 64.5 KiB      │ 64.5 KiB      │ 64.5 KiB      │         │
   ├─ 32.00 MB data splitted into 16 pieces, recoding with 8 pieces     1.234 ms      │ 1.604 ms      │ 1.371 ms      │ 1.374 ms      │ 100     │ 100
   │                                                                    14.23 GiB/s   │ 10.95 GiB/s   │ 12.81 GiB/s   │ 12.78 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      4 MiB       │ 4 MiB         │ 4 MiB         │ 4 MiB         │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      4 MiB       │ 4 MiB         │ 4 MiB         │ 4 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   ├─ 32.00 MB data splitted into 32 pieces, recoding with 16 pieces    1.112 ms      │ 1.525 ms      │ 1.192 ms      │ 1.194 ms      │ 100     │ 100
   │                                                                    14.91 GiB/s   │ 10.88 GiB/s   │ 13.92 GiB/s   │ 13.89 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   ├─ 32.00 MB data splitted into 64 pieces, recoding with 32 pieces    942.5 µs      │ 1.109 ms      │ 1.019 ms      │ 1.017 ms      │ 100     │ 100
   │                                                                    17.09 GiB/s   │ 14.51 GiB/s   │ 15.81 GiB/s   │ 15.83 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      512.1 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.1 KiB     │         │
   ├─ 32.00 MB data splitted into 128 pieces, recoding with 64 pieces   935.7 µs      │ 1.266 ms      │ 990.8 µs      │ 994.9 µs      │ 100     │ 100
   │                                                                    16.96 GiB/s   │ 12.53 GiB/s   │ 16.02 GiB/s   │ 15.95 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      512.3 KiB   │ 512.3 KiB     │ 512.3 KiB     │ 512.3 KiB     │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4             │         │
   │                                                                      512.3 KiB   │ 512.3 KiB     │ 512.3 KiB     │ 512.3 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      3           │ 3             │ 3             │ 3             │         │
   │                                                                      256.2 KiB   │ 256.2 KiB     │ 256.2 KiB     │ 256.2 KiB     │         │
   ╰─ 32.00 MB data splitted into 256 pieces, recoding with 128 pieces  969.1 µs      │ 1.437 ms      │ 1.006 ms      │ 1.015 ms      │ 100     │ 100
                                                                        16.28 GiB/s   │ 10.97 GiB/s   │ 15.67 GiB/s   │ 15.53 GiB/s   │         │
                                                                        max alloc:    │               │               │               │         │
                                                                          4           │ 4             │ 4             │ 4             │         │
                                                                          256.7 KiB   │ 256.7 KiB     │ 256.7 KiB     │ 256.7 KiB     │         │
                                                                        alloc:        │               │               │               │         │
                                                                          4           │ 4             │ 4             │ 4             │         │
                                                                          256.7 KiB   │ 256.7 KiB     │ 256.7 KiB     │ 256.7 KiB     │         │
                                                                        dealloc:      │               │               │               │         │
                                                                          3           │ 3             │ 3             │ 3             │         │
                                                                          128.5 KiB   │ 128.5 KiB     │ 128.5 KiB     │ 128.5 KiB     │         │

# ---------------------------------------------------------------------------------------------------------------------------
# Recoding with `rayon` data-parallelism, also using AVX2 intrinsics for faster vector x scalar multiplication

Timer precision: 25 ns
full_rlnc_recoder                                                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ recode                                                                             │               │               │               │         │
   ├─ 1.00 MB data splitted into 16 pieces, recoding with 8 pieces      79.71 µs      │ 314.3 µs      │ 165.7 µs      │ 169.8 µs      │ 100     │ 100
   │                                                                    6.892 GiB/s   │ 1.747 GiB/s   │ 3.314 GiB/s   │ 3.234 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.01          │         │
   │                                                                      48 B        │ 48 B          │ 48 B          │ 63.2 B        │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.01          │         │
   │                                                                      128 KiB     │ 128 KiB       │ 128 KiB       │ 128 KiB       │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      192 KiB     │ 192 KiB       │ 192 KiB       │ 192 KiB       │         │
   ├─ 1.00 MB data splitted into 32 pieces, recoding with 16 pieces     93.73 µs      │ 452.7 µs      │ 248.6 µs      │ 264.1 µs      │ 100     │ 100
   │                                                                    5.54 GiB/s    │ 1.147 GiB/s   │ 2.088 GiB/s   │ 1.965 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      96 B        │ 96 B          │ 96 B          │ 126.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      64.09 KiB   │ 64.09 KiB     │ 64.09 KiB     │ 64.12 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      96.06 KiB   │ 96.06 KiB     │ 96.06 KiB     │ 96.06 KiB     │         │
   ├─ 1.00 MB data splitted into 64 pieces, recoding with 32 pieces     91.63 µs      │ 789.7 µs      │ 211.5 µs      │ 234.4 µs      │ 100     │ 100
   │                                                                    5.517 GiB/s   │ 655.5 MiB/s   │ 2.39 GiB/s    │ 2.156 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.01          │         │
   │                                                                      192 B       │ 192 B         │ 192 B         │ 207.2 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.01          │         │
   │                                                                      32.18 KiB   │ 32.18 KiB     │ 32.18 KiB     │ 32.2 KiB      │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      48.12 KiB   │ 48.12 KiB     │ 48.12 KiB     │ 48.12 KiB     │         │
   ├─ 1.00 MB data splitted into 128 pieces, recoding with 64 pieces    57.64 µs      │ 179.8 µs      │ 102 µs        │ 103.6 µs      │ 100     │ 100
   │                                                                    8.738 GiB/s   │ 2.8 GiB/s     │ 4.934 GiB/s   │ 4.86 GiB/s    │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      384 B       │ 384 B         │ 384 B         │ 414.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      16.37 KiB   │ 16.37 KiB     │ 16.37 KiB     │ 16.4 KiB      │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      24.25 KiB   │ 24.25 KiB     │ 24.25 KiB     │ 24.25 KiB     │         │
   ├─ 1.00 MB data splitted into 256 pieces, recoding with 128 pieces   77.32 µs      │ 240.6 µs      │ 118.5 µs      │ 123.4 µs      │ 100     │ 100
   │                                                                    6.763 GiB/s   │ 2.172 GiB/s   │ 4.409 GiB/s   │ 4.235 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.01          │         │
   │                                                                      768 B       │ 768 B         │ 768 B         │ 783.2 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.01          │         │
   │                                                                      8.751 KiB   │ 8.751 KiB     │ 8.751 KiB     │ 8.766 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      12.5 KiB    │ 12.5 KiB      │ 12.5 KiB      │ 12.5 KiB      │         │
   ├─ 16.00 MB data splitted into 16 pieces, recoding with 8 pieces     1.964 ms      │ 3.789 ms      │ 2.371 ms      │ 2.424 ms      │ 100     │ 100
   │                                                                    4.473 GiB/s   │ 2.319 GiB/s   │ 3.706 GiB/s   │ 3.624 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.01          │         │
   │                                                                      48 B        │ 48 B          │ 48 B          │ 63.2 B        │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.01          │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      3 MiB       │ 3 MiB         │ 3 MiB         │ 3 MiB         │         │
   ├─ 16.00 MB data splitted into 32 pieces, recoding with 16 pieces    1.716 ms      │ 2.756 ms      │ 1.914 ms      │ 1.958 ms      │ 100     │ 100
   │                                                                    4.836 GiB/s   │ 3.011 GiB/s   │ 4.337 GiB/s   │ 4.238 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      96 B        │ 96 B          │ 96 B          │ 126.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      1.5 MiB     │ 1.5 MiB       │ 1.5 MiB       │ 1.5 MiB       │         │
   ├─ 16.00 MB data splitted into 64 pieces, recoding with 32 pieces    890.7 µs      │ 1.648 ms      │ 1.062 ms      │ 1.089 ms      │ 100     │ 100
   │                                                                    9.046 GiB/s   │ 4.887 GiB/s   │ 7.581 GiB/s   │ 7.399 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      3           │ 2             │ 2             │ 2.02          │         │
   │                                                                      1.671 KiB   │ 192 B         │ 192 B         │ 222.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      5           │ 4             │ 4             │ 4.02          │         │
   │                                                                      513.6 KiB   │ 512.1 KiB     │ 512.1 KiB     │ 512.2 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      768.1 KiB   │ 768.1 KiB     │ 768.1 KiB     │ 768.1 KiB     │         │
   ├─ 16.00 MB data splitted into 128 pieces, recoding with 64 pieces   609.9 µs      │ 1.144 ms      │ 763.9 µs      │ 787.7 µs      │ 100     │ 100
   │                                                                    13.02 GiB/s   │ 6.939 GiB/s   │ 10.39 GiB/s   │ 10.08 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      384 B       │ 384 B         │ 384 B         │ 414.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      256.3 KiB   │ 256.3 KiB     │ 256.3 KiB     │ 256.4 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      384.2 KiB   │ 384.2 KiB     │ 384.2 KiB     │ 384.2 KiB     │         │
   ├─ 16.00 MB data splitted into 256 pieces, recoding with 128 pieces  593.1 µs      │ 1.428 ms      │ 716.2 µs      │ 770 µs        │ 100     │ 100
   │                                                                    13.32 GiB/s   │ 5.532 GiB/s   │ 11.03 GiB/s   │ 10.26 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.01          │         │
   │                                                                      768 B       │ 768 B         │ 768 B         │ 783.2 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.01          │         │
   │                                                                      128.7 KiB   │ 128.7 KiB     │ 128.7 KiB     │ 128.7 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      192.5 KiB   │ 192.5 KiB     │ 192.5 KiB     │ 192.5 KiB     │         │
   ├─ 32.00 MB data splitted into 16 pieces, recoding with 8 pieces     5.115 ms      │ 6.58 ms       │ 5.597 ms      │ 5.633 ms      │ 100     │ 100
   │                                                                    3.436 GiB/s   │ 2.671 GiB/s   │ 3.14 GiB/s    │ 3.12 GiB/s    │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      48 B        │ 48 B          │ 48 B          │ 78.4 B        │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      4 MiB       │ 4 MiB         │ 4 MiB         │ 4 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      6 MiB       │ 6 MiB         │ 6 MiB         │ 6 MiB         │         │
   ├─ 32.00 MB data splitted into 32 pieces, recoding with 16 pieces    4.446 ms      │ 5.619 ms      │ 4.831 ms      │ 4.889 ms      │ 100     │ 100
   │                                                                    3.733 GiB/s   │ 2.954 GiB/s   │ 3.436 GiB/s   │ 3.395 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      96 B        │ 96 B          │ 96 B          │ 126.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      2 MiB       │ 2 MiB         │ 2 MiB         │ 2 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      3 MiB       │ 3 MiB         │ 3 MiB         │ 3 MiB         │         │
   ├─ 32.00 MB data splitted into 64 pieces, recoding with 32 pieces    2.965 ms      │ 4.632 ms      │ 3.2 ms        │ 3.27 ms       │ 100     │ 100
   │                                                                    5.433 GiB/s   │ 3.478 GiB/s   │ 5.035 GiB/s   │ 4.927 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      192 B       │ 192 B         │ 192 B         │ 222.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      1 MiB       │ 1 MiB         │ 1 MiB         │ 1 MiB         │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      1.5 MiB     │ 1.5 MiB       │ 1.5 MiB       │ 1.5 MiB       │         │
   ├─ 32.00 MB data splitted into 128 pieces, recoding with 64 pieces   1.337 ms      │ 2.332 ms      │ 1.614 ms      │ 1.661 ms      │ 100     │ 100
   │                                                                    11.86 GiB/s   │ 6.807 GiB/s   │ 9.834 GiB/s   │ 9.557 GiB/s   │         │
   │                                                                    max alloc:    │               │               │               │         │
   │                                                                      2           │ 2             │ 2             │ 2.02          │         │
   │                                                                      384 B       │ 384 B         │ 384 B         │ 414.4 B       │         │
   │                                                                    alloc:        │               │               │               │         │
   │                                                                      4           │ 4             │ 4             │ 4.02          │         │
   │                                                                      512.3 KiB   │ 512.3 KiB     │ 512.3 KiB     │ 512.4 KiB     │         │
   │                                                                    dealloc:      │               │               │               │         │
   │                                                                      5           │ 5             │ 5             │ 5             │         │
   │                                                                      768.2 KiB   │ 768.2 KiB     │ 768.2 KiB     │ 768.2 KiB     │         │
   ╰─ 32.00 MB data splitted into 256 pieces, recoding with 128 pieces  990.8 µs      │ 2.347 ms      │ 1.249 ms      │ 1.286 ms      │ 100     │ 100
                                                                        15.92 GiB/s   │ 6.722 GiB/s   │ 12.63 GiB/s   │ 12.26 GiB/s   │         │
                                                                        max alloc:    │               │               │               │         │
                                                                          2           │ 2             │ 2             │ 2.01          │         │
                                                                          768 B       │ 768 B         │ 768 B         │ 783.2 B       │         │
                                                                        alloc:        │               │               │               │         │
                                                                          4           │ 4             │ 4             │ 4.01          │         │
                                                                          256.7 KiB   │ 256.7 KiB     │ 256.7 KiB     │ 256.7 KiB     │         │
                                                                        dealloc:      │               │               │               │         │
                                                                          5           │ 5             │ 5             │ 5             │         │
                                                                          384.5 KiB   │ 384.5 KiB     │ 384.5 KiB     │ 384.5 KiB     │         │
```

#### Full RLNC Decoder

```bash
# Decoding with AVX2-powered SIMD vector x scalar multiplication

Timer precision: 18 ns
full_rlnc_decoder                             fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ decode                                                   │               │               │               │         │
   ├─ 1.00 MB data splitted into 16 pieces    600 µs        │ 1.041 ms      │ 615.2 µs      │ 622.9 µs      │ 100     │ 100
   │                                          1.627 GiB/s   │ 960.8 MiB/s   │ 1.587 GiB/s   │ 1.568 GiB/s   │         │
   ├─ 1.00 MB data splitted into 32 pieces    1.18 ms       │ 1.629 ms      │ 1.201 ms      │ 1.207 ms      │ 100     │ 100
   │                                          847.6 MiB/s   │ 614.1 MiB/s   │ 833 MiB/s     │ 829.2 MiB/s   │         │
   ├─ 1.00 MB data splitted into 64 pieces    2.267 ms      │ 2.404 ms      │ 2.299 ms      │ 2.302 ms      │ 100     │ 100
   │                                          442.7 MiB/s   │ 417.5 MiB/s   │ 436.6 MiB/s   │ 436 MiB/s     │         │
   ├─ 1.00 MB data splitted into 128 pieces   5.296 ms      │ 5.583 ms      │ 5.333 ms      │ 5.338 ms      │ 100     │ 100
   │                                          191.7 MiB/s   │ 181.9 MiB/s   │ 190.4 MiB/s   │ 190.2 MiB/s   │         │
   ├─ 1.00 MB data splitted into 256 pieces   15.27 ms      │ 16.21 ms      │ 15.55 ms      │ 15.55 ms      │ 100     │ 100
   │                                          69.59 MiB/s   │ 65.54 MiB/s   │ 68.33 MiB/s   │ 68.32 MiB/s   │         │
   ├─ 16.00 MB data splitted into 16 pieces   16.4 ms       │ 20.56 ms      │ 16.97 ms      │ 17.08 ms      │ 100     │ 100
   │                                          975.5 MiB/s   │ 778.1 MiB/s   │ 942.4 MiB/s   │ 936.5 MiB/s   │         │
   ├─ 16.00 MB data splitted into 32 pieces   27.24 ms      │ 30.95 ms      │ 28.12 ms      │ 28.26 ms      │ 100     │ 100
   │                                          587.3 MiB/s   │ 516.8 MiB/s   │ 568.9 MiB/s   │ 566 MiB/s     │         │
   ├─ 16.00 MB data splitted into 64 pieces   49.54 ms      │ 62.7 ms       │ 49.74 ms      │ 50.23 ms      │ 100     │ 100
   │                                          323 MiB/s     │ 255.2 MiB/s   │ 321.6 MiB/s   │ 318.5 MiB/s   │         │
   ├─ 16.00 MB data splitted into 128 pieces  98.43 ms      │ 102.1 ms      │ 98.98 ms      │ 99.32 ms      │ 100     │ 100
   │                                          162.6 MiB/s   │ 156.7 MiB/s   │ 161.8 MiB/s   │ 161.2 MiB/s   │         │
   ├─ 16.00 MB data splitted into 256 pieces  201.8 ms      │ 209.3 ms      │ 202.7 ms      │ 203 ms        │ 100     │ 100
   │                                          79.58 MiB/s   │ 76.73 MiB/s   │ 79.21 MiB/s   │ 79.11 MiB/s   │         │
   ├─ 32.00 MB data splitted into 16 pieces   46.33 ms      │ 49.04 ms      │ 46.51 ms      │ 46.7 ms       │ 100     │ 100
   │                                          690.5 MiB/s   │ 652.4 MiB/s   │ 687.8 MiB/s   │ 685.1 MiB/s   │         │
   ├─ 32.00 MB data splitted into 32 pieces   78.74 ms      │ 81.78 ms      │ 79.06 ms      │ 79.29 ms      │ 100     │ 100
   │                                          406.4 MiB/s   │ 391.2 MiB/s   │ 404.7 MiB/s   │ 403.5 MiB/s   │         │
   ├─ 32.00 MB data splitted into 64 pieces   132.4 ms      │ 137.6 ms      │ 132.9 ms      │ 133.3 ms      │ 100     │ 100
   │                                          241.5 MiB/s   │ 232.4 MiB/s   │ 240.7 MiB/s   │ 239.9 MiB/s   │         │
   ├─ 32.00 MB data splitted into 128 pieces  241.9 ms      │ 249.3 ms      │ 243.1 ms      │ 243.6 ms      │ 100     │ 100
   │                                          132.3 MiB/s   │ 128.4 MiB/s   │ 131.6 MiB/s   │ 131.4 MiB/s   │         │
   ╰─ 32.00 MB data splitted into 256 pieces  476 ms        │ 485.5 ms      │ 479.1 ms      │ 479.4 ms      │ 100     │ 100
                                              67.35 MiB/s   │ 66.03 MiB/s   │ 66.9 MiB/s    │ 66.87 MiB/s   │         │
```

</details>

## Usage

To use `rlnc` library crate in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
rlnc = "=0.8.1"                                      # On x86 target, it offers AVX2 and SSSE3 optimization for fast encoding, recoding and decoding.
# or
rlnc = { version = "=0.8.1", features = "parallel" } # Uses `rayon`-based data-parallelism for fast encoding and recoding. Prefer it over `default` if input blob gets split into *substantially* large many chunks. Note, this feature, doesn't yet parallelize RLNC decoding.

rand = { version = "=0.9.1" } # Required for random number generation
```

### Full RLNC Workflow Example

I maintain an example demonstrating the Full RLNC workflow:

- Encoding original data into coded pieces.
- Recoding to generate new pieces from received coded pieces. It simulates a relay node.
- Finally decoding all received pieces to recover the original data.

> [!NOTE]
> New recoded pieces could be either useful or not for the Decoder, based on Recoder input coded pieces i.e. from which they are recoded and whether they have already been seen by Decoder or not.

See [full_rlnc.rs](./examples/full_rlnc.rs) example program. Run the program with `$ make example`.

<details>
<summary>Click to view detailed example run output 👇</summary>

```bash
Initialized Encoder with 10240 bytes of data, split into 32 pieces, each of 321 bytes. Each coded piece will be of 353 bytes.
Initializing Decoder, expecting 32 original pieces of 321 bytes each.

Sender generating 16 initial coded pieces...
  Decoded direct piece 1: Useful.
  Decoded direct piece 2: Useful.
  Decoded direct piece 3: Useful.
  Decoded direct piece 4: Useful.
  Decoded direct piece 5: Useful.
  Decoded direct piece 6: Useful.
  Decoded direct piece 7: Useful.
  Decoded direct piece 8: Useful.
  Decoded direct piece 9: Useful.
  Decoded direct piece 10: Useful.
  Decoded direct piece 11: Useful.
  Decoded direct piece 12: Useful.
  Decoded direct piece 13: Useful.
  Decoded direct piece 14: Useful.
  Decoded direct piece 15: Useful.
  Decoded direct piece 16: Useful.

Initializing Recoder with 5648 bytes of received coded pieces.

Recoder active. Generating recoded pieces...
  Decoded recoded piece 1: Not useful.
  Decoded recoded piece 2: Not useful.
  Decoded recoded piece 3: Not useful.
  Decoded recoded piece 4: Not useful.
  Decoded recoded piece 5: Not useful.
  Decoded recoded piece 6: Not useful.
  Decoded recoded piece 7: Not useful.
  Decoded recoded piece 8: Not useful.
  Decoded recoded piece 9: Not useful.
  Decoded recoded piece 10: Not useful.
  Decoded recoded piece 11: Not useful.
  Decoded recoded piece 12: Not useful.
  Decoded recoded piece 13: Not useful.
  Decoded recoded piece 14: Not useful.
  Decoded recoded piece 15: Not useful.
  Decoded recoded piece 16: Not useful.
  Decoded recoded piece 17: Not useful.
  Decoded recoded piece 18: Not useful.
  Decoded recoded piece 19: Not useful.
  Decoded recoded piece 20: Not useful.
  Decoded recoded piece 21: Not useful.
  Decoded recoded piece 22: Not useful.
  Decoded recoded piece 23: Not useful.
  Decoded recoded piece 24: Not useful.
  Decoded recoded piece 25: Not useful.
  Decoded recoded piece 26: Not useful.
  Decoded recoded piece 27: Not useful.
  Decoded recoded piece 28: Not useful.
  Decoded recoded piece 29: Not useful.
  Decoded recoded piece 30: Not useful.
  Decoded recoded piece 31: Not useful.
  Decoded recoded piece 32: Not useful.
  Decoded recoded piece 33: Not useful.
  Decoded recoded piece 34: Not useful.
  Decoded recoded piece 35: Not useful.
  Decoded recoded piece 36: Not useful.
  Decoded recoded piece 37: Not useful.
  Decoded recoded piece 38: Not useful.
  Decoded recoded piece 39: Not useful.
  Decoded recoded piece 40: Not useful.
  Decoded recoded piece 41: Not useful.
  Decoded recoded piece 42: Not useful.
  Decoded recoded piece 43: Not useful.
  Decoded recoded piece 44: Not useful.
  Decoded recoded piece 45: Not useful.
  Decoded recoded piece 46: Not useful.
  Decoded recoded piece 47: Not useful.
  Decoded recoded piece 48: Not useful.
  Decoded recoded piece 49: Not useful.
  Decoded recoded piece 50: Not useful.
  Decoded recoded piece 51: Not useful.
  Decoded recoded piece 52: Not useful.
  Decoded recoded piece 53: Not useful.
  Decoded recoded piece 54: Not useful.
  Decoded recoded piece 55: Not useful.
  Decoded recoded piece 56: Not useful.
  Decoded recoded piece 57: Not useful.
  Decoded recoded piece 58: Not useful.
  Decoded recoded piece 59: Not useful.
  Decoded recoded piece 60: Not useful.
  Decoded recoded piece 61: Not useful.
  Decoded recoded piece 62: Not useful.
  Decoded recoded piece 63: Not useful.
  Decoded recoded piece 64: Not useful.

Initializing a new Recoder with 5648 bytes of received coded pieces.
  Decoded recoded piece 1: Useful.
  Decoded recoded piece 2: Useful.
  Decoded recoded piece 3: Useful.
  Decoded recoded piece 4: Useful.
  Decoded recoded piece 5: Useful.
  Decoded recoded piece 6: Useful.
  Decoded recoded piece 7: Useful.
  Decoded recoded piece 8: Useful.

Still need more pieces. Generating direct piece 17 from encoder...
  Decoded direct piece 17: Useful.

Still need more pieces. Generating direct piece 18 from encoder...
  Decoded direct piece 18: Useful.

Still need more pieces. Generating direct piece 19 from encoder...
  Decoded direct piece 19: Useful.

Still need more pieces. Generating direct piece 20 from encoder...
  Decoded direct piece 20: Useful.

Still need more pieces. Generating direct piece 21 from encoder...
  Decoded direct piece 21: Useful.

Still need more pieces. Generating direct piece 22 from encoder...
  Decoded direct piece 22: Useful.

Still need more pieces. Generating direct piece 23 from encoder...
  Decoded direct piece 23: Useful.

Still need more pieces. Generating direct piece 24 from encoder...
  Decoded direct piece 24: Useful.

Retrieving decoded data...

RLNC workflow completed successfully! Original data matches decoded data.
```

</details>
