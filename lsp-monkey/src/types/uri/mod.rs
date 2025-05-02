pub mod error;
pub use error::UriError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriComponents {
    pub scheme: String,
    pub authority: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

const GEN_DELIMS: &str = ":/?#[]@";
const SUB_DELIMS: &str = "!$&'()*+,;=";
const UNRESERVED: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

#[inline]
fn is_gen_delim(c: char) -> bool {
    GEN_DELIMS.contains(c)
}

#[inline]
fn is_sub_delim(c: char) -> bool {
    SUB_DELIMS.contains(c)
}

#[inline]
fn is_unreserved(c: char) -> bool {
    UNRESERVED.contains(c)
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for &b in input.as_bytes() {
        let c = b as char;
        if is_unreserved(c) || is_gen_delim(c) || is_sub_delim(c) {
            out.push(c);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }

    out
}

fn percent_decode(input: &str) -> Result<String, UriError> {
    let bytes = {
        let mut v = Vec::with_capacity(input.len());
        let mut i = 0;
        let bs = input.as_bytes();
        while i < bs.len() {
            if bs[i] == b'%' {
                if i + 2 >= bs.len() {
                    return Err(UriError::new("incomplete percent-encoded data".into()));
                }
                let hex = &input[i + 1..i + 3];
                let byte = u8::from_str_radix(hex, 16)
                    .map_err(|_| UriError::new(format!("invalid percent-encoded byte: {}", hex)))?;
                v.push(byte);
                i += 3;
            } else {
                v.push(bs[i]);
                i += 1;
            }
        }
        v
    };

    String::from_utf8(bytes)
        .map_err(|_| UriError::new("percent-decoded data is not valid UTF-8".into()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Uri(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentUri(String);

pub trait UriExt {
    fn parse(s: &str, strict: bool) -> Result<Self, UriError>
    where
        Self: Sized;

    fn file(path: &Path) -> Result<Self, UriError>
    where
        Self: Sized;

    fn fs_path(&self) -> Result<PathBuf, UriError>;

    fn scheme(&self) -> &str;
    fn authority(&self) -> &str;
    fn path(&self) -> &str;
    fn query(&self) -> &str;
    fn fragment(&self) -> &str;
    fn with(&self, components: UriComponents) -> Self;
    fn to_string(&self) -> String;
    fn to_json(&self) -> UriComponents;
    fn normalize(&self) -> String;
}

impl UriExt for Uri {
    fn parse(s: &str, strict: bool) -> Result<Self, UriError> {
        todo!("Implement URI parsing logic");
    }

    fn file(path: &Path) -> Result<Self, UriError> {
        todo!("Implement file URI creation logic");
    }

    fn fs_path(&self) -> Result<PathBuf, UriError> {
        todo!("Implement file system path extraction logic");
    }
    fn scheme(&self) -> &str {
        todo!("Implement scheme extraction logic");
    }

    fn authority(&self) -> &str {
        todo!("Implement authority extraction logic");
    }

    fn path(&self) -> &str {
        todo!("Implement path extraction logic");
    }

    fn query(&self) -> &str {
        todo!("Implement query extraction logic");
    }

    fn fragment(&self) -> &str {
        todo!("Implement fragment extraction logic");
    }

    fn with(&self, components: UriComponents) -> Self {
        todo!("Implement URI construction logic");
    }

    fn to_string(&self) -> String {
        todo!("Implement URI to string conversion logic");
    }

    fn to_json(&self) -> UriComponents {
        todo!("Implement URI to JSON conversion logic");
    }

    fn normalize(&self) -> String {
        todo!("Implement URI normalization logic");
    }
}
//
// impl UriExt for DocumentUri {}
