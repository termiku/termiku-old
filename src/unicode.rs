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
    panic!()
}

