use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use axum::{
    debug_handler,
    extract::Query,
    routing::{get, post},
    Extension, Json, Router,
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
            .route("/api/step", post(step))
            .route("/api/reset", post(reset))
            // .merge(SpaRouter::new("/", "public").index_file("index.html"))
            .layer(Extension(msx));

        axum::Server::bind(&self.addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
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

fn get_status(msx: &Msx) -> Json<serde_json::Value> {
    let instr = Instruction::parse(&msx.cpu.memory, msx.cpu.pc);
    Json(serde_json::json!({
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
    }))
}

#[debug_handler]
async fn index(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let msx = msx.read().unwrap();
    println!("pc: 0x{:04X}", msx.cpu.pc);
    get_status(&msx)
}

#[debug_handler]
async fn step(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let mut msx = msx.write().unwrap();
    println!("pc: 0x{:04X}", msx.cpu.pc);
    msx.cpu.execute_cycle();
    println!("pc: 0x{:04X}", msx.cpu.pc);
    get_status(&msx)
}

#[debug_handler]
async fn reset(Extension(msx): Extension<Arc<RwLock<Msx>>>) -> Json<serde_json::Value> {
    let mut msx = msx.write().unwrap();
    msx.cpu.reset();
    get_status(&msx)
}
