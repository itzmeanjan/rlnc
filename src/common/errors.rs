/// Errors that can occur during RLNC (Random Linear Network Coding) encoding/ recoding/ decoding.
#[derive(Debug, PartialEq)]
pub enum RLNCError {
    /// When the coding vector's length does not match the expected dimension during encoding.
    CodingVectorLengthMismatch,
    /// When the data length does not match the expected block size during encoding.
    DataLengthMismatch,
    /// When the piece count is zero.
    PieceCountZero,
    /// When the data length is zero.
    DataLengthZero,
    /// When the piece length is zero.
    PieceLengthZero,

    /// When there are not enough linearly independent pieces available to perform recoding.
    NotEnoughPiecesToRecode,
    /// When the full coded piece byte length is less than or equal to the number of pieces coded together.
    PieceLengthTooShort,

    /// When a received piece does not provide new linearly independent information.
    PieceNotUseful,
    /// When all necessary pieces have already been received, and no further pieces are needed to decode.
    ReceivedAllPieces,
    /// When an attempt is made to retrieve decoded data, but not all required pieces have arrived yet.
    NotAllPiecesReceivedYet,
    /// When the format or structure of the decoded data is not as expected.
    InvalidDecodedDataFormat,
    /// When the length of a received piece does not match the expected length.
    InvalidPieceLength,
    /// The user provided an invalid output buffer for the decoder.
    InvalidOutputBuffer,
}

impl std::fmt::Display for RLNCError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Encoder
            RLNCError::CodingVectorLengthMismatch => write!(f, "Coding vector length mismatch"),
            RLNCError::DataLengthMismatch => write!(f, "Data length mismatch"),
            RLNCError::PieceCountZero => write!(f, "Piece count is zero"),
            RLNCError::DataLengthZero => write!(f, "Data length is zero"),
            RLNCError::PieceLengthZero => write!(f, "Piece length is zero"),

            // Recoder
            RLNCError::NotEnoughPiecesToRecode => write!(f, "Not enough pieces received to recode"),
            RLNCError::PieceLengthTooShort => write!(f, "Piece length is too short"),

            // Decoder
            RLNCError::PieceNotUseful => write!(f, "Received piece is not useful"),
            RLNCError::ReceivedAllPieces => write!(f, "Received all pieces"),
            RLNCError::NotAllPiecesReceivedYet => write!(f, "Not all pieces are received yet"),
            RLNCError::InvalidDecodedDataFormat => write!(f, "Invalid decoded data format"),
            RLNCError::InvalidPieceLength => write!(f, "Invalid piece length"),

            RLNCError::InvalidOutputBuffer => write!(f, "Invalid output buffer provided for coding"),
        }
    }
}

impl std::error::Error for RLNCError {}
