//! yaps-gui — GUI interface for the YAPS-RS photo sorter (iced).

mod app;
mod messages;
mod settings;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    app::run()
}
