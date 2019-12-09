mod recognize;
pub mod control_type;

use recognize::*;
use control_type::*;

use std::ops::RangeInclusive;

/// ESC
pub const CSI_1: u8 = 0x1B;

/// [
const CSI_2: u8 = 0x5B;

const PARAMETER_START: u8 = 0x30;
const PARAMETER_END: u8 = 0x3F;

const PARAMETER_RANGE: RangeInclusive<u8> = PARAMETER_START..=PARAMETER_END;

const INTERMEDIARY_START: u8 = 0x20;
const INTERMEDIARY_END: u8 = 0x2F;

const INTERMEDIARY_RANGE: RangeInclusive<u8> = INTERMEDIARY_START..=INTERMEDIARY_END;

const FINAL_START: u8 = 0x40;
const FINAL_END: u8 = 0x7E;

const FINAL_RANGE: RangeInclusive<u8> = FINAL_START..=FINAL_END;


/// A control sequence parser, according to ECMA-48 definition (Section 5.4)
/// 
/// Parse bytes one by one with `parse_byte`.
/// Should be externaly `reset`ed on error.
#[derive(Debug)]
pub struct ControlSeqenceParser {
    state: ParserState,
    buffer: Vec<u8>,
    parameter_length: usize,
    intermediary_length: usize,
    
    parameters_buffer: Vec<Option<u16>>,
}

#[derive(PartialEq, Eq, Debug)]
enum ParserState {
    NotParsing,
    ParsingCsi,
    ParsingParameter,
    ParsingIntermediary,
    // No ParsingFinal, as the final byte is only of length 1.
}

#[derive(Debug)]
pub enum ControlSequenceError {
    InvalidCsi1Byte,
    InvalidCsi2Byte,
    InvalidParameterByte,
    InvalidIntermediaryByte,
    InvalidFinalByte,
}

pub type ControlReturn =  Result<Option<ControlType>, ControlSequenceError>;

impl ControlSeqenceParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::NotParsing,
            buffer: Vec::with_capacity(64),
            parameter_length: 0,
            intermediary_length: 0,
            parameters_buffer: Vec::with_capacity(64)
        }
    }
    
    /// Parse one byte of a control sequence.
    /// 
    /// If no problem was detected, will return an `Ok(Option<ControlType>)`.
    /// `None` means it still needs more data, `Some` is the parsed `ControlType` and in this case
    /// the parser was automatically reseted.
    /// 
    /// On error, returns Err(ControlSequenceError), which indicate which byte was expected.
    /// Note that it doesn't mean it expected only this type of byte but also the ones going after:
    /// for example, an `InvalidIntermediaryByte` means that it was awaiting an intermediary byte
    /// or a final byte (except for `InvalidCsi1Byte`/`InvalidCsi2Byte`, which always means it
    /// wanted these bytes). Please see Section 5.4 of ECMA-48 for more information. 
    pub fn parse_byte(&mut self, byte: u8) -> ControlReturn {
        match self.state {
            ParserState::NotParsing => {
                if byte == CSI_1 {
                    self.buffer.push(byte);
                    self.state = ParserState::ParsingCsi;
                    Ok(None)
                } else {
                    Err(ControlSequenceError::InvalidCsi1Byte)
                }
            },
            ParserState::ParsingCsi => {
                if byte == CSI_2 {
                    self.buffer.push(byte);
                    self.state = ParserState::ParsingParameter;
                    Ok(None)
                } else {
                    Err(ControlSequenceError::InvalidCsi2Byte)
                }
            },
            ParserState::ParsingParameter => {
                if PARAMETER_RANGE.contains(&byte) {
                    self.buffer.push(byte);
                    self.parameter_length += 1;
                    Ok(None)
                } else if INTERMEDIARY_RANGE.contains(&byte) {
                    self.buffer.push(byte);
                    self.intermediary_length += 1;
                    self.state = ParserState::ParsingIntermediary;
                    Ok(None)
                } else if FINAL_RANGE.contains(&byte) {
                    self.buffer.push(byte);
                    self.state = ParserState::NotParsing;
                    Ok(Some(self.parse_buffer()))
                } else {
                    Err(ControlSequenceError::InvalidParameterByte)
                }
            },
            ParserState::ParsingIntermediary => {
                if INTERMEDIARY_RANGE.contains(&byte) {
                    self.buffer.push(byte);
                    self.intermediary_length += 1;
                    Ok(None)
                } else if FINAL_RANGE.contains(&byte) {
                    self.buffer.push(byte);
                    self.state = ParserState::NotParsing;
                    Ok(Some(self.parse_buffer()))
                } else {
                    Err(ControlSequenceError::InvalidIntermediaryByte)
                }
            }
        }
    }
    
    /// Clear the buffer and reset the parser state, returning the buffered bytes.
    pub fn reset(&mut self) -> Vec<u8> {
        self.state = ParserState::NotParsing;
        self.intermediary_length = 0;
        self.parameter_length = 0;
        self.buffer.drain(0..self.buffer.len()).collect()
    }
    
    pub fn is_parsing(&self) -> bool {
        if self.state == ParserState::NotParsing {
            false
        } else {
            true
        }
    }
    
    // Parse the buffer raw data, and deleguate its interpretation, returning the result.
    // Also reset the parser.
    fn parse_buffer(&mut self) -> ControlType {
        let parameter_bytes: &[u8] = &self.buffer[2..self.parameter_length + 2];
        let intermediary_bytes: &[u8] = &self.buffer[
            self.parameter_length + 2
            ..
            self.parameter_length + self.intermediary_length + 2
        ];
        let final_byte: &u8 = &self.buffer[self.buffer.len() - 1];
        
        self.parameters_buffer.clear();
        let control_type = interpret_control(parameter_bytes, intermediary_bytes, final_byte, &mut self.parameters_buffer);
        
    
        self.reset();    
        control_type        
    }
    
}
