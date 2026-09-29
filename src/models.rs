use millegrilles_common_rust::serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionCertificat {
    pub pem: String,
    pub ca: Option<String>
}
