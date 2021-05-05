
use rocket::{Request, Response, http::{Header, Status, hyper::{header::{CONNECTION, UPGRADE}}}, response::{self, Responder, upgrade::{UpgradeResponder, Upgraded}}};
use tokio::sync::oneshot;
use std::{io::Cursor, pin::Pin};
use tungstenite::protocol::{Role, WebSocketConfig};
use async_trait::async_trait;

pub use tokio_tungstenite::tungstenite;
pub use tokio_tungstenite::WebSocketStream;

/// A Websocket Responder. This will upgrade connection into a WebsocketStream, and return it via
/// a oneshot channel.
#[derive(Debug)]
pub struct RocketWebsocket {
	inner: oneshot::Sender<Pin<Box<WebSocketStream<Upgraded>>>>,
	config: Option<WebSocketConfig>,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for RocketWebsocket {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
	let key = request.headers().get_one("Sec-WebSocket-Key")
		.ok_or(Status::BadRequest)?;
	if request.headers().get_one("Sec-WebSocket-Version").map(|v| v.as_bytes()) != Some(b"13") {
		return Err(Status::BadRequest);
	}

	let mut response = Response::build();
	response.status(Status::SwitchingProtocols);
	response.header(Header::new(CONNECTION.as_str(), "upgrade"));
	response.header(Header::new(UPGRADE.as_str(), "websocket"));
	response.header(Header::new("Sec-WebSocket-Accept", convert_key(key.as_bytes())));
	response.sized_body(None, Cursor::new("Switching protocols to WebSocket"));

	response.upgrade(Box::new(self));

	Ok(response.finalize())
    }
}

/// Turns a Sec-WebSocket-Key into a Sec-WebSocket-Accept.
fn convert_key(input: &[u8]) -> String {
	use sha1::Digest;
	// ... field is constructed by concatenating /key/ ...
	// ... with the string "258EAFA5-E914-47DA-95CA-C5AB0DC85B11" (RFC 6455)
	const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
	let mut sha1 = sha1::Sha1::default();
	sha1.update(input);
	sha1.update(WS_GUID);
	base64::encode(sha1.finalize())
}

#[async_trait]
impl UpgradeResponder for RocketWebsocket {
    async fn on_upgrade(mut self: Box<Self>, upgrade_obj: Upgraded) -> std::io::Result<()> {
	let stream = WebSocketStream::from_raw_socket(upgrade_obj, Role::Server, self.config.take()).await;
	self.inner.send(Box::pin(stream)).map_err(|_|std::io::Error::new(std::io::ErrorKind::Other, "Upgrade reciever hung up"))
    }
}

impl RocketWebsocket {
    /// Creates a new Websocket Responder, and a oneshot to recieve the websocket stream
    ///
    /// ```rust
    /// ```
    pub fn new() -> (Self, oneshot::Receiver<Pin<Box<WebSocketStream<Upgraded>>>>) {
	let (tx, rx) = oneshot::channel();
	(Self {
	    inner: tx,
	    config: None,
	}, rx)
    }
    /// Same as `new`, but allows specifiying a WebSocketConfig to control the configuration
    pub fn with_config(config: WebSocketConfig) -> (Self, oneshot::Receiver<Pin<Box<WebSocketStream<Upgraded>>>>) {
	let (tx, rx) = oneshot::channel();
	(Self {
	    inner: tx,
	    config: Some(config),
	}, rx)
    }
}
