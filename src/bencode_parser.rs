use std::collections::HashMap;
use std::fs;
use std::iter::Peekable;
use std::str::Chars;
use std::vec::IntoIter;

/// This is a bencode-parser
pub struct BencodeParser {
    pub encoded_bc_source: Box<Peekable<IntoIter<char>>>,
    pub decoded_bc: BenStruct
}

/// Possible bencode phases
#[derive(Debug, Clone)]
pub enum BenStruct {
    Int { data: isize },
    Byte { length: u128, data: String },
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
            _ => todo!("Not implemented yet, use an iter")
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
        let bc = ben_source.chars();

        BencodeParser {
            encoded_bc_source: Box::new(
                fs::read_to_string(filepath)
                    .unwrap_or_else(
                        |_| panic!("Couldn't read bencode from {filepath}")
                    )
                    .chars()
                    .collect::<Vec<char>>()
                    .into_iter()
                    .peekable()
            ),
            decoded_bc: BenStruct::Null
        }
    }

    pub fn new_w_string(bc: String) -> BencodeParser {
        BencodeParser {
            encoded_bc_source: Box::new(
                bc
                    .chars()
                    .collect::<Vec<char>>()
                    .into_iter()
                    .peekable()
            ),
            decoded_bc: BenStruct::Null
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.encoded_bc_source.next()
    }

    /// Runner element
    pub fn decode_bencode(&mut self) -> BenStruct {
        self.decoded_bc = self.process_bencode();
        self.decoded_bc.clone()
    }

    fn process_bencode(&mut self) -> BenStruct {
        let mut delimiter_stack: Vec<char> = Vec::new();
        let tag = self.advance().unwrap();

        let ben_struct_coded = match tag {
            K_INT => {
                delimiter_stack.push('$');
                self.consume_int()
            },
            // For parsing bytes
            number_delimiter if number_delimiter.is_ascii_digit() => {
                let remaining_len_chars = self.consume_while(
                    &mut |char| char != ':'
                );
                let byte_len: u128 = format!("{number_delimiter}{remaining_len_chars}")
                    .parse()
                    .expect("Couldn't parse length of byte");
                self.consume_bytes(byte_len)
            },
            K_DICT => {
                delimiter_stack.push('{');
                BenStruct::Null
            },
            K_LIST => {
                let mut base_vec = Vec::new();
                let base_list = loop {
                    if delimiter_stack.is_empty() {
                        break BenStruct::List {
                            data: base_vec
                        }
                    }

                    base_vec.push(
                        self.process_bencode()
                    );
                };

                base_list
            },
            _ => self.consume_int()
        };

        if !delimiter_stack.is_empty() {
            panic!("Invalid bencode, delimiters unclosed!")
        };

        ben_struct_coded
    }

    fn consume_while<F>(&mut self, test: &mut F) -> String
        where F: FnMut(char) -> bool
    {
        let mut result = String::new();

        loop {
            let x = self.encoded_bc_source.peek().to_owned();
            if x.is_none() || !test(*x.unwrap()) {break}
            result.push(self.encoded_bc_source.next().unwrap());
        };

        result
    }

    fn consume_int(&mut self) -> BenStruct {
        let raw_int = self.consume_while(
            &mut |char| char != 'e'
        );
        let num: isize = raw_int.parse().expect("Couldn't parse integer");
        BenStruct::Int { data: num }
    }

    fn consume_bytes(&mut self, byte_len: u128) -> BenStruct {
        let mut counter: u128 = 0;
        // assert!(byte_len > counter);
        self.advance(); // to skip initial ':' char
        let raw_bytes = self.consume_while(&mut |_| {
            counter += 1;
            counter < byte_len + 1
        });
        BenStruct::Byte {
            length: byte_len,
            data: raw_bytes
        }
    }

    fn consume_lists(&mut self) -> BenStruct {

        BenStruct::List {
            data: vec![

            ]
        }
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
    #[should_panic(expected = "Invalid bencode, delimiters unclosed!")]
    fn invalid_bencode_w_stack_underflow_panics() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("dddee")
        );
        bc_parser.decode_bencode();
    }

    #[test]
    #[should_panic(expected = "Invalid bencode, excess closing delimiters!")]
    fn invalid_bencode_w_stack_overflow_panics() {
        let mut bc_parser = BencodeParser::new_w_string(
            String::from("ddeee")
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
        let raw_bytes = "31:debian-10.2.0-amd64-netinst.iso";
        let expected_bytes = raw_bytes.split_once(':').unwrap().1;
        let mut bc_parser = BencodeParser::new_w_string(
            String::from(raw_bytes)
        );
        let result = bc_parser.decode_bencode();
        if let BenStruct::Byte {length, data} = result {
            assert_eq!(length as usize, data.clone().len(), "Length of chars not same as passed len");
            assert_eq!(data.as_str(), expected_bytes, "Wrong chars decoded")
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
            BenStruct::Byte {length: 4, data: "spam".to_string()},
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

    // Dicts
    #[test]
    fn should_parse_dicts() {
        todo!()
    }

}