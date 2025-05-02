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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uri(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

// impl UriExt for Uri {}
//
// impl UriExt for DocumentUri {}
