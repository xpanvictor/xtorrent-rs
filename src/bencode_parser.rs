use std::collections::HashMap;
use std::fs;
use std::str::Chars;

/// This is a bencode-parser
pub struct BencodeParser {
    pub encoded_bc_source: String,
    pub decoded_bc: BenStruct
}

struct BenByte {
    length: u128, data: Chars<'static>
}

/// Possible bencode phases
#[derive(Debug, Clone)]
enum BenStruct {
    Int { data: isize },
    Byte { length: u128, data: Chars<'static> },
    List { data: Vec<BenStruct> },
    // Since the key will always be strings
    Dict { data: HashMap<String, Box<BenStruct>> },
    Null // type for null initiated data, always check
}

impl PartialEq for BenStruct {
    fn eq(&self, other: &Self) -> bool {
        match self {
            BenStruct::Byte {data, .. } => {
                if let BenStruct::Byte {data: word, ..} = other {
                    data.as_str() == word.as_str()
                } else {
                    panic!("Comparing invalid BenStructs")
                }
            }
            BenStruct::Int {data} => {
                if let BenStruct::Int {data: number} = other {
                    number == data
                } else { panic!("Comparing invalid BenStructs") }
             }
            _ => false
        }
    }
}

/// keywords
const K_DICT: char = 'd';
const K_LIST: char = 'l';
const K_INT: char = 'i';
const K_END: char = 'e';


impl BencodeParser {
    pub fn new_w_file(filepath: &str) -> BencodeParser {
        let ben_source = fs::read_to_string(filepath)
            .unwrap_or_else(|_| panic!("Couldn't read bencode from {filepath}"));

        BencodeParser {
            encoded_bc_source: ben_source.clone(),
            decoded_bc: BenStruct::Null
        }
    }

    pub fn new_w_string(bc: String) -> BencodeParser {
        BencodeParser {
            encoded_bc_source: bc,
            decoded_bc: BenStruct::Null
        }
    }

    /// Runner element
    fn decode_bencode(&mut self) -> BenStruct {
        let mut delimeter_stack: Vec<char> = Vec::new();
        for ch in self.encoded_bc_source.clone().chars() {
            println!("{ch}");
            match ch {
                K_DICT => {
                    delimeter_stack.push('{');
                    self.decoded_bc = BenStruct::Dict {
                        data: HashMap::new()
                    }

                },
                K_END => {
                    delimeter_stack.pop().expect("Invalid bencode!");
                },
                '\n' => continue,
                _ => panic!("Unknown char")
            }
        };
        self.decoded_bc.clone()
    }

    fn consume_dicts(&mut self) -> (String, BenStruct) {
        (String::new(), BenStruct::Null)
    }

}


#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use super::*;

    #[test]
    #[should_panic(expected = "Invalid bencode!")]
    fn invalid_bencode_w_stack_overflow_panics() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("dee")
        );
        bc_parser.decode_bencode();
    }

    #[test]
    fn should_parse_positive_int() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("i34e")
        );
        let result = bc_parser.decode_bencode();
        if let BenStruct::Int {data} = result {
            assert_eq!(data, 34);
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    #[test]
    fn should_parse_negative_int() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("i-34e")
        );
        let result = bc_parser.decode_bencode();
        if let BenStruct::Int {data} = result {
            assert_eq!(data, -34);
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    // Strings
    #[test]
    fn should_parse_byte() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("4:spam")
        );
        let result = bc_parser.decode_bencode();
        if let BenStruct::Byte {length, data} = result {
            assert_eq!(length as usize, data.clone().count(), "Length of chars not same as passed len");
            assert_eq!(data.as_str(), "spam", "Wrong chars decoded")
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    // Lists
    #[test]
    fn should_parse_lists() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("li42e4:spami-32e")
        );
        let result = bc_parser.decode_bencode();
        let expected_result = vec![
            BenStruct::Int {data: 42},
            BenStruct::Byte {length: 4, data: "spam".chars()},
            BenStruct::Int {data: -32}
        ];
        if let BenStruct::List {data} = result {
            let matchings = data.clone().iter().zip(expected_result.iter()).filter(
                |&(r, e)| r == e
            ).count();
            assert_eq!(matchings, data.len())
        } else {
            panic!("Invalid data type decoded!")
        }
    }

}