use anyhow::Error;
use serde_json;
use serde_json::Value;
use std::env;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.unwrap().0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn decode_string(encoded_value: &str) -> Result<(Value, &str), Error>{
    let colon_index = encoded_value.find(':').unwrap();
    let number_string = &encoded_value[..colon_index];
    let number = number_string.parse::<i64>()?;
    let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
    Ok((string.to_string().into(), &encoded_value[colon_index + 1 + number as usize..]))
}

fn decode_number(encoded_value: &str) -> Result<(Value, &str), Error> {
    let end_of_num_index = encoded_value.find('e').unwrap();
    let number = &encoded_value[1..end_of_num_index];
    let parsed_number = number.parse::<i64>()?;
    Ok((parsed_number.into(), &encoded_value[end_of_num_index + 1..])
)}

fn decode_dictionary(mut encoded_value: &str) -> Result<(Value, &str), Error> {
    encoded_value = &encoded_value[1..];
    let mut serde_json_map = serde_json::Map::new();
    loop {
        if encoded_value.chars().next().unwrap() == 'e' {
            encoded_value = &encoded_value[1..];
            break;
        }

        match decode_bencoded_value(encoded_value) {
            Ok((map_key, remaining)) => {
                encoded_value = remaining;
                match decode_bencoded_value(encoded_value) {
                    Ok((value, remaining)) => {
                        
                        serde_json_map.insert(map_key.as_str().unwrap().to_string(), value);
                        encoded_value = remaining;
                        if remaining.is_empty(){
                            break;
                        }
        
                    },
                    Err(e) => {
                        print!("Error decoding dictionary.");
                        return Err(e);
                    }
                }

                if remaining.is_empty(){
                    break;
                }

            },
            Err(e) => {
                print!("Error decoding dictionary.");
                return Err(e);
            }
        }

    }
    
    
    return Ok((serde_json_map.into(), encoded_value));
}

fn decode_list(mut encoded_value: &str) -> Result<(Value, &str), Error> {
    encoded_value = &encoded_value[1..];
    let mut decoded_values = Vec::new();

    loop {
        if encoded_value.chars().next().unwrap() == 'e' {
            encoded_value = &encoded_value[1..];
            break;
        }
        match decode_bencoded_value(encoded_value) {
            Ok((value, remaining)) => {
                decoded_values.push(value);
                encoded_value = remaining;
                if remaining.is_empty(){
                    break;
                }

            },
            Err(e) => {
                print!("Error decoding list.");
                return Err(e);
            }
        }
    }

    return Ok((decoded_values.into(), encoded_value));
}

fn decode_bencoded_value(encoded_value: &str) -> Result<(Value, &str), Error> {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_digit(10) {
        return decode_string(encoded_value);
    } 
    let first_char = encoded_value.chars().next().unwrap();
    
    if first_char == 'i'{
        return decode_number(encoded_value);

    } else if first_char == 'l'{
        return decode_list(encoded_value);
    } else if first_char == 'd'{
        return decode_dictionary(encoded_value);
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}


// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_list_empty() {
        let encoded_value = "le";
        let expected = Value::Array(vec![]);

        match decode_bencoded_value(encoded_value) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, "");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }

    #[test]
    fn test_decode_list_basic() {
        let encoded_value = "l5:helloi52ee";
        let expected = Value::Array(vec![
            Value::String("hello".to_string()),
            Value::Number(52.into())
        ]);

        match decode_bencoded_value(encoded_value) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, "");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }

    #[test]
    fn test_decode_list_nested() {
        let encoded_value = "l5:helloi52el5:helloi52eei52ee";
        let expected = Value::Array(vec![
            Value::String("hello".to_string()),
            Value::Number(52.into()),
            Value::Array(vec![
                Value::String("hello".to_string()),
                Value::Number(52.into())
            ]),
            Value::Number(52.into())
        ]);

        match decode_bencoded_value(encoded_value) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, "");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }

    #[test]
    fn test_decode_dict_nested() {
        let encoded_value = "d4:testl5:helloi52el5:helloi52eei52eee";
        let expected = serde_json::json!({
            "test": [
                "hello",
                52,
                [
                    "hello",
                    52
                ],
                52
            ]
        });

        match decode_bencoded_value(encoded_value) {
            Ok((result, remaining)) => {
                assert_eq!(result, expected);
                assert_eq!(remaining, "");
            }
            Err(e) => panic!("Test failed: {}", e),
        }
    }
}
