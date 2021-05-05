//! A simple echo websocket example
//!
//! Spawns a rocket server, with two routes; '/', a static file, and '/echo', a websocket route.
//! The main file will use JS to open a websocket connection back to the server, on the echo route.
//! The websocket will simply repeat any message back to the client.
//!
//! Use `cargo run --example echo` to run

use rocket_tungstenite::RocketWebsocket;
use rocket::{futures::{StreamExt, future}, get, response::content::Html, routes};

#[get("/echo")]
async fn echo() -> RocketWebsocket {
    let (ret, rx) = RocketWebsocket::new();

    tokio::spawn(async move {
        if let Ok(ws_stream) = rx.await {
            let (write, read) = ws_stream.split();
            read.take_while(|m| future::ready(m.as_ref().unwrap().is_text()))
                .forward(write)
                .await
                .expect("failed to forward message");
        }
    });
    ret
}

#[get("/")]
fn index() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>WebSocket Echo Server</title>
    </head>
    <body>
        <h1>Echo Server</h1>
        <p id="status">Connecting...</p>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <div id="lines"></div>
        <script type="text/javascript">
            const lines = document.getElementById('lines');
            const text = document.getElementById('text');
            const status = document.getElementById('status');
            const ws = new WebSocket('ws://' + location.host + '/echo');
            ws.onopen = function() {
                status.innerText = 'Connected :)';
            };
            ws.onclose = function() {
                status.innerText = 'Disconnected :(';
                lines.innerHTML = '';
            };
            ws.onmessage = function(msg) {
                const line = document.createElement('p');
                line.innerText = msg.data;
                lines.prepend(line);
            };
            send.onclick = function() {
                ws.send(text.value);
                text.value = '';
            };
        </script>
    </body>
</html>"#)
}

#[tokio::main]
async fn main() -> Result<(), rocket::Error> {
    rocket::build().mount("/", routes![index, echo]).launch().await
}
