use std::net::SocketAddr;

use axum::{debug_handler, routing::get, Extension, Json, Router};

use crate::msx::Msx;

pub struct Console {
    addr: SocketAddr,
}

impl Console {
    pub fn new(addr: SocketAddr) -> Self {
        Console { addr }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let msx = Msx::new();
        let app = Router::new().route("/", get(index)).layer(Extension(msx));

        axum::Server::bind(&self.addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

#[debug_handler]
async fn index(Extension(msx): Extension<Msx>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "pc": msx.cpu.pc,
    }))
}
