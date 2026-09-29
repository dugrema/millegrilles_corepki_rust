use crate::constants::*;
use millegrilles_common_rust::constantes::*;
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::rabbitmq_dao::{ConfigQueue, ConfigRoutingExchange};
use millegrilles_common_rust::v3::impls::messaging_service::MessagingServiceImpl;

pub const QUEUE_TTL_DEFAULT: u32 = 30_000;
pub const QUEUE_TICKER: &str = "job_ticker";
pub const QUEUE_REQUESTS: &str = "requests";
pub const QUEUE_COMMANDS: &str = "commands";
pub const QUEUE_TRANSACTIONS: &str = "transactions";
pub const QUEUE_BACKUP: &str = "backup";

pub fn init_queues(mq: &MessagingServiceImpl) -> Result<(), CommonError> {
    // Configure the queues and add to messaging service (will spawn consumer threads)
    mq.add_named_queue(
        ConfigQueue {
            nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_TICKER),
            routing_keys: vec![
                ConfigRoutingExchange { routing_key: "evenement.ceduleur.ping".to_string(), exchange: Securite::L1Public }
            ],
            ttl: Some(QUEUE_TTL_DEFAULT),
            durable: true,
            autodelete: true,
        })?;

    mq.add_named_queue(
        ConfigQueue {
            nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_REQUESTS),
            routing_keys: vec![
                // ConfigRoutingExchange { routing_key: format!("requete.{}.{}", DOMAIN_NAME, REQUEST_USER_CATEGORIES), exchange: Securite::L2Prive },
            ],
            ttl: Some(QUEUE_TTL_DEFAULT),
            durable: true,
            autodelete: false,
        })?;

    mq.add_named_queue(
        ConfigQueue {
            nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_TRANSACTIONS),
            routing_keys: vec![
                // ConfigRoutingExchange { routing_key: format!("commande.{}.{}", DOMAIN_NAME, TRANSACTION_SAUVEGARDER_CATEGORIE_USAGER), exchange: Securite::L2Prive },
            ],
            ttl: Some(QUEUE_TTL_DEFAULT),
            durable: true,
            autodelete: false,
        })?;

    mq.add_named_queue(ConfigQueue {
        nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_BACKUP),
        routing_keys: vec![
            // ConfigRoutingExchange { routing_key: format!("requete.{}.getNombreTransactions", DOMAIN_NAME), exchange: Securite::L2Prive },
        ],
        ttl: Some(QUEUE_TTL_DEFAULT),
        durable: true,
        autodelete: true,
    })?;


    Ok(())
}
