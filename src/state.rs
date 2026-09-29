use std::sync::Arc;
use millegrilles_common_rust::openssl::pkey::{PKey, Private};
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::mongo_dao::MongoDaoImpl;
use millegrilles_common_rust::tokio::task::JoinSet;
use millegrilles_common_rust::tokio_util::sync::CancellationToken;
use millegrilles_common_rust::v3::{ChiffrageService, ConfigService};
use millegrilles_common_rust::v3::facades::message_outbound::MessageOutboundFacade;
use crate::Cli;

pub struct AppContext {
    pub join_set: JoinSet<()>,
    pub config: Arc<dyn ConfigService>,
    pub chiffrage: Arc<dyn ChiffrageService>,
    pub mongo: Arc<MongoDaoImpl>,
    pub outbound: Arc<MessageOutboundFacade>,
    pub shutdown_token: CancellationToken,
}

impl AppContext {
    pub async fn new(_cli: &Cli, _master_key: Option<PKey<Private>>) -> Result<AppContext, CommonError> {
        todo!()
    }
}
