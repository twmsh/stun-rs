use log::debug;

#[cfg(windows)]
pub async fn wait_shutdown() {
    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            debug!("recv ctrl_c, shutdown")
        }
        Err(e) => {
            debug!("error, ctrl_c, {:?}", e);
        }
    }
}

#[cfg(unix)]
pub async fn wait_shutdown() {
    use tokio::signal::unix::SignalKind;
    async fn terminate() -> std::io::Result<()> {
        let signal = match tokio::signal::unix::signal(SignalKind::terminate()) {
            Ok(v) => v,
            Err(e) => {
                debug!("error, signal, {:?}", e);
                return Err(e);
            }
        };

        let _ = signal.recv().await;
        Ok(())
    }

    tokio::select! {
        s = terminate() => {
            debug!("recv unix terminate signal, {:?}",s);
        },
        s = tokio::signal::ctrl_c() => {
            debug!("recv unix ctrl_c signal, {:?}",s);
        }
    }
}
