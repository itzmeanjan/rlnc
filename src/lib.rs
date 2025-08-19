//! # rlnc: Random Linear Network Coding
//!
//! `rlnc` is a Rust library that provides an implementation of Random Linear Network Coding (RLNC)
//! over finite field GF(2^8). Data owner creates encoded (i.e. erasure-coded) pieces by computing
//! random linear combinations of original data split into smaller pieces using random sampled coefficients.
//! These encoded pieces, tagged with coefficient vectors, often called coded pieces, are forwarded to peers.
//! Intermediate nodes in the network can combine any number of coded pieces to create new coded pieces, without
//! ever decoding to original data. Receiving nodes decode the original data by collecting enough linearly independent
//! combinations to solve the linear system of equations. RLNC enhances network throughput, robustness, and efficiency,
//! particularly in lossy or dynamic networks. It can be used in applications like video streaming, distributed storage,
//! and satellite communications, improving reliability and reducing latency.
//!
//! ## How it Works
//!
//! At its core, RLNC works by combining original data pieces into new "coded pieces"
//! using random linear combinations over GF(2^8).
//!
//! The main components of this library are:
//!
//! -   **`Encoder`**: Takes the original data, splits it into fixed-size pieces,
//!     and generates new coded pieces by applying random linear combinations.
//!     Each coded piece includes a coding vector and the linearly combined data.
//!     The encoder handles necessary padding and a boundary marker to ensure
//!     correct decoding and data recovery.
//!
//! -   **`Recoder`**: A crucial feature of network coding. A recoder takes
//!     already coded pieces as input and generates *new* coded pieces from them.
//!     This allows intermediate nodes in a network to re-encode data without
//!     first decoding it to the original form, significantly improving
//!     throughput and robustness in complex network topologies.
//!
//! -   **`Decoder`**: Receives coded pieces and attempts to reconstruct the
//!     original data. It uses Gaussian elimination to maintain a matrix of
//!     received pieces. As soon as enough linearly independent pieces are received,
//!     it can reconstruct the original data, regardless of which specific pieces were lost or received.
//!
//! ## Features
//!
//! -   **Flexible data handling**: Supports arbitrary byte lengths for input
//!     data, with internal padding and boundary markers for robust decoding.
//! -   **Error Handling**: Comprehensive `RLNCError` enum for various failure scenarios.
//!
//! ## Example Usage
//!
//! A typical workflow involves creating an `Encoder` with your original data,
//! generating coded pieces, sending them across a network (potentially through `Recoder`s),
//! and finally, using a `Decoder` to reconstruct the original data.
//!
//! ```rust
//! use rand::Rng;
//! use rlnc::{
//!     RLNCError,
//!     full::{decoder::Decoder, encoder::Encoder},
//! };
//!
//! let mut rng = rand::rng();
//!
//! // 1. Define original data parameters
//! let original_data_len = 10 * 1024; // 10 KB
//! let piece_count = 32;              // Data will be split into 32 pieces
//! let original_data: Vec<u8> = (0..original_data_len).map(|_| rng.random()).collect();
//! let original_data_copy = original_data.clone();
//!
//! // 2. Initialize the Encoder
//! let encoder = Encoder::new(original_data, piece_count).expect("Failed to create RLNC encoder");
//!
//! // 3. Initialize the Decoder
//! let mut decoder = Decoder::new(encoder.get_piece_byte_len(), encoder.get_piece_count()).expect("Failed to create RLNC decoder");
//!
//! // 4. Generate coded pieces and feed them to the decoder until decoding is complete
//! while !decoder.is_already_decoded() {
//!     let coded_piece = encoder.code(&mut rng);
//!
//!     match decoder.decode(&coded_piece) {
//!         Ok(_) => {},                                // Piece was useful
//!         Err(RLNCError::PieceNotUseful) => {},       // Piece was not useful (linearly dependent)
//!         Err(RLNCError::ReceivedAllPieces) => break, // Already decoded
//!         Err(e) => panic!("Unexpected error during decoding: {e:?}"),
//!     }
//! }
//!
//! // 5. Retrieve the decoded data
//! let decoded_data = decoder.get_decoded_data().expect("Failed to retrieve decoded data even after all pieces are received");
//!
//! // 6. Verify that the decoded data matches the original data
//! assert_eq!(original_data_copy, decoded_data);
//! println!("RLNC workflow completed successfully! Original data matches decoded data.");
//! ```
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rlnc = "=0.8.2"                                      # On x86 and aarch64 targets, it offers fast encoding, recoding and decoding, using SIMD intrinsics.
//! # or
//! rlnc = { version = "=0.8.2", features = "parallel" } # Uses `rayon`-based data-parallelism for fast encoding/ recoding. Decoding is not yet parallelized.
//!
//! rand = { version = "=0.9.1" } # Required for random number generation
//! ```
//!
//! For more see README in `rlnc` repository @ <https://github.com/itzmeanjan/rlnc>.

mod common;

pub mod full;
pub use crate::common::errors::RLNCError;
