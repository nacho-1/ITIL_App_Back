use tokio::signal::{
    self,
    unix::{signal, SignalKind},
};
use tracing::info;

pub async fn shutdown_signal() {
    // Handle Ctrl+C (SIGINT)
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // Handle SIGTERM (sent by `docker stop`)
    let mut terminate = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate.recv() => {},
    }

    info!("Shutdown signal received");
}
