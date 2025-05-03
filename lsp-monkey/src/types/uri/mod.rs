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

fn percent_encode_strict(input: &str) -> String {
    let mut out = String::new();
    for &b in input.as_bytes() {
        let c = b as char;
        if is_unreserved(c) || is_sub_delim(c) {
            out.push(c);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    out
}

fn reference_resolution(scheme: &str, mut path: String) -> String {
    match scheme {
        "file" | "http" | "https" => {
            if path.is_empty() {
                "/".into()
            } else if !path.starts_with('/') {
                path.insert(0, '/');
                path
            } else {
                path
            }
        }
        _ => path,
    }
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

impl UriExt for Uri {
    fn parse(s: &str, strict: bool) -> Result<Self, UriError> {
        let components = UriComponents::parse(s, strict)?;
        Ok(Uri(components))
    }

    fn file(path: &Path) -> Result<Self, UriError> {
        let components = UriComponents::file(path)?;
        Ok(Uri(components))
    }

    fn fs_path(&self) -> Result<PathBuf, UriError> {
        self.0.fs_path()
    }

    fn scheme(&self) -> &str {
        self.0.scheme()
    }

    fn authority(&self) -> &str {
        self.0.authority()
    }

    fn path(&self) -> &str {
        self.0.path()
    }

    fn query(&self) -> &str {
        self.0.query()
    }

    fn fragment(&self) -> &str {
        self.0.fragment()
    }

    fn with(&self, components: UriComponents) -> Self {
        Uri(components)
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }

    fn to_json(&self) -> UriComponents {
        self.0.to_json()
    }

    fn normalize(&self) -> String {
        self.0.normalize()
    }
}

impl UriExt for DocumentUri {
    fn parse(s: &str, strict: bool) -> Result<Self, UriError> {
        let components = UriComponents::parse(s, strict)?;
        Ok(DocumentUri(components))
    }

    fn file(path: &Path) -> Result<Self, UriError> {
        let components = UriComponents::file(path)?;
        Ok(DocumentUri(components))
    }

    fn fs_path(&self) -> Result<PathBuf, UriError> {
        self.0.fs_path()
    }

    fn scheme(&self) -> &str {
        self.0.scheme()
    }

    fn authority(&self) -> &str {
        self.0.authority()
    }

    fn path(&self) -> &str {
        self.0.path()
    }

    fn query(&self) -> &str {
        self.0.query()
    }

    fn fragment(&self) -> &str {
        self.0.fragment()
    }

    fn with(&self, components: UriComponents) -> Self {
        DocumentUri(components)
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }

    fn to_json(&self) -> UriComponents {
        self.0.to_json()
    }

    fn normalize(&self) -> String {
        self.0.normalize()
    }
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
            }
            if raw_path.contains('/') {
                "file".to_string()
            } else {
                return Err(UriError::new("Invalid URI: no scheme and no path".into()));
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

        path = reference_resolution(&scheme, path);

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
            path.to_string_lossy().replace('\\', "/")
        } else {
            path.to_string_lossy().to_string()
        };

        let mut authority = String::new();

        if path_str.starts_with("//") {
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

        path_str = reference_resolution("file", path_str);

        Ok(UriComponents {
            scheme: "file".to_string(),
            authority,
            path: path_str,
            query: String::new(),
            fragment: String::new(),
        })
    }

    fn fs_path(&self) -> Result<PathBuf, UriError> {
        let mut value =
            if !self.authority.is_empty() && self.path.len() > 1 && self.scheme == "file" {
                format!("//{}{}", self.authority, self.path)
            } else if self.path.as_bytes()[0] == b'/'
                && (self.path.as_bytes()[1] >= b'A' && self.path.as_bytes()[1] <= b'Z'
                    || self.path.as_bytes()[1] >= b'a' && self.path.as_bytes()[1] <= b'z')
                && self.path.as_bytes()[2] == b':'
            {
                self.path[1..].to_string()
            } else {
                self.path.clone()
            };

        if cfg!(windows) {
            value = value.replace('/', "\\");
        }

        Ok(PathBuf::from(value))
    }

    fn scheme(&self) -> &str {
        &self.scheme
    }

    fn authority(&self) -> &str {
        &self.authority
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn query(&self) -> &str {
        &self.query
    }

    fn fragment(&self) -> &str {
        &self.fragment
    }

    fn with(&self, components: UriComponents) -> Self {
        UriComponents {
            scheme: components.scheme,
            authority: components.authority,
            path: components.path,
            query: components.query,
            fragment: components.fragment,
        }
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        if !self.scheme.is_empty() {
            out.push_str(&self.scheme);
            out.push(':');
        }

        if !self.authority.is_empty() || self.scheme == "file" {
            out.push_str("//");
        }

        if !self.authority.is_empty() {
            let mut auth = self.authority.clone();
            if let Some(at) = auth.find('@') {
                let userinfo = &auth[..at];
                let hostport = &auth[at + 1..];
                if let Some(colon) = userinfo.rfind(':') {
                    out.push_str(&percent_encode(&userinfo[..colon]));
                    out.push(':');
                    out.push_str(&percent_encode(&userinfo[colon + 1..]));
                } else {
                    out.push_str(&percent_encode(userinfo));
                }
                out.push('@');
                auth = hostport.to_string();
            }

            let auth_lower = auth.to_ascii_lowercase();
            if let Some(colon_idx) = auth_lower.find(':') {
                out.push_str(&percent_encode(&auth_lower[..colon_idx]));
                out.push_str(&auth_lower[colon_idx..]);
            } else {
                out.push_str(&percent_encode(&auth_lower));
            }
        }

        if !self.path.is_empty() {
            let mut path = self.path.clone();
            if path.len() >= 3
                && path.as_bytes()[0] == b'/'
                && path.as_bytes()[2] == b':'
                && (path.as_bytes()[1] as char).is_ascii_uppercase()
            {
                let lc = (path.as_bytes()[1] as char).to_ascii_lowercase();
                path.replace_range(1..2, &lc.to_string());
            } else if path.len() >= 2
                && path.as_bytes()[1] == b':'
                && (path.as_bytes()[0] as char).is_ascii_uppercase()
            {
                let lc = (path.as_bytes()[0] as char).to_ascii_lowercase();
                path.replace_range(0..1, &lc.to_string());
            }
            out.push_str(&percent_encode(&path));
        }

        if !self.query.is_empty() {
            out.push('?');
            out.push_str(&percent_encode_strict(&self.query));
        }

        if !self.fragment.is_empty() {
            out.push('#');
            out.push_str(&percent_encode_strict(&self.fragment));
        }

        out
    }

    fn to_json(&self) -> UriComponents {
        return self.clone();
    }

    fn normalize(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode_unreserved() {
        let input = "AZaz09-._~";
        assert_eq!(percent_encode(input), input);
    }

    #[test]
    fn test_percent_encode_sub_delims() {
        let input = "!$&'()*+,;=";
        assert_eq!(percent_encode(input), input);
    }

    #[test]
    fn test_percent_encode_gen_delims() {
        let input = ":/?#[]@";
        assert_eq!(percent_encode(input), input);
    }

    #[test]
    fn test_percent_encode_space_and_unicode() {
        assert_eq!(percent_encode(" "), "%20");
        assert_eq!(percent_encode("©"), "%C2%A9");
    }

    #[test]
    fn test_strict_parse_empty_http_path() {
        let uri =
            UriComponents::parse("https://example.com", true).expect("should parse in strict mode");
        assert_eq!(uri.scheme, "https");
        assert_eq!(uri.authority, "example.com");
        assert_eq!(uri.path, "/");
    }

    #[test]
    fn test_non_strict_double_slash_allowed() {
        let uri =
            UriComponents::parse("//some/thing", false).expect("non-strict should allow //foo/bar");
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "some");
        assert_eq!(uri.path, "/thing");
    }

    #[test]
    fn test_to_string_file_unc_lowercases_authority() {
        let uri = UriComponents {
            scheme: "file".into(),
            authority: "SERVER".into(),
            path: "/share/folder".into(),
            query: "".into(),
            fragment: "".into(),
        };
        assert_eq!(uri.to_string(), "file://server/share/folder",);
    }

    #[test]
    fn test_reference_resolution_empty() {
        assert_eq!(reference_resolution("file", "".into()), "/");
        assert_eq!(reference_resolution("http", "".into()), "/");
        assert_eq!(reference_resolution("https", "".into()), "/");
    }

    #[test]
    fn test_reference_resolution_prepend_slash() {
        assert_eq!(reference_resolution("file", "foo/bar".into()), "/foo/bar");
        assert_eq!(reference_resolution("https", "baz".into()), "/baz");
    }

    #[test]
    fn test_reference_resolution_already_slash() {
        assert_eq!(reference_resolution("file", "/foo".into()), "/foo");
    }

    #[test]
    fn test_reference_resolution_other_scheme() {
        assert_eq!(reference_resolution("ftp", "foo".into()), "foo");
    }

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
        assert!(UriComponents::parse("foo/bar", true).is_err());
    }

    #[test]
    fn test_default_file_scheme_non_strict() {
        let uri = UriComponents::parse("foo/bar", false).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/foo/bar");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[test]
    fn test_https_no_path() {
        let uri = UriComponents::parse("https://example.com", false).unwrap();
        assert_eq!(uri.scheme, "https");
        assert_eq!(uri.authority, "example.com");
        assert_eq!(uri.path, "/");
    }

    #[test]
    fn test_percent_decode_in_path() {
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
        assert!(matches!(
            UriComponents::parse("http://example.com/%ZZ", false),
            Err(_)
        ));
    }

    #[test]
    fn test_invalid_scheme_characters() {
        assert!(UriComponents::parse("ht!tp://example.com", false).is_err());
    }

    #[test]
    fn test_strict_double_slash_no_authority() {
        assert!(UriComponents::parse("//foo/bar", true).is_err());
    }

    #[test]
    fn test_file_unix_absolute() {
        let uri = UriComponents::file(Path::new("/foo/bar")).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/foo/bar");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[test]
    fn test_file_unix_root() {
        let uri = UriComponents::file(Path::new("/")).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[test]
    fn test_file_relative_path() {
        let uri = UriComponents::file(Path::new("foo/bar")).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/foo/bar");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn test_file_windows_drive() {
        let uri = UriComponents::file(Path::new("C:\\Windows\\System32")).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "");
        assert_eq!(uri.path, "/C:/Windows/System32");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn test_file_windows_unc() {
        let uri = UriComponents::file(Path::new(r"\\SERVER\share\folder")).unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, "SERVER");
        assert_eq!(uri.path, "/share/folder");
        assert!(uri.query.is_empty());
        assert!(uri.fragment.is_empty());
    }

    #[test]
    fn test_to_string_basic_http() {
        let uri = UriComponents::parse("http://Example.COM/foo/bar", false).unwrap();
        // authority should be lower-cased
        assert_eq!(uri.to_string(), "http://example.com/foo/bar");
    }

    #[test]
    fn test_to_string_with_port() {
        let uri = UriComponents {
            scheme: "http".into(),
            authority: "Example.COM:8080".into(),
            path: "/".into(),
            query: String::new(),
            fragment: String::new(),
        };
        assert_eq!(uri.to_string(), "http://example.com:8080/");
    }

    #[test]
    fn test_to_string_with_userinfo() {
        let uri = UriComponents {
            scheme: "http".into(),
            authority: "User:Pass@Example.COM".into(),
            path: "/p".into(),
            query: String::new(),
            fragment: String::new(),
        };
        // userinfo must be percent-encoded (but letters and ":" in password are allowed)
        assert_eq!(uri.to_string(), "http://User:Pass@example.com/p");
    }

    #[test]
    fn test_to_string_percent_encoding() {
        let uri = UriComponents {
            scheme: "https".into(),
            authority: "example.com".into(),
            path: "/a b/©".into(),
            query: "q r".into(),
            fragment: "f#g".into(),
        };
        let out = uri.to_string();
        // spaces → %20, © → percent-encoded, '#' in fragment → %23 only in path/query
        assert!(out.starts_with("https://example.com/a%20b/%C2%A9"));
        assert!(out.contains("?q%20r"));
        assert!(out.contains("#f%23g"));
    }

    #[test]
    fn test_to_string_file_drive_letter() {
        let uri = UriComponents {
            scheme: "file".into(),
            authority: "".into(),
            path: "/C:/Foo Bar".into(),
            query: String::new(),
            fragment: String::new(),
        };
        assert_eq!(uri.to_string(), "file:///c:/Foo%20Bar");
    }

    #[test]
    fn test_to_string_file_unc() {
        let uri = UriComponents {
            scheme: "file".into(),
            authority: "SERVER".into(),
            path: "/share/folder".into(),
            query: String::new(),
            fragment: String::new(),
        };
        assert_eq!(uri.to_string(), "file://server/share/folder");
    }

    #[test]
    fn test_to_json_roundtrip() {
        let before =
            UriComponents::parse("https://Example.COM:8080/a%20b?x=1#frag", false).unwrap();

        let json = before.to_json();
        assert_eq!(json, before);
    }

    #[test]
    fn test_normalize_matches_to_string() {
        let uri = UriComponents::parse("HTTPS://Example.COM:8080/foo/Bar?Q=2#FrAg", false).unwrap();
        assert_eq!(uri.normalize(), uri.to_string());
    }

    #[test]
    fn test_normalize_canonicalizes() {
        let uri = UriComponents::parse("example.com/foo", false).unwrap();
        assert_eq!(uri.normalize(), "file:///example.com/foo");
    }
}
