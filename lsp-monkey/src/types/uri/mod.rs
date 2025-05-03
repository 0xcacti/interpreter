pub mod error;
pub use error::UriError;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

lazy_static! {
    static ref URI_REGEX: Regex =
        Regex::new(r"^(([^:/?#]+?):)?(\/\/([^/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?").unwrap();
    static ref SCHEME_REGEX: Regex = Regex::new(r"^\w[\w\d+.-]*$").unwrap();
}

fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

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
pub struct Uri(UriComponents);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentUri(UriComponents);

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

impl UriExt for UriComponents {
    fn parse(s: &str, strict: bool) -> Result<Self, UriError> {
        let captures = URI_REGEX
            .captures(s)
            .ok_or_else(|| UriError::new(format!("Failed to parse URI: {}", s)))?;

        let raw_scheme = captures.get(2).map_or("", |m| m.as_str()).to_string();
        let raw_authority = captures.get(4).map_or("", |m| m.as_str()).to_string();
        let raw_path = captures.get(5).map_or("", |m| m.as_str()).to_string();
        let raw_query = captures.get(7).map_or("", |m| m.as_str()).to_string();
        let raw_fragment = captures.get(9).map_or("", |m| m.as_str()).to_string();

        let scheme = if raw_scheme.is_empty() {
            if strict {
                return Err(UriError::new(
                    "URI scheme is required in strict mode".into(),
                ));
            } else {
                "file".to_string()
            }
        } else {
            raw_scheme.to_string()
        };

        if !SCHEME_REGEX.is_match(&scheme) {
            return Err(UriError::new(format!("Invalid URI scheme: {}", scheme)));
        }

        let authority = percent_decode(&raw_authority)?;
        let mut path = percent_decode(&raw_path)?;
        let query = percent_decode(&raw_query)?;
        let fragment = percent_decode(&raw_fragment)?;
        match scheme.as_str() {
            "file" | "http" | "https" => {
                if path.is_empty() {
                    path = "/".into();
                } else if !path.starts_with('/') {
                    path.insert(0, '/');
                }
            }
            _ => {}
        }

        if strict {
            if !authority.is_empty() {
                if !path.is_empty() && !path.starts_with('/') {
                    return Err(UriError::new(
                        "If a URI has an authority component, the path must be empty or start with a slash".into(),
                    ));
                }
            } else {
                if path.starts_with("//") {
                    return Err(UriError::new(
                        "If a URI does not have an authority component, the path must not start with two slashes".into(),
                    ));
                }
            }
        }

        Ok(UriComponents {
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }

    fn file(path: &Path) -> Result<Self, UriError> {
        let mut path_str = if is_windows() {
            let mut s = path.to_string_lossy().replace('\\', "/");
            if s.len() >= 2 && s.as_bytes()[1] == b':' {
                s = format!("/{}", s);
            }
            s
        } else {
            path.to_string_lossy().to_string()
        };

        let mut authority = String::new();

        if path_str.len() >= 2 && path_str.starts_with("//") {
            let idx = path_str[2..].find('/').map(|i| i + 2);
            match idx {
                Some(idx) => {
                    authority = path_str[2..idx].to_string();
                    path_str = path_str[idx..].to_string();
                    if path_str.is_empty() {
                        path_str = "/".to_string();
                    }
                }
                None => {
                    authority = path_str[2..].to_string();
                    path_str = "/".to_string();
                }
            }
        }

        let uri_string = if !authority.is_empty() {
            format!("file://{}{}", authority, path_str)
        } else {
            format!("file://{}", path_str)
        };

        Ok(Uri(uri_string))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_uri() {
        let uri = UriComponents::parse("http://example.com/path?query#fragment", false).unwrap();
        assert_eq!(uri.scheme, "http");
        assert_eq!(uri.authority, "example.com");
        assert_eq!(uri.path, "/path");
        assert_eq!(uri.query, "query");
        assert_eq!(uri.fragment, "fragment");
    }

    #[test]
    fn test_http_uri_strict() {
        let uri = UriComponents::parse("http://example.com/path?query#fragment", true).unwrap();
        assert_eq!(uri.scheme, "http");
        assert_eq!(uri.authority, "example.com");
        assert_eq!(uri.path, "/path");
    }

    #[test]
    fn test_invalid_uri() {
        assert!(UriComponents::parse("invalid_uri", false).is_err());
    }

    #[test]
    fn test_strict_invalid_no_scheme() {
        // no scheme + strict ⇒ error
        assert!(UriComponents::parse("foo/bar", true).is_err());
    }

    #[test]
    fn test_default_file_scheme_non_strict() {
        // no scheme + non-strict ⇒ file:// fallback
        let uri = UriComponents::parse("foo/bar", false).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/foo/bar");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[test]
    fn test_https_no_path() {
        // http(s) with empty path ⇒ "/" by reference‐resolution
        let uri = UriComponents::parse("https://example.com", false).unwrap();
        assert_eq!(uri.scheme, "https");
        assert_eq!(uri.authority, "example.com");
        assert_eq!(uri.path, "/");
    }

    #[test]
    fn test_percent_decode_in_path() {
        // percent‐encoded path segment
        let uri = UriComponents::parse("http://example.com/a%20b", false).unwrap();
        assert_eq!(uri.path, "/a b");
    }

    #[test]
    fn test_file_uri_windows_drive() {
        let uri = UriComponents::parse("file:///C:/Windows/System32", false).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/C:/Windows/System32");
    }

    #[test]
    fn test_file_uri_unc() {
        let uri = UriComponents::parse("file://SERVER/share/folder", false).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "SERVER");
        assert_eq!(uri.path, "/share/folder");
    }

    #[test]
    fn test_invalid_percent_encoding() {
        // malformed % escape
        assert!(matches!(
            UriComponents::parse("http://example.com/%ZZ", false),
            Err(_)
        ));
    }

    #[test]
    fn test_invalid_scheme_characters() {
        // scheme must match /^\w[\w\d+.-]*$/
        assert!(UriComponents::parse("ht!tp://example.com", false).is_err());
    }

    #[test]
    fn test_strict_double_slash_no_authority() {
        // no authority, but path starts with "//"
        assert!(UriComponents::parse("//foo/bar", true).is_err());
    }
}
