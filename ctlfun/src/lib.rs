pub mod recognizer;

mod utf8;

const CLASS_TABLE: [Class; 0x80] = [
    Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,
    Class::C0S,Class::C0S,Class::C0S,Class::C0S,Class::C0S,Class::C0S,Class::C0 ,Class::C0 ,
    
    Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,
    Class::C0 ,Class::C0 ,Class::C0 ,Class::ESC,Class::C0 ,Class::C0 ,Class::C0 ,Class::C0 ,
    
    Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,
    Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,Class::INT,
    
    Class::PAR,Class::PAR,Class::PAR,Class::PAR,Class::PAR,Class::PAR,Class::PAR,Class::PAR,
    Class::PAR,Class::PAR,Class::SEP,Class::SEP,Class::PRI,Class::PRI,Class::PRI,Class::PRI,
    
    Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,
    Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,
    
    Class::CSO,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,Class::C1 ,
    Class::SOS,Class::C1 ,Class::SCI,Class::CSI,Class::ST ,Class::CSO,Class::CSO,Class::CSO,
    
    Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,
    Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,
    
    Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,
    Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::ICF,Class::DEL,
];

const STATE_TABLE: [State; 0xE0] = [
    State::C0Control,   // Ground + C0
    State::C0Control,   // Ground + C0S
    State::StartEscape, // Ground + ESC
    State::Char,        // Ground + INT
    State::Char,        // Ground + PAR
    State::Char,        // Ground + SEP
    State::Char,        // Ground + PRI
    State::Char,        // Ground + C1
    State::Char,        // Ground + CSO
    State::Char,        // Ground + SOS
    State::Char,        // Ground + SCI
    State::Char,        // Ground + CSI
    State::Char,        // Ground + ST
    State::Char,        // Ground + ICF
    State::Ground,      // Ground + DEL
    State::Ground,      // Ground + PAD
    
    State::C0Control,            // Escape + C0
    State::C0Control,            // Escape + C0S
    State::StartEscape,          // Escape + ESC
    State::PushIntermediateByte, // Escape + INT
    State::FinishSequence,       // Escape + PAR
    State::FinishSequence,       // Escape + SEP
    State::FinishSequence,       // Escape + PRI
    State::C1Control,            // Escape + C1
    State::StartCommandString,   // Escape + CSO
    State::StartCharacterString, // Escape + SOS
    State::StartSingleCharacter, // Escape + SCI
    State::StartControlSequence, // Escape + CSI
    State::C1Control,            // Escape + ST
    State::FinishSequence,       // Escape + ICF
    State::Ground,               // Escape + DEL
    State::Ground,               // Escape + PAD
    
    State::ControlFunctionError,  // ControlFunction + C0
    State::ControlFunctionError,  // ControlFunction + C0S
    State::ControlFunctionError,  // ControlFunction + ESC
    State::PushIntermediateByte,  // ControlFunction + INT
    State::FinishSequence,        // ControlFunction + PAR
    State::FinishSequence,        // ControlFunction + SEP
    State::FinishSequence,        // ControlFunction + PRI
    State::FinishSequence,        // ControlFunction + C1
    State::FinishSequence,        // ControlFunction + CSO
    State::FinishSequence,        // ControlFunction + SOS
    State::FinishSequence,        // ControlFunction + SCI
    State::FinishSequence,        // ControlFunction + CSI
    State::FinishSequence,        // ControlFunction + ST
    State::FinishSequence,        // ControlFunction + ICF
    State::Ground,                // ControlFunction + DEL
    State::ControlFunctionError,  // ControlFunction + PAD
    
    State::ControlFunctionError,  // ControlFunctionError + C0
    State::ControlFunctionError,  // ControlFunctionError + C0S
    State::ControlFunctionError,  // ControlFunctionError + ESC
    State::ControlFunctionError,  // ControlFunctionError + INT
    State::Ground,                // ControlFunctionError + PAR
    State::Ground,                // ControlFunctionError + SEP
    State::Ground,                // ControlFunctionError + PRI
    State::Ground,                // ControlFunctionError + C1
    State::Ground,                // ControlFunctionError + CSO
    State::Ground,                // ControlFunctionError + SOS
    State::Ground,                // ControlFunctionError + SCI
    State::Ground,                // ControlFunctionError + CSI
    State::Ground,                // ControlFunctionError + ST
    State::Ground,                // ControlFunctionError + ICF
    State::ControlFunctionError,  // ControlFunctionError + DEL
    State::ControlFunctionError,  // ControlFunctionError + PAD
    
    State::ControlStringError,  // CommandString + C0
    State::PushCommandString,   // CommandString + C0S
    State::CommandStringEscape, // CommandString + ESC
    State::PushCommandString,   // CommandString + INT
    State::PushCommandString,   // CommandString + PAR
    State::PushCommandString,   // CommandString + SEP
    State::PushCommandString,   // CommandString + PRI
    State::PushCommandString,   // CommandString + C1
    State::PushCommandString,   // CommandString + CSO
    State::PushCommandString,   // CommandString + SOS
    State::PushCommandString,   // CommandString + SCI
    State::PushCommandString,   // CommandString + CSI
    State::PushCommandString,   // CommandString + ST
    State::PushCommandString,   // CommandString + ICF
    State::ControlStringError,  // CommandString + DEL
    State::ControlStringError,  // CommandString + PAD
    
    State::ControlStringError, // CommandStringEscape + C0
    State::ControlStringError, // CommandStringEscape + C0S
    State::ControlStringError, // CommandStringEscape + ESC
    State::ControlStringError, // CommandStringEscape + INT
    State::ControlStringError, // CommandStringEscape + PAR
    State::ControlStringError, // CommandStringEscape + SEP
    State::ControlStringError, // CommandStringEscape + PRI
    State::ControlStringError, // CommandStringEscape + C1
    State::ControlStringError, // CommandStringEscape + CSO
    State::ControlStringError, // CommandStringEscape + SOS
    State::ControlStringError, // CommandStringEscape + SCI
    State::ControlStringError, // CommandStringEscape + CSI
    State::FinishSequence,     // CommandStringEscape + ST
    State::ControlStringError, // CommandStringEscape + ICF
    State::ControlStringError, // CommandStringEscape + DEL
    State::ControlStringError, // CommandStringEscape + PAD
    
    State::PushCharacterString,   // CharacterString + C0
    State::PushCharacterString,   // CharacterString + C0S
    State::CharacterStringEscape, // CharacterString + ESC
    State::PushCharacterString,   // CharacterString + INT
    State::PushCharacterString,   // CharacterString + PAR
    State::PushCharacterString,   // CharacterString + SEP
    State::PushCharacterString,   // CharacterString + PRI
    State::PushCharacterString,   // CharacterString + C1
    State::PushCharacterString,   // CharacterString + CSO
    State::PushCharacterString,   // CharacterString + SOS
    State::PushCharacterString,   // CharacterString + SCI
    State::PushCharacterString,   // CharacterString + CSI
    State::PushCharacterString,   // CharacterString + ST
    State::PushCharacterString,   // CharacterString + ICF
    State::PushCharacterString,   // CharacterString + DEL
    State::ControlStringError,    // CharacterString + PAD
    
    State::PushCharacterStringEscape, // CharacterStringEscape + C0
    State::PushCharacterStringEscape, // CharacterStringEscape + C0S
    State::PushCharacterStringEscape, // CharacterStringEscape + ESC
    State::PushCharacterStringEscape, // CharacterStringEscape + INT
    State::PushCharacterStringEscape, // CharacterStringEscape + PAR
    State::PushCharacterStringEscape, // CharacterStringEscape + SEP
    State::PushCharacterStringEscape, // CharacterStringEscape + PRI
    State::PushCharacterStringEscape, // CharacterStringEscape + C1
    State::PushCharacterStringEscape, // CharacterStringEscape + CSO
    State::ControlStringError,        // CharacterStringEscape + SOS
    State::PushCharacterStringEscape, // CharacterStringEscape + SCI
    State::PushCharacterStringEscape, // CharacterStringEscape + CSI
    State::FinishSequence,            // CharacterStringEscape + ST
    State::PushCharacterStringEscape, // CharacterStringEscape + ICF
    State::PushCharacterStringEscape, // CharacterStringEscape + DEL
    State::ControlStringError,        // CharacterStringEscape + PAD
    
    State::ControlStringError, // ControlStringError + C0
    State::ControlStringError, // ControlStringError + C0S
    State::ControlStringError, // ControlStringError + ESC
    State::ControlStringError, // ControlStringError + INT
    State::ControlStringError, // ControlStringError + PAR
    State::ControlStringError, // ControlStringError + SEP
    State::ControlStringError, // ControlStringError + PRI
    State::ControlStringError, // ControlStringError + C1
    State::ControlStringError, // ControlStringError + CSO
    State::ControlStringError, // ControlStringError + SOS
    State::ControlStringError, // ControlStringError + SCI
    State::ControlStringError, // ControlStringError + CSI
    State::Ground,             // ControlStringError + ST
    State::ControlStringError, // ControlStringError + ICF
    State::ControlStringError, // ControlStringError + DEL
    State::ControlStringError, // ControlStringError + PAD
    
    State::Ground,         // SingleCharacter + C0
    State::FinishSequence, // SingleCharacter + C0S
    State::Ground,         // SingleCharacter + ESC
    State::FinishSequence, // SingleCharacter + INT
    State::FinishSequence, // SingleCharacter + PAR
    State::FinishSequence, // SingleCharacter + SEP
    State::FinishSequence, // SingleCharacter + PRI
    State::FinishSequence, // SingleCharacter + C1
    State::FinishSequence, // SingleCharacter + CSO
    State::FinishSequence, // SingleCharacter + SOS
    State::FinishSequence, // SingleCharacter + SCI
    State::FinishSequence, // SingleCharacter + CSI
    State::FinishSequence, // SingleCharacter + ST
    State::FinishSequence, // SingleCharacter + ICF
    State::Ground,         // SingleCharacter + DEL
    State::Ground,         // SingleCharacter + PAD
    
    State::ControlSequenceError,            // ControlSequence + C0
    State::ControlSequenceError,            // ControlSequence + C0S
    State::ControlSequenceError,            // ControlSequence + ESC
    State::ControlSequencePushIntermediate, // ControlSequence + INT
    State::ControlSequenceAddParameter,     // ControlSequence + PAR
    State::ControlSequencePushParameter,    // ControlSequence + SEP
    State::PrivateControlSequence,          // ControlSequence + PRI
    State::FinishControlSequence,           // ControlSequence + C1
    State::FinishControlSequence,           // ControlSequence + CSO
    State::FinishControlSequence,           // ControlSequence + SOS
    State::FinishControlSequence,           // ControlSequence + SCI
    State::FinishControlSequence,           // ControlSequence + CSI
    State::FinishControlSequence,           // ControlSequence + ST
    State::FinishControlSequence,           // ControlSequence + ICF
    State::ControlSequenceError,            // ControlSequence + DEL
    State::ControlSequenceError,            // ControlSequence + PAD
    
    State::ControlSequenceError,                 // ControlSequenceParameter + C0
    State::ControlSequenceError,                 // ControlSequenceParameter + C0S
    State::ControlSequenceError,                 // ControlSequenceParameter + ESC
    State::ControlSequenceParameterIntermediate, // ControlSequenceParameter + INT
    State::ControlSequenceAddParameter,          // ControlSequenceParameter + PAR
    State::ControlSequencePushParameter,         // ControlSequenceParameter + SEP
    State::ControlSequenceError,                 // ControlSequenceParameter + PRI
    State::FinishControlSequence,                // ControlSequenceParameter + C1
    State::FinishControlSequence,                // ControlSequenceParameter + CSO
    State::FinishControlSequence,                // ControlSequenceParameter + SOS
    State::FinishControlSequence,                // ControlSequenceParameter + SCI
    State::FinishControlSequence,                // ControlSequenceParameter + CSI
    State::FinishControlSequence,                // ControlSequenceParameter + ST
    State::FinishControlSequence,                // ControlSequenceParameter + ICF
    State::ControlSequenceError,                 // ControlSequenceParameter + DEL
    State::ControlSequenceError,                 // ControlSequenceParameter + PAD
    
    State::ControlSequenceError,            // ControlSequenceIntermediate + C0
    State::ControlSequenceError,            // ControlSequenceIntermediate + C0S
    State::ControlSequenceError,            // ControlSequenceIntermediate + ESC
    State::ControlSequencePushIntermediate, // ControlSequenceIntermediate + INT
    State::ControlSequenceError,            // ControlSequenceIntermediate + PAR
    State::ControlSequenceError,            // ControlSequenceIntermediate + SEP
    State::ControlSequenceError,            // ControlSequenceIntermediate + PRI
    State::FinishControlSequence,           // ControlSequenceIntermediate + C1
    State::FinishControlSequence,           // ControlSequenceIntermediate + CSO
    State::FinishControlSequence,           // ControlSequenceIntermediate + SOS
    State::FinishControlSequence,           // ControlSequenceIntermediate + SCI
    State::FinishControlSequence,           // ControlSequenceIntermediate + CSI
    State::FinishControlSequence,           // ControlSequenceIntermediate + ST
    State::FinishControlSequence,           // ControlSequenceIntermediate + ICF
    State::ControlSequenceError,            // ControlSequenceIntermediate + DEL
    State::ControlSequenceError,            // ControlSequenceIntermediate + PAD
    
    State::ControlSequenceError, // ControlSequenceError + C0
    State::ControlSequenceError, // ControlSequenceError + C0S
    State::ControlSequenceError, // ControlSequenceError + ESC
    State::ControlSequenceError, // ControlSequenceError + INT
    State::ControlSequenceError, // ControlSequenceError + PAR
    State::ControlSequenceError, // ControlSequenceError + SEP
    State::ControlSequenceError, // ControlSequenceError + PRI
    State::Ground,               // ControlSequenceError + C1
    State::Ground,               // ControlSequenceError + CSO
    State::Ground,               // ControlSequenceError + SOS
    State::Ground,               // ControlSequenceError + SCI
    State::Ground,               // ControlSequenceError + CSI
    State::Ground,               // ControlSequenceError + ST
    State::Ground,               // ControlSequenceError + ICF
    State::ControlSequenceError, // ControlSequenceError + DEL
    State::ControlSequenceError, // ControlSequenceError + PAD
]; 

#[repr(u8)]
#[derive(Copy, Clone)]
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

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    // All base states
    
    Ground                      = 0x00,
    Escape                      = 0x10,
    ControlFunction             = 0x20,
    ControlFunctionError        = 0x30,
    CommandString               = 0x40,
    CommandStringEscape         = 0x50,
    CharacterString             = 0x60,
    CharacterStringEscape       = 0x70,
    ControlStringError          = 0x80,
    SingleCharacter             = 0x90,
    ControlSequence             = 0xA0,
    ControlSequenceParameter    = 0xB0,
    ControlSequenceIntermediate = 0xC0,
    ControlSequenceError        = 0xD0,
    
    // All action states
    // The upper 4 bits set a base state to return to (see above),
    // the lower 4 bits set an action to perform (see below)
    // Base states impicitly have Action::Continue.
    
    C0Control =
        State::Ground as u8 | Action::C01Control as u8,
        
    Char =
        State::Ground as u8 | Action::Char as u8,
        
    StartEscape =
        State::Escape as u8 | Action::StartSequence as u8,
        
    PushIntermediateByte =
        State::ControlFunction as u8 | Action::PushByte as u8,
        
    C1Control =
        State::Escape as u8 | Action::C01Control as u8,
        
    FinishSequence =
        State::Ground as u8 | Action::FinishSequence as u8,
        
    StartCommandString =
        State::CommandString as u8 | Action::StartSequence as u8,
        
    StartCharacterString =
        State::CharacterString as u8 | Action::StartSequence as u8,
        
    StartSingleCharacter =
        State::SingleCharacter as u8 | Action::StartSequence as u8,
        
    StartControlSequence =
        State::ControlSequence as u8 | Action::StartSequence as u8,
        
    PushCommandString =
        State::CommandString as u8 | Action::PushByte as u8,
        
    PushCharacterString =
        State::CharacterString as u8 | Action::PushByte as u8,
        
    PushCharacterStringEscape =
        State::CharacterString as u8 | Action::PushByteWithEscape as u8,
        
    PrivateControlSequence =
        State::ControlSequence as u8 | Action::SetPrivate as u8,
        
    ControlSequencePushParameter =
        State::ControlSequence as u8 | Action::PushParam as u8,
    
    ControlSequenceAddParameter =
        State::ControlSequenceParameter as u8 | Action::AddParamValue as u8,
    
    ControlSequenceParameterIntermediate =
        State::ControlSequenceIntermediate as u8 | Action::PushParamAndByte as u8,
    
    ControlSequencePushIntermediate =
        State::ControlSequenceIntermediate as u8 | Action::PushByte as u8,
    
    FinishControlSequence =
        State::Ground as u8 | Action::PushParamAndEndSequence as u8,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum Action {
    // Variant is never constructed, but is matched on
    #[allow(dead_code)]
    /// Return Continue
    Continue,
    
    /// Return Char
    Char,
    
    /// Set `start`, return Control
    C01Control,
    
    /// Set `start`, return Continue
    StartSequence,
    
    /// Set `end`, return Control
    FinishSequence,
    
    /// Push `byte`, return Continue
    PushByte,
    
    /// Push `Escape`, push `byte`, return Continue
    PushByteWithEscape,
    
    /// Set `private`, return Continue
    SetPrivate,
    
    /// Add to parameter value, return Continue
    AddParamValue,
    
    /// Push `param`, return Continue
    PushParam,
    
    /// Push `param`, push `byte`, return Continue
    PushParamAndByte,
    
    /// Push `param`, set `end`, return Control
    PushParamAndEndSequence,
}

impl State {
    /// Decomposes a state into base state and parser action.
    fn decompose(self) -> (State, Action) {
        use std::mem::transmute as cast;
        
        unsafe {
            (cast(self as u8 & 0xF0), cast(self as u8 & 0x0F))
        }
    }
    
    /// Poisons the state
    fn poison(&mut self) {
        *self = STATE_TABLE[*self as usize + 0xF];
    }
}

impl Default for State {
    fn default() -> State {
        State::Ground
    }
}


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
    // If the UTF-8 decoder had to synchronize, two characters have to be inserted
    SyncChar(char, char),
    // FIXME: Passing this by reference saves on allocations,
    // but currently requires that it is fully processed before parsing can continue.
    // For performance, it may be better to pass a clone by value, and use a queue to avoid
    // the input buffer getting clogged. Relevant for stuff that may take longer to evaluate,
    // like SIXEL strings.
    // Will require benchmarking though.
    Control(&'a ControlFunction),
    
    SyncControl(char, &'a ControlFunction)
}

#[derive(Clone, Debug, Default)]
pub struct TerminalInputParser {
    /// The current parsing state.
    state: State,
    /// Container for parsed control function data.
    ctl: ControlFunction,
    /// Accumulator for current control sequence parameter.
    pacc: Parameter,
    /// UTF-8 character decoder.
    utf8: utf8::UTF8Decoder
}

impl TerminalInputParser {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn parse_byte(&mut self, byte: u8) -> TerminalInput {
        if byte >= 0x80 {
            if self.state != State::Ground {
                self.state.poison();
                return TerminalInput::Continue;
            }
            
            // UTF-8 here
            match self.utf8.decode_byte(byte) {
                utf8::DecodeState::Continue => TerminalInput::Continue,
                utf8::DecodeState::Done(c)  => TerminalInput::Char(c),
                utf8::DecodeState::Error    => TerminalInput::Char('\u{FFFD}'),
                utf8::DecodeState::Rewind   => {
                    // Recurse, but only once
                    let again = self.parse_byte(byte);
                    
                    match again {
                        TerminalInput::Continue => TerminalInput::Char('\u{FFFD}'),
                        TerminalInput::Char(c)  => TerminalInput::SyncChar('\u{FFFD}', c),
                        TerminalInput::Control(ctl) => TerminalInput::SyncControl('\u{FFFD}', ctl),
                        // We can't hit UTF-8 Rewind from the base state,
                        // so we can never produce SyncChar or SyncControl here
                        _ => unsafe { std::hint::unreachable_unchecked() }
                    }
                }
            }
        } else {
            let class = CLASS_TABLE[byte as usize] as usize;

            let state = unsafe {
                *STATE_TABLE.get_unchecked(self.state as usize + class)
            };
            
            let (base, action) = state.decompose();
            
            self.state = base;
            
            match action {
                Action::Continue => TerminalInput::Continue,
                Action::Char     => TerminalInput::Char(byte as char),
                Action::C01Control => {
                    self.ctl.start = byte;
                    TerminalInput::Control(&self.ctl)
                },
                
                Action::StartSequence => {
                    self.ctl.start = byte;
                    self.ctl.params.clear();
                    self.ctl.bytes.clear();
                    TerminalInput::Continue
                },
                
                Action::FinishSequence => {
                    self.ctl.end = byte;
                    TerminalInput::Control(&self.ctl)
                },
                
                Action::PushByte => {
                    self.ctl.bytes.push(byte);
                    TerminalInput::Continue
                },
                
                Action::PushByteWithEscape => {
                    self.ctl.bytes.push(0x1B);
                    self.ctl.bytes.push(byte);
                    TerminalInput::Continue
                },
                
                Action::SetPrivate => {
                    self.ctl.private = true;
                    TerminalInput::Continue
                },
                
                Action::AddParamValue => {
                    let oflw = self.pacc.add(byte as u16 - 0x30);
                    
                    // You can theoretically do this if-less
                    // using something like state ^= (0x70 * oflw)
                    // It just turns the branch into a conditional move and a xor.
                    if oflw {
                        self.state = State::ControlSequenceError;
                    }
                    
                    TerminalInput::Continue
                },
                
                Action::PushParam => {
                    self.ctl.params.push(self.pacc);
                    self.pacc = Parameter::Default;
                    
                    TerminalInput::Continue
                },
                
                Action::PushParamAndByte => {
                    self.ctl.bytes.push(byte);
                    self.ctl.params.push(self.pacc);
                    self.pacc = Parameter::Default;
                    
                    TerminalInput::Continue
                },
                
                Action::PushParamAndEndSequence => {                    
                    self.ctl.params.push(self.pacc);
                    self.pacc = Parameter::Default;
                    self.ctl.end = byte;
                    TerminalInput::Control(&self.ctl)
                }
            }
        }
    }
}
