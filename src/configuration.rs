#[derive(serde::Deserisalize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserisalize, clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Init the config yaml reader
    let settings = config::Config::builder()
        // Add config values from specific config yaml file
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    // Try to convert the read config values into the Settings type
    settings.try_deserialize::<Settings>()
}
