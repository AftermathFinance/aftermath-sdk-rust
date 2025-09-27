/// 32-byte address from a hex byte vector, optionally `0x`-prefixed.
pub const fn hex_address_bytes(bytes: &[u8]) -> [u8; 32] {
    // Just for compatibility with `Address::from_str`.
    if bytes.is_empty() {
        panic!("input hex string must be non-empty");
    }
    let hex = remove_0x_prefix(bytes);
    if hex.len() > 64 {
        panic!("input hex string is too long for address");
    }
    let mut buffer = [0; 32];
    let mut i = hex.len();
    let mut j = buffer.len();
    while i >= 2 {
        let lo = HEX_DECODE_LUT[hex[i - 1] as usize];
        let hi = HEX_DECODE_LUT[hex[i - 2] as usize];
        if lo == NIL || hi == NIL {
            panic!("input hex string has wrong character");
        }
        buffer[j - 1] = (hi << 4) | lo;
        i -= 2;
        j -= 1;
    }
    if i == 1 {
        let lo = HEX_DECODE_LUT[hex[0] as usize];
        if lo == NIL {
            panic!("input hex string has wrong character");
        }
        buffer[j - 1] = lo;
    }
    buffer
}

/// Removes initial "0x" prefix if any.
const fn remove_0x_prefix(hex: &[u8]) -> &[u8] {
    if let Some((two, hex2)) = hex.split_first_chunk::<2>() {
        if two[0] == b'0' && two[1] == b'x' {
            return hex2;
        }
    }
    hex
}

/// The lookup table of hex byte to value, used for hex decoding.
///
/// [`NIL`] is used for invalid values.
const HEX_DECODE_LUT: &[u8; 256] = &make_decode_lut();

/// Represents an invalid value in the [`HEX_DECODE_LUT`] table.
const NIL: u8 = u8::MAX;

const fn make_decode_lut() -> [u8; 256] {
    let mut lut = [0; 256];
    let mut i = 0u8;
    loop {
        lut[i as usize] = match i {
            b'0'..=b'9' => i - b'0',
            b'A'..=b'F' => i - b'A' + 10,
            b'a'..=b'f' => i - b'a' + 10,
            // use max value for invalid characters
            _ => NIL,
        };
        if i == NIL {
            break;
        }
        i += 1;
    }
    lut
}
