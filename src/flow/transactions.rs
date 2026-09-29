use crate::external::mongo::*;
use crate::external::mq::*;
use crate::models::{CertificateRow, TransactionCertificat};
use millegrilles_common_rust::async_trait::async_trait;
use millegrilles_common_rust::bson;
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::millegrilles_cryptographie::x509::EnveloppeCertificat;
use millegrilles_common_rust::mongo_dao::MongoDao;
use millegrilles_common_rust::mongodb::ClientSession;
use millegrilles_common_rust::openssl::x509::X509;
use millegrilles_common_rust::v3::impls::transaction_service::TransactionServiceImpl;
use millegrilles_common_rust::v3::models::{BatchInsertions, TransactionOperationAggregator, TransactionWrapper};
use millegrilles_common_rust::v3::{ConfigService, FormatService, TransactionRouter, TransactionService};
use std::sync::Arc;

pub struct PkiTransactionService {
    pub transaction: Arc<dyn TransactionService>,
}

impl PkiTransactionService {
    pub fn new(
        config: Arc<dyn ConfigService>,
        format: Arc<dyn FormatService>,
        mongo: Arc<dyn MongoDao>,
        _restoring: bool,
    ) -> Self {
        let router = PkiTransactionRouter {};
        let service = TransactionServiceImpl::new(
            config,
            format,
            mongo,
            COLLECTION_NAME_REDOLOG.to_string(),
            COLLECTION_NAME_TRACKING.to_string(),
            Box::new(router),
        );

        Self { transaction: Arc::new(service) }
    }

    pub async fn process_transaction(&self, wrapper: TransactionWrapper, session: Option<&mut ClientSession>) -> Result<(), CommonError> {
        self.transaction.process_transaction(wrapper, session).await
    }

    // pub async fn process_value(&self, domain: &str, action: &str, value: Value, session: Option<&mut ClientSession>) -> Result<(), CommonError> {
    //     self.transaction.process_value(domain, action, value, session).await
    // }
}

struct PkiTransactionRouter {
    // ignore_duplicates: bool,
}

#[async_trait]
impl TransactionRouter for PkiTransactionRouter {
    async fn route(
        &self,
        action: String,
        wrapper: TransactionWrapper
    ) -> Result<TransactionOperationAggregator, CommonError> {
        match action.as_str() {
            TRANSACTION_ACTION_SAVE_CERTIFICATE | TRANSACTION_ACTION_NEW_CERTIFICATE => {
                save_certificate(wrapper).await
            },
            _ => Err(CommonError::Str("Unknown transaction action"))
        }
    }
}

async fn save_certificate(
    wrapper: TransactionWrapper,
) -> Result<TransactionOperationAggregator, CommonError> {

    let certificate: TransactionCertificat = wrapper.message.deserialize()?;

    let mut enveloppe = EnveloppeCertificat::try_from(certificate.pem.as_str())?;
    if let Some(ca) = certificate.ca {
        enveloppe.millegrille = Some(X509::from_pem(ca.as_bytes())?);
    }
    let row: CertificateRow = enveloppe.try_into()?;
    let doc_row = bson::serialize_to_document(&row)?;

    let mut aggregator = TransactionOperationAggregator::new();
    aggregator.batch_insertion(BatchInsertions::new(COLLECTION_NAME_CERTIFICATES, vec![doc_row]))?;

    Ok(aggregator)
}
