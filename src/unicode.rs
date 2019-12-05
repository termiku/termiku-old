////////////////////////////////////////////////////////////////////////////////
// UTF-8 PARSER
////////////////////////////////////////////////////////////////////////////////
type Result = ::std::result::Result<Option<char>, Utf8ParserError>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Utf8ParserError {
    InvalidByte,
    InvalidContinuationByte,
    InvalidCodePoint(u32),
    UnexpectedContinuationByte
}

#[derive(Copy, Clone, Debug, Default)]
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
fn utf8_length_unchecked(first_byte: u8) -> u32 {
    if first_byte < 0x80 {
        1
    } else {
        (!first_byte).leading_zeros()
    }
}

#[inline]
pub fn utf8_length(first_byte: u8) -> u32 {
    // Returns 0 as length if invalid, return the actual length if it is valid        
    if !is_valid_utf8_byte(first_byte) || is_utf8_continuation_byte(first_byte) {
        0
    } else {
        utf8_length_unchecked(first_byte)
    }
}
