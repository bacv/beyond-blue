use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use actix_web::{error, web, Responder, Result};

use crate::SharedStore;

pub fn api_config(app: &mut web::ServiceConfig) {
    app.service(web::scope("/api").route("/peer", web::get().to(get_peer_list)));
}

async fn get_peer_list(store: web::Data<SharedStore>) -> Result<impl Responder> {
    log::info!("query");
    let res = store
        .lock()
        .map_err(|_| error::ErrorInternalServerError("storage error"))?
        .get_all()
        .iter()
        .map(PeerInfo::from)
        .collect::<Vec<PeerInfo>>();

    Ok(web::Json(res))
}

#[derive(Serialize, Deserialize)]
pub struct PeerInfo {
    addr: String,
}

impl From<&PeerId> for PeerInfo {
    fn from(peer: &PeerId) -> Self {
        PeerInfo {
            addr: peer.to_string(),
        }
    }
}
