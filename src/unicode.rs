const check_one:   u8 = 7;
const check_two:   u8 = 5;
const check_three: u8 = 4;
const check_four:  u8 = 3;

const one_byte:    u8 = 0b0;
const two_bytes:   u8 = 0b110;
const three_bytes: u8 = 0b1110;
const four_bytes:  u8 = 0b11110;

pub fn number_of_byte_needed(first_byte: u8) -> u8 {
    if (first_byte >> check_one) == one_byte {
        1
    } else if (first_byte >> check_two) == two_bytes {
        2
    } else if (first_byte >> check_three) == three_bytes {
        3
    } else if (first_byte >> check_four) == four_bytes {
        4
    } else {
        0
    }
}

const check_intermediary: u8 = 6;
const intermediary:       u8 = 0b10;

pub fn is_valid_intermediary(intermediary_byte: u8) -> bool {
    (intermediary_byte >> check_intermediary) == intermediary
}

pub fn convert_to_char(data: Vec<u8>) -> char {
    use utf8::Utf8Parser;

    let mut a = Utf8Parser::new();

    for byte in data {
        if let Some(c) = a.parse_byte(byte).unwrap() {
            return c
        }
    }

    panic!("Unable to parse UTF-8 character from `data`.")
}

////////////////////////////////////////////////////////////////////////////////
// UTF-8 PARSER
////////////////////////////////////////////////////////////////////////////////

pub mod utf8 {
    type Result = ::std::result::Result<Option<char>, Utf8ParserError>;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Utf8ParserError {
        InvalidByte,
        InvalidContinuationByte,
        InvalidCodePoint(u32),
        UnexpectedContinuationByte
    }

    #[derive(Clone, Default)]
    pub struct Utf8Parser {
        // The value of the current code point being parsed up to this point.
        value: u32,
        // The length of the current code point being parsed up to this point.
        length: u32,
    }

    // Bitmask used for leading bytes.
    // 110x xxxx for 2 byte,
    // 1110 xxxx for 3 byte, and
    // 1111 0xxx for 4 byte sequences.
    const LEAD_MASK: [u8; 3] = [0x1F, 0x0F, 0x07];

    // Bitmask used for continuation bytes.
    // Is always 11xx xxxx.
    const CONT_MASK: u8 = 0x3F;

    impl Utf8Parser {
        pub fn new() -> Utf8Parser {
            Utf8Parser::default()
        }

        // Tries to parse an UTF-8 byte.
        // Returns Ok(Some(char)) if a full character was parsed,
        // Ok(None) if the byte was parsed but no full character yet,
        // and Err(Utf8ParserError) otherwise.
        pub fn parse_byte(&mut self, byte: u8) -> Result {
            use std::convert::TryFrom;
            use Utf8ParserError::*;

            if !is_valid_utf8_byte(byte) {
                // If we get an invalid byte, reset parsing state.
                self.length = 0;
                return Err(InvalidByte)
            }

            // Start parsing a new sequence.
            if self.length == 0 {
                if byte < 0x80 {
                    return Ok(Some(char::from(byte)))
                } else if is_utf8_continuation_byte(byte) {
                    return Err(UnexpectedContinuationByte)
                }

                // We subtract 1 and treat it as the number of bytes following this one.
                self.length = utf8_length(byte) - 1;
                self.value  = (byte & LEAD_MASK[self.length as usize]) as u32;

                // Parsing is Ok, but we don't have a full char yet.
                return Ok(None)
            } else { // Continue parsing the current sequence
                if !is_utf8_continuation_byte(byte) {
                    // If we get an invalid continuation byte, reset parsing state.
                    self.length = 0;
                    return Err(InvalidContinuationByte)
                }

                self.value = (self.value << 6) | (byte & CONT_MASK) as u32;
                self.length -= 1;

                // We're done
                if self.length == 0 {
                    match char::try_from(self.value) {
                        Ok(c)  => return Ok(Some(c)),
                        Err(_) => return Err(InvalidCodePoint(self.value))
                    }
                } else {
                    // Parsing is Ok, but we don't have a full char yet.
                    return Ok(None)
                }
            }
        }

        pub fn parse_bytes(&mut self, bytes: &[u8]) {
            unimplemented!("Utf8Parser::parse_bytes is unimplemented due to uncertainties as to what its return type should be.");
        }
    }

    #[inline]
    fn is_valid_utf8_byte(byte: u8) -> bool {
        match byte {
            0xC0 |
            0xC1 |
            0xF5..=0xFF => false,
            _           => true
        }
    }

    #[inline]
    fn is_utf8_continuation_byte(byte: u8) -> bool {
        // Continuation bytes look like 10xx xxxx,
        // so we look at the top 2 bits and see if they match.
        (byte >> 6) == 0b10
    }

    #[inline]
    fn utf8_length(byte: u8) -> u32 {
        // This function assumes that `byte` is both a valid UTF-8 byte and *not* a continuation byte.
        debug_assert!(is_valid_utf8_byte(byte) && !is_utf8_continuation_byte(byte));

        if byte < 0x80 {
            1
        } else {
            (!byte).leading_zeros()
        }
    }
}
