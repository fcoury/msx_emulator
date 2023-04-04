use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Query, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use serde_json::json;

use crate::{components::instruction::Instruction, msx::Msx};

pub struct Console {
    addr: SocketAddr,
    rom: PathBuf,
}

impl Console {
    pub fn new(addr: SocketAddr, rom: PathBuf) -> Self {
        Console { addr, rom }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let mut msx = Msx::new();
        msx.load_bios(self.rom.clone())?;

        let msx = Arc::new(RwLock::new(msx));
        let app = Router::new()
            .route("/api/status", get(index))
            .route("/api/program", get(program))
            .route("/api/memory", get(memory))
            .route("/api/vram", get(vram))
            .route("/api/step", post(step))
            .route("/api/reset", post(reset))
            .route("/ws", get(ws_handler))
            // .merge(SpaRouter::new("/", "public").index_file("index.html"))
            .layer(Extension(msx));

        axum::Server::bind(&self.addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;

        Ok(())
    }
}

#[debug_handler]
async fn ws_handler(
    Extension(msx): Extension<Arc<RwLock<Msx>>>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let result = ws.on_upgrade(move |socket| handle_socket(socket, addr, msx));

    // Add a log here to see if the upgrade process is successful
    tracing::info!("WebSocket upgrade result: {:?}", result);

    result
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, msx: Arc<RwLock<Msx>>) {
    // By splitting the socket, we can send and receive at the same time.
    let (mut sender, mut receiver) = socket.split();

    // This task will receive messages from the client and process them
    let recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // Process the message and send updates as needed
            process_and_send_updates(msx.clone(), msg, who, &mut sender).await;
        }
        cnt
    });

    // Wait for the recv_task to complete
    match recv_task.await {
        Ok(b) => println!("Received {} messages", b),
        Err(b) => println!("Error receiving messages {:?}", b),
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {} destroyed", who);
}

/// Process the incoming message, send updates as needed
async fn process_and_send_updates(
    msx: Arc<RwLock<Msx>>,
    msg: Message,
    who: SocketAddr,
    sender: &mut SplitSink<WebSocket, Message>,
) {
    // Process the message based on your application requirements
    match msg {
        // Handle text messages
        Message::Text(json) => {
            let msg: serde_json::Value = serde_json::from_str(&json).unwrap();
            let data = msg.get("data");
            tracing::info!("Received message from {}: {:?}", who, msg);
            let response = match msg.get("type").unwrap().as_str().unwrap() {
                "status" => {
                    let status = get_status(&msx.read().unwrap());
                    json!({ "type": "status", "data": status })
                }
                "memory" => {
                    let empty = json!("");
                    let hash = match data {
                        Some(data) => data.get("hash").unwrap_or(&empty).as_str().unwrap(),
                        None => "",
                    };
                    let memory = &msx.write().unwrap().delta_memory(hash);
                    json!({ "type": "memory", "data": memory })
                }
                "program" => {
                    let msx = msx.read().unwrap();
                    let mut program = Vec::new();
                    let mut pc = msx.cpu.pc;

                    loop {
                        if pc.checked_add(1).is_none() {
                            break;
                        }

                        if program.len() > 100 {
                            break;
                        }

                        let instr = Instruction::parse(&msx.cpu.memory, pc);
                        program.push(json!({
                            "address": format!("{:04X}", pc),
                            "instruction": instr.name(),
                            "hexcontents": instr.opcode_with_args(),
                        }));
                        pc += instr.len() as u16;
                    }

                    json!({ "type": "program", "data": program })
                }
                "vram" => json!({ "type": "vram", "data": msx.read().unwrap().vdp.vram.to_vec() }),
                "step" => {
                    msx.write().unwrap().cpu.execute_cycle();
                    json!({ "type": "status", "data": get_status(&msx.read().unwrap()) })
                }
                // "reset" => {
                //     msx.write().unwrap().cpu.reset();
                //     json!({ "type": "status", "data": get_status(&msx.read().unwrap()) })
                // }
                _ => json!({ "type": "error", "message": "Invalid request" }),
            };
            let response = response.to_string();
            if let Err(e) = sender.send(Message::Text(response)).await {
                eprintln!("Error sending WebSocket message: {:?}", e);
            }
        }

        // Handle the close message
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {} somehow sent close message without CloseFrame", who);
            }
        }

        // Handle ping and pong messages
        Message::Ping(v) => {
            println!(">>> {} sent ping with {:?}", who, v);
        }
        Message::Pong(v) => {
            println!(">>> {} sent pong with {:?}", who, v);
        }

        // Handle binary messages if needed
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
    }
}

#[debug_handler]
async fn memory(
    Extension(msx): Extension<Arc<RwLock<Msx>>>,
    Query(query): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let mut msx = msx.write().unwrap();
    Json(json!(
        msx.delta_memory(query.get("hash").unwrap_or(&"".to_string()))
    ))
}

#[debug_handler]
async fn vram(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let msx = msx.read().unwrap();
    Json(json!(msx.vdp.vram.to_vec()))
}

#[debug_handler]
async fn program(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<Vec<serde_json::Value>> {
    let msx = msx.read().unwrap();
    let mut program = Vec::new();
    let mut pc = msx.cpu.pc;

    loop {
        if pc.checked_add(1).is_none() {
            break;
        }

        if program.len() > 100 {
            break;
        }

        let instr = Instruction::parse(&msx.cpu.memory, pc);
        program.push(json!({
            "address": format!("{:04X}", pc),
            "instruction": instr.name(),
            "hexcontents": instr.opcode_with_args(),
        }));
        pc += instr.len() as u16;
    }

    Json(program)
}

fn get_status(msx: &Msx) -> serde_json::Value {
    let instr = Instruction::parse(&msx.cpu.memory, msx.cpu.pc);
    serde_json::json!({
        "registers": vec![
            json!({"name": "pc", "value": format!("{:04X}", msx.cpu.pc)}),
            json!({"name": "a", "value": format!("{:02X}", msx.cpu.a)}),
            json!({"name": "f", "value": format!("{:02X}", msx.cpu.f)}),
            json!({"name": "b", "value": format!("{:02X}", msx.cpu.b)}),
            json!({"name": "c", "value": format!("{:02X}", msx.cpu.c)}),
            json!({"name": "d", "value": format!("{:02X}", msx.cpu.d)}),
            json!({"name": "e", "value": format!("{:02X}", msx.cpu.e)}),
            json!({"name": "h", "value": format!("{:02X}", msx.cpu.h)}),
            json!({"name": "l", "value": format!("{:02X}", msx.cpu.l)}),
            json!({"name": "af", "value": format!("{:04X}", msx.cpu.get_af())}),
            json!({"name": "bc", "value": format!("{:04X}", msx.cpu.get_bc())}),
            json!({"name": "de", "value": format!("{:04X}", msx.cpu.get_de())}),
            json!({"name": "hl", "value": format!("{:04X}", msx.cpu.get_hl())}),
            json!({"name": "sp", "value": format!("{:04X}", msx.cpu.sp)}),
        ],
        "pc": format!("{:04X}", msx.cpu.pc),
        "cycles": msx.cpu.cycles,
        "instruction": instr.name(),
        "opcode": instr.opcode_with_args(),
    })
}

#[debug_handler]
async fn index(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let msx = msx.read().unwrap();
    println!("pc: 0x{:04X}", msx.cpu.pc);
    Json(get_status(&msx))
}

#[debug_handler]
async fn step(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let mut msx = msx.write().unwrap();
    println!("pc: 0x{:04X}", msx.cpu.pc);
    msx.cpu.execute_cycle();
    println!("pc: 0x{:04X}", msx.cpu.pc);
    Json(get_status(&msx))
}

#[debug_handler]
async fn reset(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let mut msx = msx.write().unwrap();
    msx.cpu.reset();
    Json(get_status(&msx))
}
