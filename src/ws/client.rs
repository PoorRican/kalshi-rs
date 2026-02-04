use crate::auth::KalshiAuth;
use crate::env::{KalshiEnvironment, WS_PATH};
use crate::error::KalshiError;
use crate::ws::types::{
    validate_subscription, WsEnvelope, WsListSubscriptionsCmd, WsMessage, WsSubscribeCmd,
    WsSubscriptionParams, WsUnsubscribeCmd, WsUnsubscribeParams, WsUpdateSubscriptionCmd,
    WsUpdateSubscriptionParams,
};

use futures::{SinkExt, StreamExt};

use tokio_tungstenite::tungstenite::http::{HeaderValue, Request};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
>;

pub struct KalshiWsClient {
    write: futures::stream::SplitSink<WsStream, Message>,
    read: futures::stream::SplitStream<WsStream>,
    next_id: u64,
    authenticated: bool,
}

impl KalshiWsClient {
    /// Connect without auth (public channels only).
    pub async fn connect(env: KalshiEnvironment) -> Result<Self, KalshiError> {
        let (ws_stream, _resp) = connect_async(&env.ws_url)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            next_id: 1,
            authenticated: false,
        })
    }

    /// Connect with auth headers so you can subscribe to private channels.
    pub async fn connect_authenticated(env: KalshiEnvironment, auth: KalshiAuth) -> Result<Self, KalshiError> {
        let mut req: Request<()> = env
            .ws_url
            .into_client_request()
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        // WS signing: timestamp + "GET" + "/trade-api/ws/v2"
        let headers = auth.build_headers("GET", WS_PATH)?;

        req.headers_mut().insert(
            "KALSHI-ACCESS-KEY",
            HeaderValue::from_str(&headers.key).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-SIGNATURE",
            HeaderValue::from_str(&headers.signature).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&headers.timestamp_ms).map_err(|e| KalshiError::Header(e.to_string()))?,
        );

        let (ws_stream, _resp) = connect_async(req)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            next_id: 1,
            authenticated: true,
        })
    }

    /// Subscribe to channels.
    pub async fn subscribe(&mut self, params: WsSubscriptionParams) -> Result<u64, KalshiError> {
        let needs_auth = params.channels.iter().any(|c| c.is_private());
        if needs_auth && !self.authenticated {
            return Err(KalshiError::AuthRequired("WebSocket private channel subscription"));
        }

        validate_subscription(&params)?;

        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsSubscribeCmd {
            id,
            cmd: "subscribe",
            params,
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Unsubscribe from a subscription id.
    pub async fn unsubscribe(&mut self, sid: u64) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsUnsubscribeCmd {
            id,
            cmd: "unsubscribe",
            params: WsUnsubscribeParams { sid },
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Update an existing subscription.
    pub async fn update_subscription(&mut self, params: WsUpdateSubscriptionParams) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsUpdateSubscriptionCmd {
            id,
            cmd: "update_subscription",
            params,
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// List active subscriptions.
    pub async fn list_subscriptions(&mut self) -> Result<u64, KalshiError> {
        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsListSubscriptionsCmd { id, cmd: "list_subscriptions" };
        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Read the next JSON envelope from the stream.
    pub async fn next_envelope(&mut self) -> Result<WsEnvelope, KalshiError> {
        while let Some(msg) = self.read.next().await {
            let msg = msg.map_err(|e| KalshiError::Ws(e.to_string()))?;
            match msg {
                Message::Text(s) => return Ok(serde_json::from_str::<WsEnvelope>(&s)?),
                Message::Binary(b) => {
                    return Ok(serde_json::from_slice::<WsEnvelope>(&b)?);
                }
                Message::Ping(payload) => {
                    self.write
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|e| KalshiError::Ws(e.to_string()))?;
                    self.write
                        .flush()
                        .await
                        .map_err(|e| KalshiError::Ws(e.to_string()))?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => return Err(KalshiError::Ws("websocket closed".to_string())),
                _ => {}
            }
        }
        Err(KalshiError::Ws("websocket stream ended".to_string()))
    }

    /// Read the next JSON message and parse into a typed WsMessage.
    pub async fn next_message(&mut self) -> Result<WsMessage, KalshiError> {
        let env = self.next_envelope().await?;
        env.into_message()
    }
}

#[cfg(test)]
mod tests {
    use crate::ws::types::WsChannel;

    #[test]
    fn private_channel_check() {
        assert!(WsChannel::Fill.is_private());
        assert!(WsChannel::OrderbookDelta.is_private());
        assert!(WsChannel::MarketPositions.is_private());
        assert!(WsChannel::Communications.is_private());
        assert!(WsChannel::OrderGroupUpdates.is_private());

        assert!(!WsChannel::Ticker.is_private());
        assert!(!WsChannel::TickerV2.is_private());
        assert!(!WsChannel::Trade.is_private());
        assert!(!WsChannel::MarketLifecycleV2.is_private());
        assert!(!WsChannel::Multivariate.is_private());
    }
}
