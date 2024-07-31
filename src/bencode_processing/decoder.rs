use std::error::Error;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;


// Function to decode a bencoded byte slice
pub fn decode_bencoded_value(encoded_value: &[u8], is_utf8: bool) -> Result<(Value, &[u8]), Box<dyn  Error>> {
    if encoded_value[0].is_ascii_digit() {
        return decode_string(encoded_value, is_utf8);
    }
    let first_byte = encoded_value[0];
    if first_byte == b'i' {
        return decode_number(encoded_value);
    } else if first_byte == b'l' {
        return decode_list(encoded_value, is_utf8);
    } else if first_byte == b'd' {
        return decode_dictionary(encoded_value, is_utf8);
    } else {
        panic!("Unhandled encoded value: {:?}", encoded_value);
    }
}

fn decode_string(encoded_value: &[u8], as_utf_8: bool) -> Result<(Value, &[u8]), Box<dyn  Error>> {
    let mut number: usize = 0;
    let mut i = 0;
    // Read the length of the string
    while i < encoded_value.len() && encoded_value[i].is_ascii_digit() {
        number = number * 10 + (encoded_value[i] - b'0') as usize;
        i += 1;
    }
    // Skip the colon
    if i < encoded_value.len() && encoded_value[i] == b':' {
        i += 1;
    } else {
        return Err("Invalid bencoded string: missing colon".into());
    }
    // Extract the string bytes
    let string_bytes = &encoded_value[i..i + number];
    let decoded_string;

    if as_utf_8 {
        match std::str::from_utf8(string_bytes) {
            Ok(decoded_utf8)=> decoded_string = decoded_utf8.to_string(),
            Err(e) => {return Err(e.into())}

        }
    } else {
        decoded_string = general_purpose::STANDARD.encode(string_bytes);
    }


    Ok((decoded_string.into(), &encoded_value[i + number..]))
}

// Function to decode a bencoded number
fn decode_number(encoded_value: &[u8]) -> Result<(Value, &[u8]), Box<dyn Error>> {
    let end_of_num_index = encoded_value.iter().position(|&x| x == b'e').unwrap();
    let number = &encoded_value[1..end_of_num_index];
    let parsed_number = std::str::from_utf8(number)?.parse::<i64>()?;
    Ok((parsed_number.into(), &encoded_value[end_of_num_index + 1..]))
}

// Function to decode a bencoded dictionary
fn decode_dictionary(mut encoded_value: &[u8], is_utf8: bool) -> Result<(Value, &[u8]), Box<dyn Error>> {
    encoded_value = &encoded_value[1..];
    let mut serde_json_map = serde_json::Map::new();
    loop {
        if encoded_value[0] == b'e' {
            encoded_value = &encoded_value[1..];
            break;
        }
        match decode_string(encoded_value, true) {
            Ok((map_key, remaining)) => {
                encoded_value = remaining;
                match decode_bencoded_value(encoded_value, is_utf8) {
                    Ok((value, remaining)) => {
                        serde_json_map.insert(map_key.as_str().unwrap().to_string(), value);
                        encoded_value = remaining;
                        if remaining.is_empty() {
                            break;
                        }
                    },
                    Err(e) => {
                        print!("Box<dyn Error> decoding dictionary.");
                        return Err(e);
                    }
                }
            },
            Err(e) => {
                print!("Box<dyn Error> decoding dictionary.");
                return Err(e);
            }
        }
    }
    Ok((serde_json_map.into(), encoded_value))
}

// Function to decode a bencoded list
fn decode_list(mut encoded_value: &[u8], is_utf8: bool) -> Result<(Value, &[u8]), Box<dyn Error>> {
    encoded_value = &encoded_value[1..];
    let mut decoded_values = Vec::new();
    loop {
        if encoded_value[0] == b'e' {
            encoded_value = &encoded_value[1..];
            break;
        }
        match decode_bencoded_value(encoded_value, is_utf8) {
            Ok((value, remaining)) => {
                decoded_values.push(value);
                encoded_value = remaining;
            },
            Err(e) => {
                print!("Box<dyn Error> decoding list.");
                return Err(e);
            }
        }
    }
    Ok((decoded_values.into(), encoded_value))
}



// Tests for the decoding functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_list_empty() {
        let encoded_value = b"le";
        let expected = Value::Array(vec![]);
        match decode_bencoded_value(encoded_value, false) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, b"");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }

    #[test]
    fn test_decode_list_basic() {
        let encoded_value = b"l5:helloi52ee";
        let expected = Value::Array(vec![
            Value::String(general_purpose::STANDARD.encode(b"hello")),
            Value::Number(52.into())
        ]);
        match decode_bencoded_value(encoded_value, false) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, b"");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }

    #[test]
    fn test_decode_list_nested() {
        let encoded_value = b"l5:helloi52el5:helloi52eei52ee";
        let expected = Value::Array(vec![
            Value::String(general_purpose::STANDARD.encode(b"hello")),
            Value::Number(52.into()),
            Value::Array(vec![
                Value::String(general_purpose::STANDARD.encode(b"hello")),
                Value::Number(52.into())
            ]),
            Value::Number(52.into())
        ]);
        match decode_bencoded_value(encoded_value, false) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, b"");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }

    #[test]
    fn test_decode_dict_nested() {
        let encoded_value = b"d4:testd7:in_testl5:helloi52el5:helloi52eei52eee";
        let expected = serde_json::json!({
            "test": {
                "in_test": [
                    general_purpose::STANDARD.encode(b"hello"),
                    52,
                    [
                        general_purpose::STANDARD.encode(b"hello"),
                        52
                    ],
                    52
            ]}
        });
        match decode_bencoded_value(encoded_value, false) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, b"");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }
    #[test]
fn test_decode_dict_with_binary_data() {
    let encoded_value = b"ld4:name7:example11:binary_data11:hello\x80worldei52ee";


    let expected = serde_json::json!([{
        "name": general_purpose::STANDARD.encode(b"example"),
        "binary_data": general_purpose::STANDARD.encode(b"hello\x80world")

    }, 52]);
    match decode_bencoded_value(encoded_value, false) {
        Ok((result, remaining)) => {
            assert_eq!(result, expected);
            assert!(remaining.is_empty());
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

}
