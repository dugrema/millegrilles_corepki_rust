use std::sync::Arc;
use millegrilles_common_rust::async_trait::async_trait;
use millegrilles_common_rust::mongo_dao::MongoDao;
use millegrilles_common_rust::v3::{ConfigService, FormatService, TransactionRouter, TransactionService};
use millegrilles_common_rust::v3::models::{TransactionOperationAggregator, TransactionWrapper};
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::mongodb::ClientSession;
use millegrilles_common_rust::serde_json::Value;
use millegrilles_common_rust::v3::impls::transaction_service::TransactionServiceImpl;
use crate::external::mongo::*;

pub struct PkiTransactionService {
    pub transaction: Arc<dyn TransactionService>,
}

impl PkiTransactionService {
    pub fn new(
        config: Arc<dyn ConfigService>,
        format: Arc<dyn FormatService>,
        mongo: Arc<dyn MongoDao>,
        restoring: bool,
    ) -> Self {
        let router = PkiTransactionRouter { mongo: mongo.clone(), ignore_duplicates: restoring };
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

    pub async fn process_value(&self, domain: &str, action: &str, value: Value, session: Option<&mut ClientSession>) -> Result<(), CommonError> {
        self.transaction.process_value(domain, action, value, session).await
    }
}

struct PkiTransactionRouter {
    mongo: Arc<dyn MongoDao>,
    ignore_duplicates: bool,
}

#[async_trait]
impl TransactionRouter for PkiTransactionRouter {
    async fn route(
        &self,
        action: String,
        wrapper: TransactionWrapper
    ) -> Result<TransactionOperationAggregator, CommonError> {
        match action.as_str() {
            // TODO TRANSACTION_SAUVEGARDER_CATEGORIE_USAGER => save_user_category(self.mongo.as_ref(), wrapper).await,

            _ => Err(CommonError::Str("Unknown transaction action"))
        }
    }
}
