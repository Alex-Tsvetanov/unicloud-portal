use tracing_subscriber::EnvFilter;
use unicloud_portal::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=info")))
        .init();
    let cfg = Config::from_env()?;
    let bind = cfg.bind.clone();
    if cfg.issuer.is_none() {
        tracing::warn!("APP_SSO_ISSUER is not set: the issuer is worked out from each request (development only)");
    }
    let st = unicloud_portal::build_state(cfg).await?;
    unicloud_portal::ops::spawn_cleanup(st.clone());
    let app = unicloud_portal::build_app(st).await?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "UniCloud Portal listening");
    axum::serve(listener, app).with_graceful_shutdown(shutdown()).await?;
    Ok(())
}

/// Finish in-flight requests before the replica stops (deploys, scale-in)
async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
    tracing::info!("shutting down");
}
