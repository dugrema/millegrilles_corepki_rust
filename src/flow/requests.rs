use millegrilles_common_rust::bson::doc;
use millegrilles_common_rust::constantes::PKI_DOCUMENT_CHAMP_FINGERPRINT;
use crate::external::mq::{REQUEST_ACTION_CERTIFICATEBYPK, REQUEST_ACTION_INFOCERTIFICAT};
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::mongo_dao::MongoDaoTyped;
use millegrilles_common_rust::mongodb::options::Hint;
use millegrilles_common_rust::tracing::{debug, info};
use millegrilles_common_rust::serde::Deserialize;
use millegrilles_common_rust::v3::facades::message_inbound::MessageValidated;
use millegrilles_common_rust::v3::facades::message_outbound::MessageOutboundFacade;
use millegrilles_common_rust::v3::models::ErrorMessage;
use crate::external::mongo::{COLLECTION_NAME_CERTIFICATES, PKI_DOCUMENT_CHAMP_FINGERPRINT_PK};
use crate::models::{CertificateRow, ReponseEnveloppe, RequestCertificate};

pub async fn process_request<M>(
    mongo: &M,
    outbound: &MessageOutboundFacade,
    wrapper: MessageValidated,
) -> Result<(), CommonError> where
    M: MongoDaoTyped,
{
    let action = match wrapper.get_routing_action() {
        Some(action) => action,
        None => return outbound.respond(wrapper.delivery_info, ErrorMessage::err("No action provided in transaction")).await
    };
    match action {
        REQUEST_ACTION_INFOCERTIFICAT => request_certificate(mongo, outbound, wrapper).await,
        REQUEST_ACTION_CERTIFICATEBYPK => request_certificate_by_pk(mongo, outbound, wrapper).await,
        _ => {
            info!("Unknown action {} for process_requests, skipping", action);
            Ok(())
        }
    }
}

async fn request_certificate<M>(
    mongo: &M,
    outbound: &MessageOutboundFacade,
    wrapper: MessageValidated,
) -> Result<(), CommonError> where M: MongoDaoTyped {

    let request: RequestCertificate = wrapper.message.deserialize()?;
    let fingerprint = request.fingerprint;
    debug!("Request for cert {}", fingerprint);

    let filter = doc!{ PKI_DOCUMENT_CHAMP_FINGERPRINT: &fingerprint};
    let collection = mongo.get_collection_typed::<CertificateRow>(COLLECTION_NAME_CERTIFICATES)?;
    let certificate = collection
        .find_one(filter)
        .hint(Hint::Name(PKI_DOCUMENT_CHAMP_FINGERPRINT.to_string()))
        .await?;

    match certificate {
        Some(certificate) => {
            let response: ReponseEnveloppe = certificate.try_into()?;
            outbound.respond(wrapper.delivery_info, response).await
        },
        None => outbound.respond(wrapper.delivery_info, ErrorMessage::err_code(404, "Certificate not found")).await
    }
}

#[derive(Clone, Deserialize)]
struct RequestFingerprintByPublicKey {
    fingerprint_pk: String
}

async fn request_certificate_by_pk<M>(
    mongo: &M,
    outbound: &MessageOutboundFacade,
    wrapper: MessageValidated,
) -> Result<(), CommonError> where M: MongoDaoTyped {
    let request: RequestFingerprintByPublicKey = wrapper.message.deserialize()?;
    let fingerprint_pk = request.fingerprint_pk;

    let filter = doc!{ PKI_DOCUMENT_CHAMP_FINGERPRINT_PK: &fingerprint_pk};
    let collection = mongo.get_collection_typed::<CertificateRow>(COLLECTION_NAME_CERTIFICATES)?;
    let certificate = collection
        .find_one(filter)
        .hint(Hint::Name(PKI_DOCUMENT_CHAMP_FINGERPRINT_PK.to_string()))
        .await?;

    match certificate {
        Some(certificate) => {
            let response: ReponseEnveloppe = certificate.try_into()?;
            outbound.respond(wrapper.delivery_info, response).await
        },
        None => outbound.respond(wrapper.delivery_info, ErrorMessage::err_code(404, "Certificate not found")).await
    }
}
