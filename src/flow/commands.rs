use millegrilles_common_rust::bson::doc;
use millegrilles_common_rust::certificats::VerificateurPermissions;
use millegrilles_common_rust::common_messages::BackupEvent;
use millegrilles_common_rust::constantes::*;
use millegrilles_common_rust::v3::{BackupService, MessagingService, PkiService, PresenceService};
use millegrilles_common_rust::v3::facades::message_inbound::MessageValidated;
use millegrilles_common_rust::v3::facades::message_outbound::MessageOutboundFacade;
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::millegrilles_cryptographie::x509::EnveloppeCertificat;
use millegrilles_common_rust::mongo_dao::MongoDaoTyped;
use millegrilles_common_rust::tracing::{debug, error, info, warn};
use millegrilles_common_rust::v3::models::ErrorMessage;
use crate::constants::*;
use crate::external::mongo::*;
use crate::external::mq::{TRANSACTION_ACTION_NEW_CERTIFICATE, TRANSACTION_ACTION_SAVE_CERTIFICATE};
use crate::flow::transactions::PkiTransactionService;
use crate::models::TransactionCertificat;

/// Process the command part of the transaction (checks, validations, volatile updates),
/// calls transaction processor and then handles responses and emits events.
pub async fn process_transaction<M>(
    mongo: &M,
    messaging: &dyn MessagingService,
    pki: &dyn PkiService,
    outbound: &MessageOutboundFacade,
    transaction: &PkiTransactionService,
    wrapper: MessageValidated
) -> Result<(), CommonError> where M: MongoDaoTyped {
    let action = match wrapper.get_routing_action() {
        Some(action) => action,
        None => return outbound.respond(wrapper.delivery_info, ErrorMessage::err("No action provided in transaction")).await
    };
    match action {
        TRANSACTION_ACTION_SAVE_CERTIFICATE | TRANSACTION_ACTION_NEW_CERTIFICATE => {
            save_certificate(mongo, pki, outbound, transaction, wrapper).await
        },
        _ => {
            info!("Unknown action {} for process_transaction, skipping", action);
            Ok(())
        }
    }
}

async fn save_certificate<M>(
    mongo: &M,
    pki: &dyn PkiService,
    outbound: &MessageOutboundFacade,
    transaction: &PkiTransactionService,
    wrapper: MessageValidated,
) -> Result<(), CommonError> where M: MongoDaoTyped {
    let transaction_value: TransactionCertificat = wrapper.message.deserialize()?;

    let ca = match transaction_value.ca.as_ref() {
        Some(ca) => Some(ca.as_str()),
        None => None
    };

    // Validate the certificate (current date)
    let certificate = match pki.validate_pem(transaction_value.pem.as_str(), ca.clone(), None) {
        Ok(certificate) => certificate,
        Err(_e) => {
            // Assume the certificate is not currently valid. Get the not-before-date to confirm.
            let enveloppe = match EnveloppeCertificat::try_from(transaction_value.pem.as_str()) {
                Ok(enveloppe) => enveloppe,
                Err(e) => {
                    info!("Invalid certificate: {:?}", e);
                    return outbound.respond(wrapper.delivery_info, ErrorMessage::err_code(400, "Invalid certificate")).await
                }
            };
            let not_valid_before = enveloppe.not_valid_before()?;
            match pki.validate_pem(transaction_value.pem.as_str(), ca, Some(&not_valid_before)) {
                Ok(certificate) => certificate,
                Err(e) => {
                    info!("Invalid certificate: {:?}", e);
                    return outbound.respond(wrapper.delivery_info, ErrorMessage::err_code(400, "Invalid certificate")).await
                }
            }
        }
    };

    // Check if certificate already exists
    let fingerprint = match certificate.fingerprint() {
        Ok(fingerprint) => fingerprint,
        Err(e) => {
            info!("Error getting fingerprint of certificate: {:?}", e);
            return outbound.respond(wrapper.delivery_info, ErrorMessage::err_code(500, "Error getting fingerprint")).await
        }
    };

    let filter = doc!{ PKI_DOCUMENT_CHAMP_FINGERPRINT: &fingerprint };
    let collection = mongo.get_collection(COLLECTION_NAME_CERTIFICATES)?;
    if let Some(_row) = collection.find_one(filter).projection(doc!{"_id": true}).await? {
        // The certificate has already been received and processed successfully - respond with OK
        return outbound.respond(wrapper.delivery_info, ErrorMessage::ok()).await
    }

    // Run transaction updates
    let delivery_info = wrapper.delivery_info.clone();
    if let Err(e) = transaction.process_transaction(wrapper.into(), None).await {
        info!("Error saving certificate: {:?}", e);
        return outbound.respond(delivery_info, ErrorMessage::err_code(500, "Error saving certificate")).await
    }

    // Success
    outbound.respond(delivery_info, ErrorMessage::ok()).await
}
