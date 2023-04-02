use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use axum::{
    debug_handler,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::routing::SpaRouter;

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
            .route("/status", get(index))
            .route("/step", post(step))
            .merge(SpaRouter::new("/", "public").index_file("index.html"))
            .layer(Extension(msx));

        axum::Server::bind(&self.addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

fn get_status(msx: &Msx) -> Json<serde_json::Value> {
    let instr = Instruction::parse(&msx.cpu.memory, msx.cpu.pc);
    Json(serde_json::json!({
        "pc": format!("0x{:04X}", msx.cpu.pc),
        "cycles": msx.cpu.cycles,
        "a": format!("0x{:02X}", msx.cpu.a),
        "f": format!("0x{:02X}", msx.cpu.f),
        "b": format!("0x{:02X}", msx.cpu.b),
        "c": format!("0x{:02X}", msx.cpu.c),
        "d": format!("0x{:02X}", msx.cpu.d),
        "e": format!("0x{:02X}", msx.cpu.e),
        "h": format!("0x{:02X}", msx.cpu.h),
        "l": format!("0x{:02X}", msx.cpu.l),
        "hl": format!("0x{:04X}", msx.cpu.get_hl()),
        "pc": format!("0x{:04X}", msx.cpu.pc),
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
