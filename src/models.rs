use std::collections::HashMap;
use millegrilles_common_rust::certificats::VerificateurPermissions;
use millegrilles_common_rust::chrono::{DateTime, Utc};
use millegrilles_common_rust::millegrilles_cryptographie::x509::EnveloppeCertificat;
use millegrilles_common_rust::serde::{Serialize, Deserialize};
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::bson::serde_helpers::datetime::FromChrono04DateTime;
use millegrilles_common_rust::chrono::serde::ts_milliseconds;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionCertificat {
    pub pem: String,
    pub ca: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandSaveCertificate {
    pub chaine_pem: Vec<String>,
    pub ca: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateRow {
    pub fingerprint: String,
    pub fingerprint_pk: String,
    pub certificat: Vec<String>,
    pub chaine: Vec<String>,
    #[serde(with="FromChrono04DateTime")]
    pub not_valid_after: DateTime<Utc>,
    #[serde(with="FromChrono04DateTime")]
    pub not_valid_before: DateTime<Utc>,
    pub sujet: HashMap<String, String>,
    #[serde(with="FromChrono04DateTime")]
    pub creation: DateTime<Utc>,
    pub est_ca: bool,
    pub idmg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchanges: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domaines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation_globale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation_domaines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca: Option<String>,
}

impl TryFrom<EnveloppeCertificat> for CertificateRow {
    type Error = CommonError;

    fn try_from(value: EnveloppeCertificat) -> Result<Self, Self::Error> {
        let fingerprint = value.fingerprint()?;
        let fingerprint_pk = value.fingerprint_pk()?;

        // Get the calculated IDMG value and the CA flag.
        let (idmg, is_ca) = match value.est_ca()? {
            true => (value.calculer_idmg()?, true),
            false => (value.idmg()?, false)
        };

        // Align certs and their fingerprints
        let fp_certs = value.chaine_fingerprint_pem()?;
        let mut certs = Vec::new();
        let mut chaine = Vec::new();
        for fp_cert in fp_certs {
            chaine.push(fp_cert.fingerprint);
            certs.push(fp_cert.pem);
        }

        let mut subject = HashMap::new();
        for (key, value) in value.subject()? {
            subject.insert(key, value);
        }

        let extensions = value.get_extensions()?;
        let (
            exchanges,
            domaines,
            roles,
            user_id,
            delegation_globale,
            delegation_domaines
        ) = match extensions {
            Some(extensions) => {
                let exchanges =extensions.exchanges.clone();
                let domaines = extensions.domaines.clone();
                let roles = extensions.roles.clone();
                let user_id = extensions.user_id.clone();
                let delegation_globale = extensions.delegation_globale.clone();
                let delegation_domaines = extensions.delegation_domaines.clone();
                (exchanges, domaines, roles, user_id, delegation_globale, delegation_domaines)
            },
            None => (None, None, None, None, None, None)
        };

        Ok(Self {
            fingerprint,
            fingerprint_pk,
            certificat: certs,
            chaine,
            not_valid_after: value.not_valid_after()?,
            not_valid_before: value.not_valid_before()?,
            sujet: subject,
            creation: Utc::now(),
            est_ca: is_ca,
            idmg,
            exchanges,
            domaines,
            roles,
            user_id,
            delegation_globale,
            delegation_domaines,
            ca: value.ca_pem()?,
        })
    }
}

#[derive(Clone, Deserialize)]
pub struct RequestCertificate {
    pub fingerprint: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReponseEnveloppe {
    pub chaine_pem: Vec<String>,
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_pem: Option<String>,
}

impl TryFrom<&EnveloppeCertificat> for ReponseEnveloppe {
    type Error = CommonError;
    fn try_from(value: &EnveloppeCertificat) -> Result<Self, Self::Error> {
        Ok(Self {
            chaine_pem: value.chaine_pem()?,
            fingerprint: value.fingerprint()?,
            ca_pem: value.ca_pem()?,
        })
    }
}

impl TryFrom<CertificateRow> for ReponseEnveloppe {
    type Error = CommonError;
    fn try_from(value: CertificateRow) -> Result<Self, Self::Error> {
        Ok(Self {
            chaine_pem: value.certificat,
            fingerprint: value.fingerprint,
            ca_pem: value.ca,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReponseCertificatSigne {
    pub ok: bool,
    pub certificat: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandSignUserCsr {
    pub nom_usager: String,
    pub user_id: String,
    pub csr: String,
    pub delegation_globale: Option<String>,
    pub compte_prive: Option<bool>,
    pub delegations_version: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestTopologyConfiguration { pub filename: String }

/// Decrypted content of the configuration property value field.
#[derive(Clone, Debug, Deserialize)]
pub struct ConfigurationValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inumber: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fnumber: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigurationProperty {
    pub file_id: String,
    /// Name of this property
    pub key: String,
    /// Values
    pub value: ConfigurationValue,
    #[serde(with="ts_milliseconds")]
    pub last_modified: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResponseConfigurationGetProperties {
    pub file_id: String,
    pub filename: String,
    /// Roles, sent for admin requests (when key requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// Domains, sent for admin requests (when key requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    /// Decryption key id (keymaster ref)
    pub key_id: String,
    /// Last modification of either file or any properties in the list
    #[serde(with="ts_milliseconds")]
    pub last_modified: DateTime<Utc>,
    /// Decrypted properties
    pub list: Vec<ConfigurationProperty>,
    /// Contains the unencrypted secret key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    /// False if all properties have not been returned
    pub done: bool,
}

impl ResponseConfigurationGetProperties {
    pub fn map_values(&self) -> HashMap<&str, &ConfigurationProperty> {
        let mut map = HashMap::new();
        for property in self.list.iter() {
            map.insert(property.key.as_str(), property);
        }
        map
    }
}

#[derive(Deserialize)]
pub struct ConfigurationFileEvent {
    pub file_id: String,
    pub filename: String,
    pub event: Option<String>,
}
