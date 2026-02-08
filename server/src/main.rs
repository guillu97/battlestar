use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _rx) = broadcast::channel(256);
    let game_state = Arc::new(Mutex::new(GameState::new()));

    let app_state = Arc::new(AppState {
        broadcaster: tx,
        game_state,
    });

    spawn_game_loop(app_state.clone());

    let app = Router::new().route("/ws", get(ws_handler)).with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn spawn_game_loop(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(16));
        loop {
            interval.tick().await;
            let mut gs = state.game_state.lock().await;
            gs.step(1.0 / 60.0);
            if let Ok(payload) = serde_json::to_string(&*gs) {
                let _ = state.broadcaster.send(payload);
            }
        }
    });
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.broadcaster.subscribe();

    loop {
        tokio::select! {
            maybe_msg = socket.recv() => {
                let Some(Ok(msg)) = maybe_msg else {
                    break;
                };

                if let Message::Text(text) = msg {
                    if let Ok(input) = serde_json::from_str::<ClientInput>(&text) {
                        let mut gs = state.game_state.lock().await;
                        gs.apply_input(input);
                    }
                }
            }
            msg = rx.recv() => {
                if let Ok(text) = msg {
                    let _ = socket.send(Message::Text(text.into())).await;
                }
            }
        }
    }
}

struct AppState {
    broadcaster: broadcast::Sender<String>,
    game_state: Arc<Mutex<GameState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientInput {
    player_id: u32,
    thrust: f32,
    rotate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameState {
    ships: Vec<Ship>,
    asteroids: Vec<Asteroid>,
    tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ship {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    rotation: f32,
    color: Color,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Asteroid {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    radius: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl GameState {
    fn new() -> Self {
        Self {
            ships: Vec::new(),
            asteroids: vec![
                Asteroid {
                    id: 1,
                    position: Vec2 { x: 200.0, y: 100.0 },
                    velocity: Vec2 { x: 20.0, y: 15.0 },
                    radius: 16.0,
                },
                Asteroid {
                    id: 2,
                    position: Vec2 { x: -150.0, y: -120.0 },
                    velocity: Vec2 { x: -10.0, y: 25.0 },
                    radius: 24.0,
                },
            ],
            tick: 0,
        }
    }

    fn apply_input(&mut self, input: ClientInput) {
        let ship_exists = self.ships.iter().any(|ship| ship.id == input.player_id);
        
        if !ship_exists {
            let mut rng = rand::rng();
            self.ships.push(Ship {
                id: input.player_id,
                position: Vec2 { x: 0.0, y: 0.0 },
                velocity: Vec2 { x: 0.0, y: 0.0 },
                rotation: 0.0,
                color: Color {
                    r: rng.random_range(0.3..1.0),
                    g: rng.random_range(0.3..1.0),
                    b: rng.random_range(0.3..1.0),
                },
            });
        }
        
        let ship = self
            .ships
            .iter_mut()
            .find(|ship| ship.id == input.player_id)
            .expect("ship exists");

        ship.rotation += input.rotate;
        ship.velocity.x += input.thrust * ship.rotation.cos() * 0.1;
        ship.velocity.y += input.thrust * ship.rotation.sin() * 0.1;
    }

    fn step(&mut self, dt: f32) {
        self.tick = self.tick.wrapping_add(1);

        for ship in &mut self.ships {
            ship.position.x += ship.velocity.x * dt * 60.0;
            ship.position.y += ship.velocity.y * dt * 60.0;
            wrap_position(&mut ship.position);
        }

        for asteroid in &mut self.asteroids {
            asteroid.position.x += asteroid.velocity.x * dt;
            asteroid.position.y += asteroid.velocity.y * dt;
            wrap_position(&mut asteroid.position);
        }

        for ship in &mut self.ships {
            for asteroid in &self.asteroids {
                if distance(ship.position, asteroid.position) < asteroid.radius {
                    ship.velocity.x = -ship.velocity.x;
                    ship.velocity.y = -ship.velocity.y;
                }
            }
        }
    }
}

fn wrap_position(pos: &mut Vec2) {
    let limit = 400.0;
    if pos.x > limit {
        pos.x = -limit;
    } else if pos.x < -limit {
        pos.x = limit;
    }

    if pos.y > limit {
        pos.y = -limit;
    } else if pos.y < -limit {
        pos.y = limit;
    }
}

fn distance(a: Vec2, b: Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
