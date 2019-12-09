use std::ops::RangeInclusive;

pub const CSI_1: u8 = 0x1B;
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

#[derive(Debug)]
pub struct ControlSeqenceParser {
    state: ParserState,
    buffer: Vec<u8>,
    parameter_length: usize,
    intermediary_length: usize,
}

#[derive(Debug)]
pub enum ParserState {
    NotParsing,
    ParsingCsi,
    ParsingParameter,
    ParsingIntermediary,
    // No ParsingFinal, as the final byte is only of length 1
}

#[derive(Debug)]
pub enum ControlSequenceError {
    InvalidCsi1Byte,
    InvalidCsi2Byte,
    InvalidParameterByte,
    InvalidIntermediaryByte,
    InvalidFinalByte,
}

#[derive(Debug)]
pub enum ControlType {
    Unknown,
    Color
}

pub type ControlReturn =  Result<Option<ControlType>, ControlSequenceError>;

impl ControlSeqenceParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::NotParsing,
            buffer: Vec::with_capacity(64),
            parameter_length: 0,
            intermediary_length: 0,
        }
    }
    
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
                    Ok(Some(self.parse_control()))
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
                    Ok(Some(self.parse_control()))
                } else {
                    Err(ControlSequenceError::InvalidIntermediaryByte)
                }
            }
        }
    }
    
    fn parse_control(&mut self) -> ControlType {
        self.reset();
        ControlType::Unknown
    }
    
    /// Clear the buffer and reset parser, returning the parsed bytes
    pub fn reset(&mut self) -> Vec<u8> {
        self.state = ParserState::NotParsing;
        self.intermediary_length = 0;
        self.parameter_length = 0;
        self.buffer.drain(0..self.buffer.len()).collect()
    }
    
    pub fn is_parsing(&self) -> bool {
        if let ParserState::NotParsing = self.state {
            false
        } else {
            true
        }
    }
}

