use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub server_ssh_string: String,
}

impl TestConfig {
    pub fn parse_toml(toml: &str) -> anyhow::Result<Self> {
        let config: Self = toml::from_str(toml)?;
        Ok(config)
    }
    pub fn parse_config() -> anyhow::Result<Self> {
        let config = std::fs::read_to_string("test_config.toml")?;
        Self::parse_toml(&config)
    }
}