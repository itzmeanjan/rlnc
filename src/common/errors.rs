/// Errors that can occur during RLNC (Random Linear Network Coding) encoding/ recoding/ decoding.
#[derive(Debug)]
pub enum RLNCError {
    // Encoder
    CodingVectorLengthMismatch,
    DataLengthMismatch,

    // Recoder
    NotEnoughPiecesToRecode,

    // Decoder
    PieceNotUseful,
    ReceivedAllPieces,
    NotAllPiecesReceivedYet,
    InvalidDecodedDataFormat,
}

impl std::fmt::Display for RLNCError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Encoder
            RLNCError::CodingVectorLengthMismatch => write!(f, "Coding vector length mismatch"),
            RLNCError::DataLengthMismatch => write!(f, "Data length mismatch"),

            // Recoder
            RLNCError::NotEnoughPiecesToRecode => write!(f, "Not enough pieces received to recode"),

            // Decoder
            RLNCError::PieceNotUseful => write!(f, "Received piece is not useful"),
            RLNCError::ReceivedAllPieces => write!(f, "Received all pieces"),
            RLNCError::NotAllPiecesReceivedYet => write!(f, "Not all pieces are received yet"),
            RLNCError::InvalidDecodedDataFormat => write!(f, "Invalid decoded data format"),
        }
    }
}
