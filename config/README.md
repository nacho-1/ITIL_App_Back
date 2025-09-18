# itil-back-config

This crate contains the `Config` struct that holds the application's configuration settings at runtime, as well as functionality for parsing configuration settings from various sources and building the `Config` struct.

The `Config` struct has fields for the server and database configuration by default and can be extended freely with any application-specific settings:

```rs
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    // add your config settings here…
}
```

Gerust uses [figment](https://crates.io/crates/figment) to populate the `Config` struct from environment variables and TOML files such that:

* the `ServerConfig` that contains interface and port to bind to, is populated from the `APP_SERVER__IP` and `APP_SERVER__PORT` environment variables.
* the `DatabaseConfig` that contains the connection URL for the database is populated from the `APP_DATABASE__URL` environment variable.
* any application-specific configuration values are read from the `app.toml` and environment-specific configuration files such that settings in the environment-specific configuration files override values for the same setting in `app.toml`.
