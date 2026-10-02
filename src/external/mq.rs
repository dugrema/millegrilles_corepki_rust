use crate::constants::*;
use millegrilles_common_rust::constantes::*;
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::rabbitmq_dao::{ConfigQueue, ConfigRoutingExchange};
use millegrilles_common_rust::v3::impls::messaging_service::MessagingServiceImpl;

pub const QUEUE_TTL_DEFAULT: u32 = 30_000;
pub const QUEUE_TTL_TRANSACTIONS: u32 = 3 * 3_600_000;
pub const QUEUE_TICKER: &str = "job_ticker";
pub const QUEUE_REQUESTS: &str = "requests";
pub const QUEUE_ISSUE: &str = "issue";
pub const QUEUE_TRANSACTIONS: &str = "transactions";
pub const QUEUE_BACKUP: &str = "backup";

pub const REQUEST_ACTION_INFOCERTIFICAT: &str = "infoCertificat";
pub const REQUEST_ACTION_CERTIFICATEBYPK: &str = "certificatParPk";
pub const TRANSACTION_ACTION_SAVE_CERTIFICATE: &str = "certificat";
pub const TRANSACTION_ACTION_NEW_CERTIFICATE: &str = "nouveauCertificat";
pub const COMMAND_ACTION_SIGN_CSR: &str = "signerCsr";
pub const EVENT_KEYMASTER_CERTIFICATE: &str = "certMaitreDesCles";

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
                ConfigRoutingExchange { routing_key: format!("requete.{}.{}", DOMAIN_NAME, REQUEST_ACTION_INFOCERTIFICAT), exchange: Securite::L1Public },
                ConfigRoutingExchange { routing_key: format!("requete.{}.{}", DOMAIN_NAME, REQUEST_ACTION_CERTIFICATEBYPK), exchange: Securite::L1Public },
                // rk_volatils.push(ConfigRoutingExchange {routing_key: format!("evenement.{}.{}", PKI_DOMAINE_CERTIFICAT_NOM, PKI_REQUETE_CERTIFICAT), exchange: niveau.clone()});
            ],
            ttl: Some(QUEUE_TTL_DEFAULT),
            durable: true,
            autodelete: false,
        })?;

    mq.add_named_queue(
        ConfigQueue {
            nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_ISSUE),
            routing_keys: vec![
                ConfigRoutingExchange { routing_key: format!("commande.{}.{}", DOMAIN_NAME, COMMAND_ACTION_SIGN_CSR), exchange: Securite::L1Public },

                // Events
                ConfigRoutingExchange { routing_key: format!("evenement.{}.{}", DOMAIN_KEYMASTER, EVENT_KEYMASTER_CERTIFICATE), exchange: Securite::L1Public },
            ],
            ttl: Some(QUEUE_TTL_DEFAULT),
            durable: true,
            autodelete: false,
        })?;

    mq.add_named_queue(
        ConfigQueue {
            nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_TRANSACTIONS),
            routing_keys: vec![
                ConfigRoutingExchange { routing_key: format!("commande.{}.{}", DOMAIN_NAME, TRANSACTION_ACTION_SAVE_CERTIFICATE), exchange: Securite::L3Protege },
                ConfigRoutingExchange { routing_key: format!("commande.{}.{}", DOMAIN_NAME, TRANSACTION_ACTION_NEW_CERTIFICATE), exchange: Securite::L3Protege },
            ],
            ttl: Some(QUEUE_TTL_TRANSACTIONS),
            durable: true,
            autodelete: false,
        })?;

    mq.add_named_queue(ConfigQueue {
        nom_queue: format!("{}/{}", DOMAIN_NAME, QUEUE_BACKUP),
        routing_keys: vec![
            ConfigRoutingExchange { routing_key: format!("requete.{}.getNombreTransactions", DOMAIN_NAME), exchange: Securite::L2Prive },
            ConfigRoutingExchange { routing_key: format!("commande.{}.{}", DOMAIN_NAME, COMMANDE_DECLENCHER_BACKUP), exchange: Securite::L3Protege },
        ],
        ttl: Some(QUEUE_TTL_DEFAULT),
        durable: true,
        autodelete: true,
    })?;

    Ok(())
}
