#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Init the config yaml reader
    let settings = config::Config::builder()
        // Add config values from specific config yaml file
        .add_source(config::File::new(
            "configuration/base",
            config::FileFormat::Yaml,
        ))
        // 2. Add "local" configuration (secrets, ignored by git)
        // .required(false) means the app won't crash if this file is missing
        // Differing from the book as they just push everything to VCS...tsk tsk
        .add_source(config::File::with_name("configuration/local").required(false))
        // 3. Add Environment Variables (The ultimate override)
        // This allows us to set APP_DATABASE__PASSWORD in production
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    // Try to convert the read config values into the Settings type
    settings.try_deserialize::<Settings>()
}
