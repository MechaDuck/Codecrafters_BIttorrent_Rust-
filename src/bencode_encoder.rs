use serde_json::Value;

use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose};

pub fn encode_value(serde_json_object: &Value) -> Result<Vec<u8>> {
    match serde_json_object {
        Value::Number(_) => encode_number(serde_json_object),
        Value:: String(_) => encode_string(serde_json_object, false),
        Value::Array(_) => encode_list(serde_json_object),
        Value::Object(_) => encode_dict(serde_json_object),
        _ => {return Err(anyhow!("Datatype not supported for bencoding..."))}


    }
}

pub fn encode_list(serde_json_object: &Value) -> Result<Vec<u8>> {
    let decoded_array = serde_json_object.as_array().ok_or_else(|| anyhow!("Value could not be parsed as array"))?;
    let mut encoded_list: Vec<u8> = b"l".to_vec();
    for element in decoded_array {
        match encode_value(element){
            Ok(result) => {
                encoded_list.extend_from_slice(&result);
            },
            Err(e) => {
                Err(anyhow!("Error encoding list. {}", e))?
            }
        }
    }

    encoded_list.extend_from_slice(b"e");
    return Ok(encoded_list)
}

pub fn encode_dict(serde_json_object: &Value) -> Result<Vec<u8>> {
    let decoded_array = serde_json_object.as_object().ok_or_else(|| anyhow!("Value could not be parsed as array"))?;
    let mut encoded_dict: Vec<u8> = b"d".to_vec();
    for (key, value) in decoded_array {
        match encode_string(&Value::String(key.clone()), true){
            Ok(result) => {
                encoded_dict.extend_from_slice(&result);
            },
            Err(e) => {
                Err(anyhow!("Error encoding dict key: {}", e))?
            }
        }

        match encode_value(value){
            Ok(result) => {
                encoded_dict.extend_from_slice(&result);
            },
            Err(e) => {
                Err(anyhow!("Error encoding dict value: {}", e))?
            }
        }
    }

    encoded_dict.extend_from_slice(b"e");
    return Ok(encoded_dict)
}



pub fn encode_string(serde_json_object: &Value, is_utf8: bool) -> Result<Vec<u8>> {

    let decoded_string = serde_json_object.as_str().ok_or_else(|| anyhow!("Value could not be parsed as str"))?;

    let string_bytes: Vec<u8>;

    if is_utf8 {
        string_bytes = decoded_string.as_bytes().to_vec();
    } else {
        // decode base64 encoding
        string_bytes = general_purpose::STANDARD.decode(decoded_string).map_err(|e| anyhow!(e))?;
    }


    // add prefix for string
    let length_prefix = format!("{0}:", string_bytes.len()).into_bytes();

    let mut bencoded_string = Vec::with_capacity(length_prefix.len() + string_bytes.len());

    bencoded_string.extend_from_slice(&length_prefix);
    bencoded_string.extend_from_slice(&string_bytes);


    return Ok(bencoded_string)

}

pub fn encode_number(serde_json_object: &Value) -> Result<Vec<u8>> {

    let decoded_number = serde_json_object.as_i64().ok_or_else(|| anyhow!("Value could not be parsed as i64"))?;
    // decode base64 encoding
    let bytes = decoded_number.to_string().as_bytes().to_vec();
    let mut bencoded_string: Vec<u8> = Vec::with_capacity(2 + bytes.len());
    bencoded_string.extend_from_slice(b"i");
    bencoded_string.extend_from_slice(&bytes);
    bencoded_string.extend_from_slice(b"e");

    return Ok(bencoded_string)

}


// Tests for the decoding functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_string() -> Result<()>{
        helper_test_string(b"hello\x80".to_vec(), b"6:hello\x80".to_vec(), "Test with non utf8 string" );
        helper_test_string(b"hello".to_vec(), b"5:hello".to_vec(), "Test with utf8 string" );
        Ok(())
    }

    fn helper_test_string(given: Vec<u8>, expectation: Vec<u8>, testname: &str) {

        let decoded_json = &Value::String(general_purpose::STANDARD.encode(given));
        match encode_string(decoded_json, false) {
            Ok(result) => {
                assert_eq!(result, expectation, "{testname}");
                println!("{} passed", testname);
            },
            Err(e) => panic!("Test failed. Name: {}, Error: {}", testname, e),
        }
    }

    #[test]
    fn test_encode_number() {
        let decoded_json = &Value::Number(52.into());
        let expected = b"i52e";

        match encode_number(decoded_json) {
            Ok(result) => {
                assert_eq!(result, expected);
            },
            Err(e) => panic!("Test failed: {}", e),
        }

    }

    #[test]
    fn test_encode_list() {
        let given = serde_json::json!([general_purpose::STANDARD.encode("test"), 104]);
        helper_test_complex(&given, b"l4:testi104ee".to_vec(), "simple list");

    }

    #[test]
    fn test_encode_nested_list() {
        let given = serde_json::json!([general_purpose::STANDARD.encode("test"), 104, [23, general_purpose::STANDARD.encode("nested")], 203]);
        helper_test_complex(&given, b"l4:testi104eli23e6:nestedei203ee".to_vec(), "simple list");

    }



    #[test]
    fn test_encode_nested_dict() {
        let given = serde_json::json!({"testkey": [general_purpose::STANDARD.encode("test"), 104, [23, general_purpose::STANDARD.encode("nested")], 203]});
        helper_test_complex(&given, b"d7:testkeyl4:testi104eli23e6:nestedei203eee".to_vec(), "simple list");

    }
    fn helper_test_complex(given: &Value, expectation: Vec<u8>, testname: &str) {

        let decoded_json = given;
        match encode_value(decoded_json) {
            Ok(result) => {
                println!("Compare: \n{} \n{}",  std::str::from_utf8(&result).unwrap(), std::str::from_utf8(&expectation).unwrap());

                assert_eq!(result, expectation, "{testname}");
                println!("{} passed", testname);
            },
            Err(e) => panic!("Test failed. Name: {}, Error: {}", testname, e),
        }
    }


}