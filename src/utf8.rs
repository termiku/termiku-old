/*
 * Copyright (c) 2008-2009 Bjoern Hoehrmann <bjoern@hoehrmann.de>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software
 * and associated documentation files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or
 * substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
 * BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

// See https://bjoern.hoehrmann.de/utf-8/decoder/dfa/ for more information on how this works.

const OK: u8 =   0;
const RJ: u8 =  96;
const RW: u8 = 108;

const UTF8_TABLE: [u8; 256+96] = [
    // Byte to character class translation
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x00
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x80
     9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, // 0x90
     7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, // 0xA0
     7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
     8, 8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xC0
     2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    10, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, // 0xE0
    11, 6, 6, 6, 5, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, // 0xF0

    // State + character class to state translation
    /*   0 - OK (Done)
     *  12 - 1 byte  needed
     *  24 - 2 bytes needed
     *  36 - 2 bytes needed (E0 lead)
     *  48 - 2 bytes needed (ED lead)
     *  60 - 3 bytes needed (F0 lead)
     *  72 - 3 bytes needed
     *  84 - 3 bytes needed (F4 lead)
     *  96 - RJ (Error)
     * 108 - RW (Rewind)
     *
     0   1   2   3   4   5   6   7   8   9  10  11  // Character class
     */
    OK, RJ, 12, 24, 48, 84, 72, RJ, RJ, RJ, 36, 60, // State   0
    RW, OK, RW, RW, RW, RW, RW, OK, RW, OK, RW, RW, // State  12
    RW, 12, RW, RW, RW, RW, RW, 12, RW, 12, RW, RW, // State  24
    RW, RW, RW, RW, RW, RW, RW, 12, RW, RW, RW, RW, // State  36
    RW, 12, RW, RW, RW, RW, RW, RW, RW, 12, RW, RW, // State  48
    RW, RW, RW, RW, RW, RW, RW, 24, RW, 24, RW, RW, // State  60
    RW, 24, RW, RW, RW, RW, RW, 24, RW, 24, RW, RW, // State  72
    RW, 24, RW, RW, RW, RW, RW, RW, RW, RW, RW, RW, // State  84
];

#[derive(Copy, Clone, Debug, Default)]
pub struct UTF8Decoder {
    code_point: u32,
    state: u8
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DecodeState {
    Done(char),
    Continue,
    Error,
    Rewind
}

impl UTF8Decoder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    
    #[inline]
    pub fn reset(&mut self) {
        self.state = 0;
    }
    
    pub fn decode_byte(&mut self, byte: u8) -> DecodeState {
        let class = UTF8_TABLE[byte as usize];
        
        self.code_point =
            if self.state == OK {
                (0xFF >> class) & byte as u32
            } else {
                (self.code_point << 6) | (byte as u32 & 0x3F)
            };
        
        unsafe {
            // The compiler can't verify this access is always in bounds, but it is, I promise.
            self.state = *UTF8_TABLE.get_unchecked(256 + self.state as usize + class as usize);
        
            match self.state {
                // Surrogate or out of bounds code points will be rejected, so this is safe.
                OK => DecodeState::Done(std::char::from_u32_unchecked(self.code_point)),
                RJ => { self.reset(); DecodeState::Error  },
                RW => { self.reset(); DecodeState::Rewind },
                _  => DecodeState::Continue
            }
        }
    }
}
