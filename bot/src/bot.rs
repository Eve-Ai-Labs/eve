use crypto::ed25519::private::PrivateKey;
use eyre::Result;
use orchestrator_client::ClientWithKey;
use std::{ops::Deref, time::Duration};
use tracing::{info, warn};
use types::ai::request::{History, Role};

pub struct Bot {
    client: ClientWithKey,
    delay: Duration,
    max_history: usize,
}

impl Bot {
    pub fn new(client: orchestrator_client::Client, delay_secs: u64, max_history: usize) -> Self {
        let key = PrivateKey::generate();
        info!("Creating bot: {}", key.public_key());

        Self {
            client: ClientWithKey::with_key_and_client(key, client),
            delay: Duration::from_secs(delay_secs),
            max_history,
        }
    }

    pub fn reset(&mut self) {
        info!("Resetting bot: {}", self.client.public_key());
        self.client =
            ClientWithKey::with_key_and_client(PrivateKey::generate(), self.client.deref().clone());
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting bot: {}", self.client.public_key());
        self.client.history.push(History {
            content: "Hi!".to_owned(),
            role: Role::User,
        });

        loop {
            self.airdrop().await?;

            let last_message = self.client.history.pop().unwrap().content;
            info!("Asking: {}", last_message);
            self.invert_history();
            let id = self.client.query(last_message).await?;
            let query = self.client.answer_wait(&id, None).await?;
            self.client.history = query.as_history();

            if self.client.history.len() > self.max_history {
                self.client.history.remove(0);
            }
            tokio::time::sleep(self.delay).await;
        }
    }

    fn invert_history(&mut self) {
        for row in self.client.history.iter_mut() {
            if row.role == Role::User {
                row.role = Role::Assistant;
            } else {
                row.role = Role::User;
            }
        }
    }

    async fn airdrop(&self) -> Result<()> {
        let balance = self.client.balance().await?;
        if balance < 1_000_000 {
            info!("Airdropping...");
            if let Err(err) = self.client.airdrop().await {
                warn!("Failed to airdrop: {}", err);
            }
        }
        Ok(())
    }
}
