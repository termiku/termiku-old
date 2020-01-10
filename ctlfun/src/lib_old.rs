pub mod recognizer;
pub mod v2;
mod utf8;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// A control sequence parameter.
pub enum Parameter {
    /// A for the control sequence appropriate default value should be used.
    Default,
    /// The parameter has a value.
    Value(u16)
}

impl Parameter {
    pub fn new(v: u16) -> Self {
        Self::Value(v)
    }
    
    /// Returns the value of the parameter, if present, otherwise returns the given default.
    pub fn value_or(&self, or: u16) -> u16 {
        match self {
            Self::Default  => or,
            Self::Value(v) => *v,
        }
    }
    
    /// Parsing parameters requires an [`atoi`]-like loop.
    ///
    /// Parameter value overflow causes the sequence to be rejected.
    ///
    /// [`atoi`]: https://en.cppreference.com/w/c/string/byte/atoi
    pub fn add(&mut self, x: u16) -> bool {
        match self {
            Self::Default => {
                *self = Self::Value(x);
                false
            },
            
            Self::Value(v) => {
                let (v2, oflw) = v.overflowing_add(x);
                *v = v2;
                oflw
            }
        }
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct ControlFunction {
    /// The start of the control function.
    ///
    /// For C0 and C1 controls, which are only 1 byte,
    /// this is the only necessary field.
    start: u8,
    /// Whether this control sequence has a private parameter string.
    private: bool,
    /// The parameters of the control sequence, if it is one.
    params: Vec<Parameter>,
    /// If this function is a control string,
    /// this is the string's content.
    ///
    /// Otherwise, it's the intermediate bytes of the function.
    /// For control sequences with private parameters, this contains the raw parameter string.
    bytes: Vec<u8>,
    /// The final byte of the control function.
    ///
    /// For C0 and C1 controls, as well as control strings,
    /// this field is left unset.
    end: u8,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TerminalInput<'a> {
    Continue,
    Char(char),
    // FIXME: Passing this by reference saves on allocations,
    // but currently requires that it is fully processed before parsing can continue.
    // For performance, it may be better to pass a clone by value, and use a queue to avoid
    // the input buffer getting clogged. Relevant for stuff that may take longer to evaluate,
    // like SIXEL strings.
    // Will require benchmarking though.
    Control(&'a ControlFunction),
}

#[derive(Clone, Debug, Default)]
pub struct TerminalInputParser {
    /// The current parsing state.
    state: State,
    /// Container for parsed control function data.
    ctl: ControlFunction,
    /// Accumulator for current control sequence parameter.
    pacc: Parameter,
    // /// UTF-8 character decoder.
    // utf8: UTF8Decoder
}

impl TerminalInputParser {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn parse_byte(&mut self, byte: u8) -> TerminalInput {
        
        unimplemented!()
    }
}



#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Class {
    /// C0 Control Functions
    ///
    /// 00..1F
    C0,
    
    /// C0 Control Functions permitted in Control Strings
    ///
    /// 08..0D
    C0S,
    
    /// ESCAPE
    ///
    /// 1B
    ESC,
    
    /// Control Function / Control Sequence Intermediate Bytes
    ///
    /// 20..2F
    INT,
    
    /// Control Sequence Parameter Bytes
    ///
    /// 30..39
    PAR,
    
    /// Control Sequence Parameter Separators
    ///
    /// 3A..3B
    SEP,
    
    /// Control Sequence Private Parameter String Indicator
    ///
    /// 3C..3F
    PRI,
    
    /// C1 Control Functions
    ///
    /// ESC 40..5F
    C1,
    
    /// Command String Opening Delimiter
    ///
    /// ESC 50, ESC 5D..5F
    CSO,
    
    /// Start Of String
    ///
    /// ESC 58
    SOS,
    
    /// Single Character Introducer
    ///
    /// ESC 5A
    SCI,
    
    /// Control Sequence Introducer
    ///
    /// ESC 5B
    CSI,
    
    /// String Terminator
    ///
    /// ESC 5C
    ST,
    
    /// Independent Control Function Final Bytes
    ///
    /// 60..7E
    ICF,
    
    /// DELETE
    ///
    /// 7F
    DEL,
}

use Class::*;

/// Byte to Class translation table
const CLASS_TABLE: [Class; 128] = [
    C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0S,C0S,C0S,C0S,C0S,C0S,C0 ,C0 ,
    C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,C0 ,ESC,C0 ,C0 ,C0 ,C0 ,
    INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,INT,
    PAR,PAR,PAR,PAR,PAR,PAR,PAR,PAR,PAR,PAR,SEP,SEP,PRI,PRI,PRI,PRI,
    C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,
    CSO,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,C1 ,SOS,C1 ,SCI,CSI,ST ,CSO,CSO,CSO,
    ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,
    ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,ICF,DEL,
];

/// State + Class to State transition table
const STATE_TABLE: [State; 185] = [State::OK;185];

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Action {
    Continue,
    ReturnChar,
    C01Control,
    StartSequence,
    FinishSequence,
    PushByte,
    SetPrivate,
    PushLastParam,
    PushParamAndByte,
    PushParam,
    AddParamValue,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
// NCB: 20..7E
// CFF: 30..7E
// CSC: 08..0D,20..7E

// 11 unique actions
// return char
// return continue
// set start, return control
// set start, return continue
// set end, return control
// push byte, return continue
// set private, return continue
// push param, set end, return control
// push param, push byte, return continue
// push param, return continue
// add value, return continue

// always return to some base state, which is a multiple of 15
// for example, we will never match against state 61
// use modulo 15 arithmetic to encode 61 as "action 1, state 60"
// modulo 15 not ideal
// pad rows to 16 so we can use modulo 16 (easy bitmask)



enum State {
    /// Base state
    ///
    /// ```text,ignore
    /// C0  -> OK_C01  (set start, return control)
    /// ESC -> ESC    (set start, return continue)
    /// NCB -> OK_NCB (           return char)
    /// DEL -> OK     (           return continue)
    /// ```
    OK = 0x00,
    
    /// Received ESCAPE
    ///
    /// ```text,ignore
    /// C0  -> OK_C01 (set start, return control)
    /// INT -> CF     (push byte, return continue)
    /// CFF -> OK_CF  (set end,   return control)
    /// C1  -> OK_C01 (set start, return control)
    /// CSO -> CMS    (set start, return continue)
    /// SOS -> SOS    (set start, return continue)
    /// SCI -> SCI    (set start, return continue)
    /// CSI -> CSI    (set start, return continue)
    /// _   -> OK     (           return continue)
    /// ```
    ESC = 0x10,
    
    /// Control Function
    ///
    /// ```text,ignore
    /// INT -> CF     (push byte, return continue)
    /// CFF -> OK_CF  (set end,   return control)
    /// _   -> ERR_CF (           return continue)
    /// ```
    CF = 0x20,
    
    /// Poisoned Control Function
    ///
    /// ```text,ignore
    /// CFF -> OK     (return continue)
    /// _   -> ERR_CF (return continue)
    /// ```
    ERR_CF = 0x30,
    
    /// Command String
    ///
    /// ```text,ignore
    /// ESC -> CMS_ESC (           return continue)
    /// CSC -> CMS_ACC (push byte, return continue)
    /// _   -> ERR_CMS (           return continue)
    /// ```
    CMS = 0x40,
    
    /// Command String, Received ESCAPE
    ///
    /// ```text,ignore
    /// ST -> OK_CF   (set end, return control)
    /// _  -> ERR_CMS (         return continue)
    /// ```
    CMS_ESC = 0x50,
    
    /// Poisoned Command String
    ///
    /// ```text,ignore
    /// ESC -> CMS_ESC (return continue)
    /// _   -> ERR_CMS (return continue)
    /// ```
    ERR_CMS = 0x60,
    
    /// Start Of String
    ///
    /// ```text,ignore
    /// ESC -> SOS_ESC (           return continue)
    /// _   -> SOS_ACC (push byte, return continue)
    /// ```
    SOS = 0x70,
    
    /// Start Of String, Received ESCAPE
    ///
    /// ```text,ignore
    /// ST  -> OK_CF   (set end,   return control)
    /// SOS -> ERR_CMS (           return continue)
    /// _   -> SOS_ACC (push byte, return continue)
    /// ```
    SOS_ESC = 0x80,
    
    /// Single Character Introducer
    ///
    /// ```text,ignore
    /// CSC -> OK_CF (set end, return control)
    /// _   -> OK    (         return continue)
    /// ```
    SCI = 0x90,
    
    /// Control Sequence Introducer
    ///
    /// ```text,ignore
    /// PRI -> CSI_PRI (set private,  return continue)
    /// PAR -> CSI_PAR (add value,    return continue)
    /// SEP -> CSI_SEP (push param,   return continue)
    /// INT -> CSI_INT (push byte,    return continue)
    /// CSF -> OK_CF   (set end,      return control)
    /// _   -> ERR_CSI (              return continue)
    /// ```
    CSI = 0xA0,
    
    /// Control Sequence Introducer, Received Parameter Byte
    ///
    /// ```text,ignore
    /// PAR -> CSI_PAR (add value,             return continue)
    /// SEP -> CSI_SEP (push param,            return continue)
    /// INT -> CSI_PIN (push param, push byte, return continue)
    /// CSF -> OK_CSI  (push param, set end,   return control)
    /// _   -> ERR_CSI (                       return continue)
    /// ```
    CSI_PAR = 0xB0,
    
    /// Control Sequence Introducer, Received Intermediate Byte
    ///
    /// ```text,ignore
    /// INT -> CSI_INT (push byte, return continue)
    /// CSF -> OK_CF   (set end,   return control)
    /// _   -> ERR_CSI (           return continue)
    /// ```
    CSI_INT = 0xC0,
    
    /// Poisoned Control Sequence Introducer
    ///
    /// ```text,ignore
    /// CSF -> OK      (return continue)
    /// _   -> ERR_CSI (return continue)
    /// ```
    ERR_CSI = 0xD0,
    
    // All states with an action.
    // Base states are implicitly Action::Continue.
    OK_C01  = State::OK  as u8 | Action::C01Control       as u8,
    OK_NCB  = State::OK  as u8 | Action::ReturnChar       as u8,
    TR_ESC  = State::ESC as u8 | Action::StartSequence    as u8,
    TR_CF   = State::CF  as u8 | Action::PushByte         as u8,
    OK_CF   = State::OK  as u8 | Action::FinishSequence   as u8,
    TR_CMS  = State::CMS as u8 | Action::StartSequence    as u8,
    TR_SOS  = State::SOS as u8 | Action::StartSequence    as u8,
    TR_SCI  = State::SCI as u8 | Action::StartSequence    as u8,
    TR_CSI  = State::CSI as u8 | Action::StartSequence    as u8,
    CMS_ACC = State::CMS as u8 | Action::PushByte         as u8,
    SOS_ACC = State::SOS as u8 | Action::PushByte         as u8,
    CSI_PRI = State::CSI as u8 | Action::SetPrivate       as u8,
    CSI_SEP = State::CSI as u8 | Action::PushParam        as u8,
    CSI_PAC = State::CSI as u8 | Action::AddParamValue    as u8,
    CSI_IAC = State::CSI as u8 | Action::PushByte         as u8,
    CSI_PIN = State::CSI as u8 | Action::PushParamAndByte as u8,
}

impl State {
    /// Decomposes a state into base state and parser action.
    fn decompose(self) -> (State, Action) {
        use std::mem::transmute as cast;
        
        unsafe {
            (cast(self as u8 & 0xF0), cast(self as u8 & 0x0F))
        }
    }
}

impl Default for State {
    fn default() -> State {
        State::OK
    }
}
