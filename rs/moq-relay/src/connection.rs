use crate::Cluster;
use tracing::{info,debug,warn};

pub struct Connection {
    pub id: u64,
    pub session: web_transport::Session,
    pub cluster: Cluster,
    pub token: moq_token::Claims,
}

impl Connection {
    #[tracing::instrument("conn", skip_all, fields(id = self.id, path = %self.token.path))]
    pub async fn run(mut self) {
        info!(?self.token, "🔍 Inside Connection::run() ");
        let mut session = match moq_lite::Session::accept(self.session).await {
            Ok(session) => session,
            Err(err) => {
                warn!(?err, "failed to accept session");
                return;
            }
        };
        // 🔍 Logged subscription handling
        if let Some(subscribe) = self.token.subscribe {
            let full = format!("{}{}", self.token.path, subscribe);

            debug!(
                id = self.id,
                path = %self.token.path,
                subscribe = %subscribe,
                full_path = %full,
                "Processing subscription"
            );

            let primary = if full.is_empty() || full.ends_with("/") {
                debug!("Consuming prefix from primary: {}", full);
                self.cluster.primary.consume_prefix(&full)
            } else {
                debug!("Consuming exact from primary: {}", full);
                self.cluster.primary.consume_exact(&full)
            };

            session.publish_prefix(&subscribe, primary);

            if !self.token.cluster {
                let secondary = if full.is_empty() || full.ends_with("/") {
                    debug!("Consuming prefix from secondary: {}", full);
                    self.cluster.secondary.consume_prefix(&full)
                } else {
                    debug!("Consuming exact from secondary: {}", full);
                    self.cluster.secondary.consume_exact(&full)
                };

                session.publish_prefix(&subscribe, secondary);
            }
        }

        // Publish all broadcasts produced by the session to the local origin.
        if let Some(publish) = self.token.publish {
            let full = format!("{}{}", self.token.path, publish);

            let cluster = match self.token.cluster {
                true => &mut self.cluster.secondary,
                false => &mut self.cluster.primary,
            };

            if full.is_empty() || full.ends_with("/") {
                let produced = session.consume_prefix(&publish);
                cluster.publish_prefix(&full, produced);
            } else {
                let produced = session.consume_exact(&publish);
                cluster.publish_prefix(&full, produced);
            }
        }

        let err = session.closed().await;
        info!(?err, "connection terminated");
    }
}