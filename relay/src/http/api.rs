use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use actix_web::{error, web, Responder, Result};

use crate::{RelayInfo, SharedStore};

pub fn api_config(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("/api")
            .route("/peers", web::get().to(get_peer_list))
            .route("/relay", web::get().to(get_relay_info)),
    );
}

async fn get_peer_list(store: web::Data<SharedStore>) -> Result<impl Responder> {
    let res = store
        .lock()
        .map_err(|_| error::ErrorInternalServerError("storage error"))?
        .get_all()
        .iter()
        .map(WebPeerInfo::from)
        .collect::<Vec<WebPeerInfo>>();

    Ok(web::Json(res))
}

async fn get_relay_info(store: web::Data<SharedStore>) -> Result<impl Responder> {
    let res: WebRelayInfo = store
        .lock()
        .map_err(|_| error::ErrorInternalServerError("storage error"))?
        .get_relay()
        .into();

    Ok(web::Json(res))
}

#[derive(Serialize, Deserialize)]
pub struct WebPeerInfo {
    addr: String,
}

impl From<&PeerId> for WebPeerInfo {
    fn from(peer: &PeerId) -> Self {
        Self {
            addr: peer.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct WebRelayInfo {
    peer_id: String,
    ips: Vec<String>,
}

impl From<RelayInfo> for WebRelayInfo {
    fn from(peer: RelayInfo) -> Self {
        Self {
            peer_id: peer.peer_id.to_string(),
            ips: peer.addrs,
        }
    }
}
