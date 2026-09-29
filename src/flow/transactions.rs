use std::sync::Arc;
use millegrilles_common_rust::mongo_dao::MongoDao;
use millegrilles_common_rust::v3::{ConfigService, FormatService, TransactionService};

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
    todo!()}
}
