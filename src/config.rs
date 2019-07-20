use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

/// The configuration options available with this backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    /// If a link on the internet is encountered, should we still try to check
    /// if it's valid? Defaults to `false` because this has a big performance
    /// impact.
    pub follow_web_links: bool,
    /// Are we allowed to link to files outside of the book's source directory?
    pub traverse_parent_directories: bool,
    #[serde(with = "regex_serde")]
    pub exclude: Vec<Regex>,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    /// The number of seconds a cached result is valid for.
    #[serde(default = "default_cache_timeout")]
    pub cache_timeout: u64,
}

impl Config {
    /// The default cache timeout (around 12 hours).
    pub const DEFAULT_CACHE_TIMEOUT: Duration =
        Duration::from_secs(60 * 60 * 12);
    pub const DEFAULT_USER_AGENT: &'static str =
        concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION"));

    pub fn should_skip(&self, link: &str) -> bool {
        self.exclude.iter().any(|pat| pat.is_match(link))
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            follow_web_links: false,
            traverse_parent_directories: false,
            exclude: Vec::new(),
            user_agent: default_user_agent(),
            cache_timeout: Config::DEFAULT_CACHE_TIMEOUT.as_secs(),
        }
    }
}

fn default_cache_timeout() -> u64 { Config::DEFAULT_CACHE_TIMEOUT.as_secs() }
fn default_user_agent() -> String { Config::DEFAULT_USER_AGENT.to_string() }

mod regex_serde {
    use regex::Regex;
    use serde::{
        de::{Deserialize, Deserializer, Error},
        ser::{SerializeSeq, Serializer},
    };

    #[allow(clippy::ptr_arg)]
    pub fn serialize<S>(re: &Vec<Regex>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = ser.serialize_seq(Some(re.len()))?;

        for pattern in re {
            seq.serialize_element(pattern.as_str())?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Vec<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Vec::<String>::deserialize(de)?;
        let mut patterns = Vec::new();

        for pat in raw {
            let re = Regex::new(&pat).map_err(D::Error::custom)?;
            patterns.push(re);
        }

        Ok(patterns)
    }
}

impl PartialEq for Config {
    fn eq(&self, other: &Config) -> bool {
        let Config {
            follow_web_links,
            traverse_parent_directories,
            ref exclude,
            ref user_agent,
            cache_timeout,
        } = self;

        *follow_web_links == other.follow_web_links
            && *traverse_parent_directories == other.traverse_parent_directories
            && exclude.len() == other.exclude.len()
            && *user_agent == other.user_agent
            && *cache_timeout == other.cache_timeout
            && exclude
                .iter()
                .zip(other.exclude.iter())
                .all(|(l, r)| l.as_str() == r.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;
    const CONFIG: &str = r#"follow-web-links = true
traverse-parent-directories = true
exclude = ["google\\.com"]
user-agent = "Internet Explorer"
cache-timeout = 3600
"#;

    #[test]
    fn deserialize_a_config() {
        let should_be = Config {
            follow_web_links: true,
            traverse_parent_directories: true,
            exclude: vec![Regex::new(r"google\.com").unwrap()],
            user_agent: String::from("Internet Explorer"),
            cache_timeout: 3600,
        };

        let got: Config = toml::from_str(CONFIG).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn round_trip_config() {
        let deserialized: Config = toml::from_str(CONFIG).unwrap();
        let reserialized = toml::to_string(&deserialized).unwrap();

        assert_eq!(reserialized, CONFIG);
    }
}
