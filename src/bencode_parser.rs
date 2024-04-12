use std::collections::HashMap;
use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::vec::IntoIter;

/// This is a bencode-parser
pub struct BencodeParser {
    pub encoded_bc_source: Box<Peekable<IntoIter<char>>>,
    pub decoded_bc: BenStruct,
}

// logic
// recursive approach baby

/// Possible bencode phases
#[derive(Debug, Clone, Eq)]
pub enum BenStruct {
    Int { data: isize },
    Byte { length: u128, data: String },
    List { data: Vec<BenStruct> },
    // Since the key will always be strings
    Dict { data: HashMap<String, BenStruct> },
    Null, // type for null initiated data, always check
}

impl PartialEq for BenStruct {
    fn eq(&self, other: &Self) -> bool {
        match self {
            BenStruct::Byte { data, .. } => {
                if let BenStruct::Byte { data: word, .. } = other {
                    data.as_str() == word.as_str()
                } else {
                    false
                }
            }
            BenStruct::Int { data } => {
                if let BenStruct::Int { data: number } = other {
                    number == data
                } else {
                    false
                }
            }
            BenStruct::List { data } => {
                if let BenStruct::List {
                    data: expected_list,
                } = other
                {
                    let match_count = data
                        .clone()
                        .iter()
                        .zip(expected_list.iter())
                        .filter(|&(r, e)| r == e)
                        .count();
                    match_count == data.len()
                } else {
                    false
                }
            }
            BenStruct::Dict { data } => {
                if let BenStruct::Dict {
                    data: expected_dict,
                } = other
                {
                    expected_dict == data
                } else {
                    false
                }
            }
            _ => panic!("This data type doesn't exist on the benstruct"),
        }
    }
}

impl BenStruct {
    fn get_data(&self) -> Box<dyn std::any::Any> {
        match self {
            BenStruct::Int { data } => Box::new(data),
            BenStruct::Byte { length, data } => Box::new(data),
            BenStruct::Dict { data } => Box::new(data),
            BenStruct::List { data } => Box::new(data),
            BenStruct::Null => Box::new(0),
        }
    }
}

/// keywords
const K_DICT: char = 'd';
const K_LIST: char = 'l';
const K_INT: char = 'i';
const K_END: char = 'e';

impl BencodeParser {
    pub fn parse_input(content: String) -> Peekable<IntoIter<char>> {
        content
            .chars()
            .filter(|ch| !matches!(ch, '\n' | '\t'))
            .collect::<Vec<char>>()
            .into_iter()
            .peekable()
    }

    // # Panics
    pub fn new_w_file(filepath: &Path) -> BencodeParser {
        // let ben_source = fs::read_to_string(filepath)
        //     .unwrap_or_else(|_| panic!("Couldn't read bencode from {filepath}"));

        BencodeParser {
            encoded_bc_source: Box::new(Self::parse_input(
                fs::read_to_string(filepath)
                    .unwrap_or_else(|_| panic!("Couldn't read bencode from {:?}", filepath)),
            )),
            decoded_bc: BenStruct::Null,
        }
    }

    pub fn new_w_string(bc: String) -> BencodeParser {
        BencodeParser {
            encoded_bc_source: Box::new(Self::parse_input(bc)),
            decoded_bc: BenStruct::Null,
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

    /// recursive approach
    fn process_bencode(&mut self) -> BenStruct {
        let tag = self.advance().expect("Couldn't extract tag");
        match tag {
            K_INT => self.consume_int(),
            K_LIST => {
                let mut base_vec = Vec::new();

                // while we have elements in the list
                // extract next element, append into base_vec
                loop {
                    let elem = self.process_bencode();
                    match elem {
                        BenStruct::Null => break,
                        _ => base_vec.push(elem),
                    }
                }

                BenStruct::List { data: base_vec }
            }
            // Dict
            K_DICT => {
                let mut base_hash_map: HashMap<String, BenStruct> = HashMap::new();

                // while k-v pairs in dict, extract and append
                loop {
                    let key = if let BenStruct::Byte { length: _, data } = self.process_bencode() {
                        data
                    } else {
                        break;
                    };

                    let value = self.process_bencode();

                    // insert key-value to HashMap
                    base_hash_map.insert(key, value);
                }

                BenStruct::Dict {
                    data: base_hash_map,
                }
            }
            // Bytes - For parsing bytes
            number_delimiter if number_delimiter.is_ascii_digit() => {
                let remaining_len_chars = self.consume_while(&mut |char| char != ':');
                let byte_len: u128 = format!("{number_delimiter}{remaining_len_chars}")
                    .parse()
                    .expect("Couldn't parse length of byte");
                self.consume_bytes(byte_len)
            }

            _ => BenStruct::Null,
        }
    }

    fn consume_while<F>(&mut self, test: &mut F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::new();

        loop {
            let x = self.encoded_bc_source.peek().to_owned();
            if x.is_none() || !test(*x.unwrap()) {
                break;
            }
            result.push(self.encoded_bc_source.next().unwrap());
        }

        result
    }

    fn check_end(&mut self) {
        assert_eq!(self.advance().unwrap(), K_END)
    }

    fn consume_int(&mut self) -> BenStruct {
        let raw_int = self.consume_while(&mut |char| char != 'e');
        let num: isize = raw_int.parse().expect("Couldn't parse integer");
        self.check_end();
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
            data: raw_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    #[should_panic(expected = "Invalid bencode, delimiters unclosed!")]
    fn invalid_bencode_w_stack_underflow_panics() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("dddee"));
        bc_parser.decode_bencode();
    }

    #[test]
    #[ignore = "Changed structure of parser, will recheck"]
    #[should_panic(expected = "Invalid bencode, excess closing delimiters!")]
    fn invalid_bencode_w_stack_overflow_panics() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("ddeee"));
        bc_parser.decode_bencode();
    }

    #[test]
    fn should_parse_positive_int() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("i34e"));
        let result = bc_parser.decode_bencode();
        println!("{:#?}", result);
        if let BenStruct::Int { data } = result {
            assert_eq!(data, 34);
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    #[test]
    fn should_parse_negative_int() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("i-34e"));
        let result = bc_parser.decode_bencode();
        if let BenStruct::Int { data } = result {
            assert_eq!(data, -34);
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    // Strings
    #[test]
    fn should_parse_byte() {
        let raw_bytes = "4:spam";
        let expected_bytes = raw_bytes.split_once(':').unwrap().1;
        let mut bc_parser = BencodeParser::new_w_string(String::from(raw_bytes));
        let result = bc_parser.decode_bencode();
        if let BenStruct::Byte { length, data } = result {
            assert_eq!(
                length as usize,
                data.clone().len(),
                "Length of chars not same as passed len"
            );
            assert_eq!(data.as_str(), expected_bytes, "Wrong chars decoded")
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    // Lists
    #[test]
    fn should_parse_lists() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("li42e4:spami-32ee"));
        let result = bc_parser.decode_bencode();
        let expected_result = vec![
            BenStruct::Int { data: 42 },
            BenStruct::Byte {
                length: 4,
                data: "spam".to_string(),
            },
            BenStruct::Int { data: -32 },
        ];
        if let BenStruct::List { data } = result {
            println!("{:#?}", data);
            let match_count = data
                .clone()
                .iter()
                .zip(expected_result.iter())
                .filter(|&(r, e)| r == e)
                .count();
            assert_eq!(match_count, data.len())
        } else {
            panic!("Invalid data type decoded!")
        }
    }

    #[test]
    fn should_parse_nested_lists() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("li39eli29ee4:spami-32ee"));
        let result = bc_parser.decode_bencode();
        let expected_result = vec![
            BenStruct::Int { data: 39 },
            BenStruct::List {
                data: vec![BenStruct::Int { data: 29 }],
            },
            BenStruct::Byte {
                length: 4,
                data: "spam".to_string(),
            },
            BenStruct::Int { data: -32 },
        ];
        assert_eq!(
            result,
            BenStruct::List {
                data: expected_result
            }
        )
    }

    // Dicts
    #[test]
    fn should_parse_dicts() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("d3:bar4:spam3:fooi45ee "));
        let result = bc_parser.decode_bencode();
        let expected_result = HashMap::from([
            (
                "bar".to_string(),
                BenStruct::Byte {
                    length: 4,
                    data: "spam".to_string(),
                },
            ),
            ("foo".to_string(), BenStruct::Int { data: 45 }),
        ]);

        assert_eq!(
            result,
            BenStruct::Dict {
                data: expected_result
            }
        )
    }

    // New line, carriage return parsing
    #[test]
    fn should_support_new_lines() {
        let mut bc_parser = BencodeParser::new_w_string(String::from("d\n3:foo\n3:bar\ne"));
        let result = bc_parser.decode_bencode();
        let expected_map = HashMap::from([(
            "foo".to_string(),
            BenStruct::Byte {
                length: 3,
                data: "bar".to_string(),
            },
        )]);
        println!("{:#?}", result);
        assert_eq!(result, BenStruct::Dict { data: expected_map });
    }
}
