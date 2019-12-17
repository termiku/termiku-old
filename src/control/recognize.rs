use super::*;

// Interpret a control sequence, given all of its raw data.
// Also need a paramater buffer, which get reused to reduce dynamic allocations.
pub fn interpret_long_control(
    parameter_bytes: &[u8], intermediary_bytes: &[u8], final_byte: u8, parameters_buffer: &mut Vec<Option<u16>>
) -> ControlType {
    use ControlType::*;
    
    match final_byte {
        0x41 => {
            // CUU
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorUp(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x42 => {
            // CUD
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorDown(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x43 => {
            // CUF
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorRight(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x44 => {
            // CUB
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorLeft(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x45 => {
            // CNL
            if intermediary_bytes.is_empty(){
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorNextLine(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x46 => {
            // CPL
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorPrecedingLine(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x47 => {
            // CHA
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                CursorCharacterAbsolute(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x48 => {
            // CUP
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value_1 = get_parameter_default(parameters_buffer, 0, 1);
                let value_2 = get_parameter_default(parameters_buffer, 1, 1);
                
                CursorPosition(value_1, value_2)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x4A => {
            // ED
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 0);
                
                EraseInPage(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x4D => {
            // DL
            if intermediary_bytes.is_empty() {
                parse_parameters(parameter_bytes, parameters_buffer);
                
                let value = get_parameter_default(parameters_buffer, 0, 1);
                
                DeleteLine(value)
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x68 => {
            if intermediary_bytes.is_empty() {                
                SetMode(parse_unknown_length(parameter_bytes, parameters_buffer))
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x6C => {
            if intermediary_bytes.is_empty() {                
                ResetMode(parse_unknown_length(parameter_bytes, parameters_buffer))
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        }        
        0x6D => {
            // SGR
            if intermediary_bytes.is_empty() {
                SelectGraphicRendition(parse_unknown_length(parameter_bytes, parameters_buffer))
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x73 => {
            // SaveCursor
            if intermediary_bytes.is_empty() {
                SaveCursor
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        0x75 => {
            // RestoreCursor
            if intermediary_bytes.is_empty() {
                RestoreCursor
            } else {
                log_unknown(parameter_bytes, intermediary_bytes, final_byte)
            }
        },
        _ => {
            log_unknown(parameter_bytes, intermediary_bytes, final_byte)
        }
    }
}

fn parse_unknown_length(parameters_bytes: &[u8], parameters_buffer: &mut Vec<Option<u16>>) -> Vec<u16> {
    parse_parameters(parameters_bytes, parameters_buffer);
    
    let mut parameters = Vec::<u16>::with_capacity(8);

    for index in 0..parameters_buffer.len() {                    
        parameters.push(
            match parameters_buffer.get(index) {
                Some(maybe_data) => {
                    match maybe_data {
                        Some(data) => *data,
                        None => 0u16
                    }
                },
                None => break,
        });
    }
    
    parameters
}

fn log_unknown(parameter_bytes: &[u8], intermediary_bytes: &[u8], final_byte: u8) -> ControlType {
    println!("unknown sequence: params: {:x?}, inter: {:x?}, final: {:x?}", parameter_bytes, intermediary_bytes, final_byte);
    ControlType::Unknown
}

const NUMBER_RANGE: std::ops::RangeInclusive<u8> = 0x30..=0x39;

// Parse the parameters bytes.
// Not always called to save time for cases when they're not actually required.
// Somewhat follows ECMA-48 definition (Section 5.4.1 and 5.4.2), but doesn't handle sub-strings,
// since the majority of control functions doesn't use them.
// TODO: Another implementation which handle sub-strings.
// 
// Doesn't differentiate `:` and `;` for the delimiters.
// '?' and the likes are ignored. TODO: It shouldn't.
// 
// If a parameter is present, parse it to a Some(value), if not, parse it to a None.
// This way we can replace a None to its default value.
fn parse_parameters(parameter_bytes: &[u8], buffer: &mut Vec<Option<u16>>) {
    let mut current_value = None;
    
    if parameter_bytes.is_empty() {
        return;
    }
    
    for &byte in parameter_bytes {        
        // `0x3A` is ':', `0x3B` is ';'
        if byte == 0x3A || byte == 0x3B {
            buffer.push(current_value);
            current_value = None;
        } else if NUMBER_RANGE.contains(&byte) {
            // Get the last 4 bits, which will nicely translate to the actual number
            let byte_value = (byte & 0b0000_1111) as u16;
            
            match current_value {
                Some(value) => current_value = Some(value * 10 + byte_value),
                None => current_value = Some(byte_value)
            } 
        }
    }
    
    buffer.push(current_value);
}

fn get_parameter(buffer: &[Option<u16>], index: usize) -> Option<u16> {
    match buffer.get(index) {
        Some(value) => *value,
        None => None
    }
}

fn get_parameter_default(buffer: &[Option<u16>], index: usize, default: u16) -> u16 {
    match get_parameter(buffer, index) {
        Some(value) => value,
        None => default
    }
}


