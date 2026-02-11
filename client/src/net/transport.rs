use bevy::prelude::*;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[derive(Resource)]
pub struct NetworkClient {
    pub player_id: u32,
    pub messages: Arc<Mutex<Vec<String>>>,
    pub connected: bool,
    ws_url: String,
    connected_flag: Arc<AtomicBool>,
}

// Manual Send+Sync implementation
// Safe because we only use thread-safe types (Arc<Mutex<_>>)
unsafe impl Send for NetworkClient {}
unsafe impl Sync for NetworkClient {}

impl Default for NetworkClient {
    fn default() -> Self {
        // Player ID will be assigned by server
        let player_id = 0;

        // Determine WebSocket URL
        let ws_url = {
            let window = web_sys::window().expect("no global `window` exists");
            let location = window.location();

            // Get hostname - if it's local development, use localhost:3000
            let hostname = location.hostname().unwrap_or_default();
            let is_local = hostname.is_empty()
                || hostname == "localhost"
                || hostname == "127.0.0.1"
                || hostname.starts_with("localhost.")
                || hostname.starts_with("127.0.0.1");

            let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
            let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };

            if is_local {
                // Force localhost:3000 for local development
                format!("{}://localhost:3000/ws", ws_protocol)
            } else {
                // Production: use Fly.io server
                "wss://battlestar.fly.dev/ws".to_string()
            }
        };

        Self {
            player_id,
            messages: Arc::new(Mutex::new(Vec::new())),
            connected: false,
            ws_url,
            connected_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Resource)]
pub struct WebSocketHandle {
    pub ws: WebSocket,
}

// Manual Send+Sync because we're in WASM (single-threaded)
unsafe impl Send for WebSocketHandle {}
unsafe impl Sync for WebSocketHandle {}

pub fn setup_network(mut commands: Commands) {
    let client = NetworkClient::default();

    let ws_url = client.ws_url.clone();
    info!("Connecting to WebSocket: {}", ws_url);

    // Debug: log hostname detection
    let window = web_sys::window().expect("no global `window` exists");
    let location = window.location();
    let hostname = location.hostname().unwrap_or_default();
    let protocol = location.protocol().unwrap_or_default();
    info!("Detected hostname: '{}', protocol: '{}'", hostname, protocol);

    match WebSocket::new(&ws_url) {
        Ok(ws) => {
            let messages = client.messages.clone();

            // Setup onmessage callback
            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let msg = String::from(txt);
                    if let Ok(mut msgs) = messages.lock() {
                        msgs.push(msg);
                    }
                }
            });
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();

            // Setup onerror callback
            let ws_url_for_error = ws_url.clone();
            let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                error!("WebSocket error connecting to {}: {:?}", ws_url_for_error, e);
                error!("Make sure the server is running on the expected address");
            });
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Setup onopen callback
            let connected_flag = client.connected_flag.clone();
            let onopen_callback = Closure::<dyn FnMut()>::new(move || {
                info!("WebSocket connected!");
                connected_flag.store(true, Ordering::Relaxed);
            });
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();

            // Setup onclose callback
            let ws_url_for_close = ws_url.clone();
            let connected_flag_close = client.connected_flag.clone();
            let onclose_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::CloseEvent| {
                warn!(
                    "WebSocket to {} closed: code={}, reason={}, clean={}",
                    ws_url_for_close,
                    e.code(),
                    e.reason(),
                    e.was_clean()
                );
                connected_flag_close.store(false, Ordering::Relaxed);
            });
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();

            commands.insert_resource(WebSocketHandle { ws });
        }
        Err(e) => {
            error!("Failed to create WebSocket: {:?}", e);
        }
    }

    commands.insert_resource(client);
}

/// Polls the WebSocket connected flag each frame and updates the resource
pub fn poll_connection_state(mut client: ResMut<NetworkClient>) {
    client.connected = client.connected_flag.load(Ordering::Relaxed);
}
