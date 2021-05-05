# rocket-tungstenite

This crate allows [`rocket`](https://github.com/the10thwiz/Rocket/tree/websockets)
servers to accept websocket connections, backed by
[`tungstenite`](https://docs.rs/tungstenite).

The `RocketWebsocket` responder implements Rocket's `Responder` trait, and
automatically handles converting the connection into a websocket.
By default, the `RocketWebsocket` will return an `UpgradeRequired` status to the
client if the request cannot be upgraded to a websocket. For most applications,
this is perfectly acceptable. Alternatively, expecially if the route is dynamic
and might not exist, it should return a `Result<Websocket, Status>`, which behaves
exactly as expected.

## Rocket support

This crate is only supported by a specific branch of a fork, namely the
websocket branch of [the10thwiz/Rocket](https://github.com/the10thwiz/Rocket/tree/websockets).
Because of this, this crate will not be published until Rocket reaches 5.0
and the http upgrade mechanism is merged into it.

## Example

```rust
use rocket::{get, futures::{SinkExt, StreamExt}};
use rocket_tungstenite::{RocketWebsocket, Message};

#[get("/socket")]
async fn join_room() -> Websocket {
    let (ret, rx) = Websocket::new();
    tokio::spawn(async move {
        if let Ok(mut ws) = rx.await {
            ws.send(Message::text("Example message")).await.expect("Error");
            //ws.send("example_message").await?;
            let mut ctr: usize = 0;
            while let Some(Ok(message)) = ws.next().await {
                ws.send(Message::text(format!("Recieved: {}", message))).await.expect("Error");
                ctr+= 1;
                if ctr > 5 {
                    return;
                }
            }
        }
    });
    ret
}
```

See [examples/echo.rs] for a full example.

## TODO

[-] Create optional request gaurd that checks headers

