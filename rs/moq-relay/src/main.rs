use tracing::{info};
mod auth;
mod cluster;
mod config;
mod connection;
mod web;

pub use auth::*;
pub use cluster::*;
pub use config::*;
pub use connection::*;
pub use web::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    let addr = config.server.listen.unwrap_or("[::]:443".parse().unwrap());
    let mut server = config.server.init()?;
    let client = config.client.init()?;
    let auth = config.auth.init()?;
    let fingerprints = server.fingerprints().to_vec();

    let cluster = Cluster::new(config.cluster, client);

    // Start cluster background task
    let cloned = cluster.clone();
    tokio::spawn(async move {
        cloned.run().await.expect("cluster failed");
    });

    // 🔁 Periodically log active broadcasts
    let producer = cluster.origin_producer().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            producer.log_active_broadcasts();
        }
    });

    // Create and run web server
    let web = Web::new(WebConfig {
        bind: addr,
        fingerprints,
        cluster: cluster.clone(),
    });

    tokio::spawn(async move {
        web.run().await.expect("failed to run web server");
    });

    tracing::info!(%addr, "listening");

    let mut conn_id = 0;

    while let Some(conn) = server.accept().await {
         info!("📥 Received WebTransport connection from: {}", conn.remote_address());
let mut token = match auth.validate(conn.url()) {
    Ok(token) => token,
    Err(err) => {
        tracing::warn!(?err, "failed to validate token");
        conn.close(1, b"invalid token");
        continue;
    }
};

// 🛠️ Add defaults if missing
/*if token.publish.is_none() {
    token.publish = Some("my-broadcast".into());
}
if token.subscribe.is_none() {
    token.subscribe = Some("my-broadcast".into());
}*/

        let conn = Connection {
            id: conn_id,
            session: conn.into(),
            cluster: cluster.clone(),
            token,
        };
        info!(id = conn_id, "🚀 Spawning Connection::run");
        conn_id += 1;
        tokio::spawn(conn.run());
    }

    Ok(())
}