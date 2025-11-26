use anyhow::{Context, Result};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // Application
    pub app_env: String,
    pub server_host: String,
    pub server_port: u16,
    pub app_secret_key: SecretString,

    // Database
    pub database_url: SecretString,

    // Redis
    pub redis_url: SecretString,

    // Sentry
    pub sentry_dsn: String,

    // Nostr
    pub sabi_nostr_nsec: SecretString,
    pub nostr_relays: Vec<String>,

    // Breez SDK
    pub breez_api_key: SecretString,
    pub breez_mnemonic: SecretString,

    // Paystack
    pub paystack_secret_key: SecretString,

    // Africa's Talking (for USSD)
    pub at_api_key: SecretString,
    pub at_username: String,

    // Admin
    pub default_admin_password: SecretString,

    // Lettre (Email for Admin Alerts)
    pub smtp_username: Option<SecretString>,
    pub smtp_password: Option<SecretString>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub alert_email_from: String,
    pub admin_email_recipient: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load from environment variables. 'config' crate can do more complex loading.
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".into());
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse::<u16>()
            .context("SERVER_PORT must be a valid u16")?;
        let app_secret_key = SecretString::new(
            env::var("APP_SECRET_KEY").context("APP_SECRET_KEY must be set")?,
        );

        let database_url = SecretString::new(
            env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
        );

        let redis_url = SecretString::new(env::var("REDIS_URL").context("REDIS_URL must be set")?);

        let sentry_dsn = env::var("SENTRY_DSN").unwrap_or_default();

        let sabi_nostr_nsec = SecretString::new(
            env::var("SABI_NOSTR_NSEC").context("SABI_NOSTR_NSEC must be set")?,
        );
        let nostr_relays = env::var("NOSTR_RELAYS")
            .unwrap_or_else(|_| "".into())
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let breez_api_key = SecretString::new(
            env::var("BREEZ_API_KEY").context("BREEZ_API_KEY must be set")?,
        );
        let breez_mnemonic = SecretString::new(
            env::var("BREEZ_MNEMONIC").context("BREEZ_MNEMONIC must be set")?,
        );

        let paystack_secret_key = SecretString::new(
            env::var("PAYSTACK_SECRET_KEY").context("PAYSTACK_SECRET_KEY must be set")?,
        );

        let at_api_key = SecretString::new(
            env::var("AT_API_KEY").context("AT_API_KEY must be set")?,
        );
        let at_username = env::var("AT_USERNAME").context("AT_USERNAME must be set")?;

        let default_admin_password = SecretString::new(
            env::var("DEFAULT_ADMIN_PASSWORD").context("DEFAULT_ADMIN_PASSWORD must be set")?,
        );

        let smtp_username = env::var("SMTP_USERNAME").ok().map(SecretString::new);
        let smtp_password = env::var("SMTP_PASSWORD").ok().map(SecretString::new);
        let smtp_host = env::var("SMTP_HOST").ok();
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .map(|s| s.parse::<u16>())
            .transpose()
            .context("SMTP_PORT must be a valid u16 if set")?;

        let alert_email_from = env::var("ALERT_EMAIL_FROM")
            .unwrap_or_else(|_| "Sabi Wallet Alerts <noreply@sabi.money>".into());
        let admin_email_recipient = env::var("ADMIN_EMAIL_RECIPIENT")
            .unwrap_or_else(|_| "admin@sabi.money".into());

        Ok(Self {
            app_env,
            server_host,
            server_port,
            app_secret_key,
            database_url,
            redis_url,
            sentry_dsn,
            sabi_nostr_nsec,
            nostr_relays,
            breez_api_key,
            breez_mnemonic,
            paystack_secret_key,
            at_api_key,
            at_username,
            default_admin_password,
            smtp_username,
            smtp_password,
            smtp_host,
            smtp_port,
            alert_email_from,
            admin_email_recipient,
        })
    }
}
