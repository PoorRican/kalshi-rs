use crate::{KalshiAuth, KalshiEnvironment, KalshiError};
use crate::ws::{KalshiWsClient, WsMessage, WsSubscriptionParams};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WsReconnectConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_retries: Option<usize>,
}

impl Default for WsReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            max_retries: None,
        }
    }
}

pub struct KalshiWsManager {
    env: KalshiEnvironment,
    auth: Option<KalshiAuth>,
    client: Option<KalshiWsClient>,
    reconnect: WsReconnectConfig,
    desired: HashSet<WsSubscriptionParams>,
    pending: HashMap<u64, WsSubscriptionParams>,
    active_by_sid: HashMap<u64, WsSubscriptionParams>,
}

impl KalshiWsManager {
    pub fn new(env: KalshiEnvironment, auth: Option<KalshiAuth>) -> Self {
        Self {
            env,
            auth,
            client: None,
            reconnect: WsReconnectConfig::default(),
            desired: HashSet::new(),
            pending: HashMap::new(),
            active_by_sid: HashMap::new(),
        }
    }

    pub fn with_reconnect_config(mut self, config: WsReconnectConfig) -> Self {
        self.reconnect = config;
        self
    }

    async fn ensure_connected(&mut self) -> Result<(), KalshiError> {
        if self.client.is_some() {
            return Ok(());
        }

        let client = if let Some(auth) = self.auth.clone() {
            KalshiWsClient::connect_authenticated(self.env.clone(), auth).await?
        } else {
            KalshiWsClient::connect(self.env.clone()).await?
        };

        self.client = Some(client);
        self.resubscribe_all().await?;
        Ok(())
    }

    async fn reconnect_with_backoff(&mut self) -> Result<(), KalshiError> {
        let mut attempts = 0usize;
        let mut delay = self.reconnect.initial_delay;

        loop {
            match self.ensure_connected().await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    attempts += 1;
                    if let Some(max) = self.reconnect.max_retries {
                        if attempts > max {
                            return Err(err);
                        }
                    }
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(self.reconnect.max_delay);
                }
            }
        }
    }

    async fn resubscribe_all(&mut self) -> Result<(), KalshiError> {
        if let Some(client) = self.client.as_mut() {
            self.pending.clear();
            self.active_by_sid.clear();
            for params in self.desired.clone() {
                let id = client.subscribe(params.clone()).await?;
                self.pending.insert(id, params);
            }
        }
        Ok(())
    }

    fn handle_control_message(&mut self, msg: &WsMessage) {
        match msg {
            WsMessage::Subscribed { id: Some(id), sid: Some(sid) } => {
                if let Some(params) = self.pending.remove(id) {
                    self.active_by_sid.insert(*sid, params);
                }
            }
            WsMessage::Unsubscribed { sid: Some(sid), .. } => {
                self.active_by_sid.remove(sid);
            }
            _ => {}
        }
    }

    pub async fn subscribe(&mut self, params: WsSubscriptionParams) -> Result<u64, KalshiError> {
        let params = params.normalized();
        self.desired.insert(params.clone());
        self.ensure_connected().await?;

        let client = self.client.as_mut().ok_or_else(|| KalshiError::Ws("missing ws client".to_string()))?;
        let id = client.subscribe(params.clone()).await?;
        self.pending.insert(id, params);
        Ok(id)
    }

    pub async fn unsubscribe(&mut self, sid: u64) -> Result<u64, KalshiError> {
        if let Some(params) = self.active_by_sid.remove(&sid) {
            self.desired.remove(&params);
        }

        let client = self.client.as_mut().ok_or_else(|| KalshiError::Ws("missing ws client".to_string()))?;
        client.unsubscribe(sid).await
    }

    pub async fn next_message(&mut self) -> Result<WsMessage, KalshiError> {
        loop {
            if let Err(_err) = self.ensure_connected().await {
                self.client = None;
                self.reconnect_with_backoff().await?;
                continue;
            }

            let result = {
                let client = self.client.as_mut().ok_or_else(|| KalshiError::Ws("missing ws client".to_string()))?;
                client.next_message().await
            };

            match result {
                Ok(msg) => {
                    self.handle_control_message(&msg);
                    return Ok(msg);
                }
                Err(_) => {
                    self.client = None;
                    self.reconnect_with_backoff().await?;
                }
            }
        }
    }
}
