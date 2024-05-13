use std::str::FromStr;
use regex::Regex;
use serde_derive::Deserialize;

const SNAFFLER_RECORDS_REGEX: &str = r"^\[(.+)\\(.+)@(.+)\]\s(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})Z\s\[File\]\s\{(?P<triage_level>Green|Yellow|Red|Black)\}<(?P<rule>.+)\|(R|RW)\|(.*)\|(.+)\|(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})Z>\((?P<filepath>.+)\)\s(?P<match_context>.*)$";

#[derive(Debug, Deserialize)]
pub struct SnafflerRecord {
    pub triage_level: String,
    pub rule: String,
    pub filepath: String,
    pub match_context: String,
}

impl FromStr for SnafflerRecord {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(SNAFFLER_RECORDS_REGEX).unwrap();
        if let Some(caps) = re.captures(s) {
            Ok(SnafflerRecord {
                triage_level: caps["triage_level"].to_string(),
                rule: caps["rule"].to_string(),
                filepath: caps["filepath"].to_string(),
                match_context: caps["match_context"].to_string(),
            })
        } else { 
            Err("Failed to parse string into FileRecord".to_string())
        }
    }
}