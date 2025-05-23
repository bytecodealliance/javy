use crate::{
    hold, hold_and_release,
    quickjs::{prelude::MutFn, Ctx, Function, String as JSString, Value},
    to_js_error, val_to_string, Args,
};
use anyhow::{anyhow, Result};

/// Register `btoa` and `atob` global functions for base64 encoding/decoding.
pub(crate) fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();

    globals.set(
        "btoa",
        Function::new(
            this.clone(),
            MutFn::new(move |cx, args| {
                let (cx, args) = hold_and_release!(cx, args);
                btoa(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
            }),
        )?,
    )?;

    globals.set(
        "atob",
        Function::new(
            this.clone(),
            MutFn::new(move |cx, args| {
                let (cx, args) = hold_and_release!(cx, args);
                atob(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
            }),
        )?,
    )?;

    Ok(())
}

/// Encode a string to base64 (btoa - "binary to ASCII")
fn btoa<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (ctx, args) = args.release();
    let args = args.into_inner();

    if args.is_empty() {
        return Err(anyhow!("btoa requires 1 argument"));
    }

    // Get the string to encode
    let input_str = val_to_string(&ctx, args[0].clone())?;
    
    // Check for invalid characters (btoa should only work with "binary strings" - ASCII range 0-255)
    for ch in input_str.chars() {
        if ch as u32 > 255 {
            return Err(anyhow!("InvalidCharacterError: The string to be encoded contains characters outside of the Latin1 range"));
        }
    }

    // Convert string to bytes (each character becomes one byte)
    let bytes: Vec<u8> = input_str.chars().map(|c| c as u8).collect();
    
    // Encode to base64
    let encoded = base64_encode(&bytes);

    let js_string = JSString::from_str(ctx.clone(), &encoded)?;
    Ok(Value::from_string(js_string))
}

/// Decode a base64 string (atob - "ASCII to binary") 
fn atob<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (ctx, args) = args.release();
    let args = args.into_inner();

    if args.is_empty() {
        return Err(anyhow!("atob requires 1 argument"));
    }

    // Get the base64 string to decode
    let input_str = val_to_string(&ctx, args[0].clone())?;
    
    // Remove whitespace (browsers are lenient with whitespace)
    let cleaned = input_str.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    
    // Decode from base64
    let decoded_bytes = base64_decode(&cleaned)
        .map_err(|_| anyhow!("InvalidCharacterError: The string to be decoded is not correctly encoded"))?;

    // Convert bytes back to string (each byte becomes one character)
    let result_string: String = decoded_bytes.into_iter().map(|b| b as char).collect();

    let js_string = JSString::from_str(ctx.clone(), &result_string)?;
    Ok(Value::from_string(js_string))
}

/// Simple base64 encoder implementation
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    let mut i = 0;
    
    while i < input.len() {
        let b1 = input[i];
        let b2 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b3 = if i + 2 < input.len() { input[i + 2] } else { 0 };
        
        let bitmap = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
        
        result.push(ALPHABET[((bitmap >> 18) & 63) as usize] as char);
        result.push(ALPHABET[((bitmap >> 12) & 63) as usize] as char);
        
        if i + 1 < input.len() {
            result.push(ALPHABET[((bitmap >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        
        if i + 2 < input.len() {
            result.push(ALPHABET[(bitmap & 63) as usize] as char);
        } else {
            result.push('=');
        }
        
        i += 3;
    }
    
    result
}

/// Simple base64 decoder implementation
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    const DECODE_TABLE: [u8; 128] = [
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 62,  255, 255, 255, 63,
        52,  53,  54,  55,  56,  57,  58,  59,  60,  61,  255, 255, 255, 64,  255, 255,
        255, 0,   1,   2,   3,   4,   5,   6,   7,   8,   9,   10,  11,  12,  13,  14,
        15,  16,  17,  18,  19,  20,  21,  22,  23,  24,  25,  255, 255, 255, 255, 255,
        255, 26,  27,  28,  29,  30,  31,  32,  33,  34,  35,  36,  37,  38,  39,  40,
        41,  42,  43,  44,  45,  46,  47,  48,  49,  50,  51,  255, 255, 255, 255, 255,
    ];
    
    let input_bytes = input.as_bytes();
    let input_len = input_bytes.len();
    
    // Check for valid length
    if input_len % 4 != 0 {
        return Err(anyhow!("Invalid base64 length"));
    }
    
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < input_len {
        let mut bitmap = 0u32;
        let mut padding = 0;
        
        for j in 0..4 {
            let byte = input_bytes[i + j];
            if byte == b'=' {
                padding += 1;
                bitmap <<= 6;
            } else if byte > 127 {
                return Err(anyhow!("Invalid character"));
            } else {
                let decoded = DECODE_TABLE[byte as usize];
                if decoded == 255 {
                    return Err(anyhow!("Invalid character"));
                }
                bitmap = (bitmap << 6) | (decoded as u32);
            }
        }
        
        result.push((bitmap >> 16) as u8);
        if padding < 2 {
            result.push((bitmap >> 8) as u8);
        }
        if padding < 1 {
            result.push(bitmap as u8);
        }
        
        i += 4;
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Runtime};
    use anyhow::Error;

    #[test]
    fn test_register() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Check that btoa is available
            let result: Value = cx.eval("typeof btoa")?;
            let type_str = val_to_string(&cx, result)?;
            assert_eq!(type_str, "function");
            
            // Check that atob is available
            let result: Value = cx.eval("typeof atob")?;
            let type_str = val_to_string(&cx, result)?;
            assert_eq!(type_str, "function");
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_btoa_basic() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test basic encoding
            let result: Value = cx.eval("btoa('hello')")?;
            let encoded = val_to_string(&cx, result)?;
            assert_eq!(encoded, "aGVsbG8=");
            
            // Test empty string
            let result: Value = cx.eval("btoa('')")?;
            let encoded = val_to_string(&cx, result)?;
            assert_eq!(encoded, "");
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_atob_basic() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test basic decoding
            let result: Value = cx.eval("atob('aGVsbG8=')")?;
            let decoded = val_to_string(&cx, result)?;
            assert_eq!(decoded, "hello");
            
            // Test empty string
            let result: Value = cx.eval("atob('')")?;
            let decoded = val_to_string(&cx, result)?;
            assert_eq!(decoded, "");
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_round_trip() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test round trip encoding/decoding
            let result: Value = cx.eval("atob(btoa('Hello World!'))")?;
            let round_trip = val_to_string(&cx, result)?;
            assert_eq!(round_trip, "Hello World!");
            
            // Test with special characters (ASCII range)
            let result: Value = cx.eval("atob(btoa('ABC123!@#$%^&*()'))")?;
            let round_trip = val_to_string(&cx, result)?;
            assert_eq!(round_trip, "ABC123!@#$%^&*()");
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_btoa_invalid_characters() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test that non-Latin1 characters throw an error
            let result = cx.eval::<Value, _>("btoa('€')"); // Euro symbol (outside Latin1)
            assert!(result.is_err());
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_atob_invalid_input() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test invalid base64 string
            let result = cx.eval::<Value, _>("atob('invalid!')");
            assert!(result.is_err());
            
            // Test wrong length
            let result = cx.eval::<Value, _>("atob('abc')");
            assert!(result.is_err());
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_whitespace_handling() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test that whitespace is ignored in atob
            let result: Value = cx.eval("atob('aGVs\\nbG8=')")?; // "hello" with newline
            let decoded = val_to_string(&cx, result)?;
            assert_eq!(decoded, "hello");
            
            let result: Value = cx.eval("atob('aGVs bG8=')")?; // "hello" with space
            let decoded = val_to_string(&cx, result)?;
            assert_eq!(decoded, "hello");
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_padding_variants() -> Result<()> {
        let config = Config::default();
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test different padding scenarios
            let test_cases = vec![
                ("h", "aA=="),      // 1 byte -> 2 padding
                ("he", "aGU="),     // 2 bytes -> 1 padding  
                ("hel", "aGVs"),    // 3 bytes -> 0 padding
                ("hell", "aGVsbA=="), // 4 bytes -> 2 padding
            ];
            
            for (input, expected) in test_cases {
                let script = format!("btoa('{}')", input);
                let result: Value = cx.eval(script.as_str())?;
                let encoded = val_to_string(&cx, result)?;
                assert_eq!(encoded, expected, "Failed for input: {}", input);
                
                // Test round trip
                let script = format!("atob('{}')", expected);
                let result: Value = cx.eval(script.as_str())?;
                let decoded = val_to_string(&cx, result)?;
                assert_eq!(decoded, input, "Round trip failed for: {}", input);
            }
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_base64_encoder() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"h"), "aA==");
        assert_eq!(base64_encode(b"he"), "aGU=");
        assert_eq!(base64_encode(b"hel"), "aGVs");
    }

    #[test]
    fn test_base64_decoder() -> Result<()> {
        assert_eq!(base64_decode("aGVsbG8=")?, b"hello");
        assert_eq!(base64_decode("")?, b"");
        assert_eq!(base64_decode("aA==")?, b"h");
        assert_eq!(base64_decode("aGU=")?, b"he");
        assert_eq!(base64_decode("aGVs")?, b"hel");
        
        // Test error cases
        assert!(base64_decode("invalid!").is_err());
        assert!(base64_decode("abc").is_err()); // Wrong length
        
        Ok(())
    }
} 