/// To ensure that any RLNC coded data gets decoded correctly, first we append a 1-byte boundary marker and then a N>=0 -many
/// zeros to make all data chunks equal sized. At decoding time, we can use this boundary marker to determine how far is the original data.
/// Once this boundary marker is encountered, there could be zero or more zero bytes following it. The number of zero bytes is determined by the
/// length of the original data and number of chunks.
pub const BOUNDARY_MARKER: u8 = 0x81;
