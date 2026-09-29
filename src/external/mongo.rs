use millegrilles_common_rust::configuration::ConfigMessages;
use millegrilles_common_rust::constantes::*;
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::mongo_dao::{ChampIndex, IndexOptions, MongoDao};

pub const COLLECTION_NAME_REDOLOG: &str = "CorePki/redolog";
pub const COLLECTION_NAME_TRACKING: &str = "CorePki/tracking";
pub const COLLECTION_NAME_CERTIFICATES: &str = "CorePki/certificates";
pub const INDEX_REDO_LOG_ID: &str = "redo_log_id";
pub const PKI_DOCUMENT_CHAMP_FINGERPRINT_PK: &str = "fingerprint_pk";

pub async fn create_index_mongodb(db: &dyn MongoDao, config: &dyn ConfigMessages) -> Result<(), CommonError> {
    db.create_index(
        config,
        COLLECTION_NAME_REDOLOG,
        vec!(
            ChampIndex { nom_champ: String::from(TRANSACTION_CHAMP_ID), direction: 1 },
        ),
        Some(IndexOptions {
            nom_index: Some(String::from(INDEX_REDO_LOG_ID)),
            unique: true,
        }),
    ).await?;

    db.create_index(
        config,
        COLLECTION_NAME_REDOLOG,
        vec!(
            ChampIndex { nom_champ: String::from(FIELD_PROCESSED), direction: 1 },
        ),
        Some(IndexOptions {
            nom_index: Some(String::from(INDEX_DATE_PROCESSED)),
            unique: false,
        }),
    ).await?;

    db.create_index(
        config,
        COLLECTION_NAME_TRACKING,
        vec!(
            ChampIndex { nom_champ: String::from(FIELD_BID), direction: 1 },
        ),
        Some(IndexOptions {
            nom_index: Some(String::from(INDEX_BID)),
            unique: true,
        }),
    ).await?;

    db.create_index(
        config,
        COLLECTION_NAME_TRACKING,
        vec!(
            ChampIndex { nom_champ: String::from(FIELD_DATE_PROCESSED), direction: 1 },
        ),
        Some(IndexOptions {
            nom_index: Some(String::from(INDEX_DATE_PROCESSED)),
            unique: false,
        })
    ).await?;

    // TODO Index

    db.create_index(
        config,
        COLLECTION_NAME_CERTIFICATES,
        vec!(
            ChampIndex { nom_champ: String::from(PKI_DOCUMENT_CHAMP_FINGERPRINT), direction: 1 },
        ),
        Some(IndexOptions {
            nom_index: Some(String::from(PKI_DOCUMENT_CHAMP_FINGERPRINT)),
            unique: true,
        })
    ).await?;

    db.create_index(
        config,
        COLLECTION_NAME_CERTIFICATES,
        vec!(
            ChampIndex { nom_champ: String::from(PKI_DOCUMENT_CHAMP_FINGERPRINT_PK), direction: 1 },
        ),
        Some(IndexOptions {
            nom_index: Some(String::from(PKI_DOCUMENT_CHAMP_FINGERPRINT_PK)),
            unique: true,
        })
    ).await?;

    Ok(())
}
