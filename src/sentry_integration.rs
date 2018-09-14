use sentry;

use config;

#[derive(Debug, Deserialize, Clone)]
pub struct SentryConfig {
    pub dsn: String,
}

pub fn init(config: &config::Config) -> Option<sentry::ClientInitGuard> {
    config.sentry.as_ref().map(|config_sentry| {
        println!("initialization support with sentry");
        let result = sentry::init((
            config_sentry.dsn.clone(),
            sentry::ClientOptions {
                release: sentry_crate_release!(),
                ..Default::default()
            },
        ));
        sentry::integrations::panic::register_panic_handler();

        result
    })
}
