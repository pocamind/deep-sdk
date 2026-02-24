use std::{ops::{Add, AddAssign}, str::FromStr};

use serde::{Deserialize, Deserializer, de};

use std::path::Path;

use crate::{error, model::opt::OptionalGroup, model::req::Requirement};

/// The parsed representation of a reqfile
#[derive(Clone, Debug)]
pub struct Reqfile {
    pub general: Vec<Requirement>,
    pub post: Vec<Requirement>,

    pub optional: Vec<OptionalGroup>
}

impl Add for Reqfile {
    type Output = Reqfile;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            general: self.general.iter().chain(rhs.general.iter()).cloned().collect(),
            post: self.post.iter().chain(rhs.post.iter()).cloned().collect(),
            optional: self.optional.iter().chain(rhs.optional.iter()).cloned().collect(),
        }
    }
}

impl AddAssign for Reqfile {
    fn add_assign(&mut self, rhs: Self) {
        self.general.extend(rhs.general);
        self.post.extend(rhs.post);
        self.optional.extend(rhs.optional);
    }
}

impl Reqfile {
    pub fn parse_str(content: &str) -> error::Result<Self> {
        crate::parse::reqfile::parse_reqfile_str(content)
    }

    pub fn from_file(path: &Path) -> error::Result<Self> {
        crate::parse::reqfile::parse_reqfile(path)
    }

    pub fn generate(&self) -> String {
        crate::parse::reqfile::gen_reqfile(self)
    }

    /// Retrieve an iterator containing the required requirements
    pub fn req_iter(&self) -> impl Iterator<Item = &Requirement> {
        self.general.iter().chain(self.post.iter())
    }
}

impl FromStr for Reqfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::reqfile::parse_reqfile_str(s)
            .map_err(|e| format!("Failed to parse requirement: {}", e))
    }
}

impl<'de> Deserialize<'de> for Reqfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}
