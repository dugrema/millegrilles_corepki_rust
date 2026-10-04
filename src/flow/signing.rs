use crate::models::ReponseCertificatSigne;
use millegrilles_common_rust::certificats::VerificateurPermissions;
use millegrilles_common_rust::common_messages::DemandeSignature;
use millegrilles_common_rust::constantes::*;
use millegrilles_common_rust::error::Error as CommonError;
use millegrilles_common_rust::millegrilles_cryptographie::messages_structs::MessageMilleGrillesBufferDefault;
use millegrilles_common_rust::millegrilles_cryptographie::x509::EnveloppeCertificat;
use millegrilles_common_rust::reqwest;
use millegrilles_common_rust::tracing::{debug, error, info, warn};
use millegrilles_common_rust::v3::ConfigService;
use millegrilles_common_rust::v3::facades::message_inbound::MessageValidated;

pub fn validate_csr_signature_request(
    request: &DemandeSignature,
    certificat: &EnveloppeCertificat,
) -> Result<(), CommonError> {
    if certificat.verifier_roles(vec![RolesCertificats::Instance])? {
        if certificat.verifier_exchanges(vec![Securite::L3Protege, Securite::L4Secure])? {
            debug!("valider_demande_signature_csr Demande de CSR signee par une instance 3.protege ou 4.secure, demande approuvee");
            // message = Some(Cow::Borrowed(&m.message.parsed));
            return Ok(())
        } else if certificat.verifier_exchanges(vec![Securite::L2Prive])? {
            debug!("valider_demande_signature_csr Demande de CSR signee par une instance 2.prive");
            // message = Some(Cow::Borrowed(&m.message.parsed));
            return Ok(())
        } else if certificat.verifier_exchanges(vec![Securite::L1Public])? {
            debug!("valider_demande_signature_csr Demande de CSR signee par une instance 1.public, demande approuvee");
            // message = Some(Cow::Borrowed(&m.message.parsed));
            return Ok(())
        } else {
            error!("valider_demande_signature_csr Demande de CSR signee par une instance sans exchanges, REFUSE");
        }
    } else if certificat.verifier_roles(vec![RolesCertificats::MaitreDesClesConnexion])? {
        if certificat.verifier_exchanges(vec![Securite::L4Secure])? {
            // Un certificat de maitre des cles connexion (4.secure) supporte une cle volatile

            // Validation des valeurs
            if request.domaines.is_some() || request.dns.is_some() {
                // warn!("valider_demande_signature_csr Signature certificat maitre des cles volatil refuse (exchanges/domaines/dns presents)");
                Err("valider_demande_signature_csr Signature certificat maitre des cles volatil refuse (exchanges/domaines/dns presents)")?
            }

            // Verifier que le role demande est MaitreDesClesConnexionVolatil
            match request.roles.as_ref() {
                Some(r) => {
                    let role_maitre_des_cles_string = ROLE_MAITRE_DES_CLES.to_string();
                    let role_maitre_des_cles_volatil_string = ROLE_MAITRE_DES_CLES_VOLATIL.to_string();
                    if r.len() == 2 && r.contains(&role_maitre_des_cles_string) && r.contains(&role_maitre_des_cles_volatil_string) {
                        // message = Some(Cow::Borrowed(&m.message.parsed))
                        return Ok(())
                    } else {
                        warn!("valider_demande_signature_csr Signature certificat maitre des cles volatil refuse (mauvais role)");
                        // let e = json!({"ok": false, "err": "Mauvais role"});
                        // return Ok(Some(Cow::Owned(middleware.formatter_reponse(e, None)?)));
                        Err("Mauvais role")?
                    }
                },
                None => {
                    // let e = json!({"ok": false, "err": "Aucun role"});
                    // return Ok(Some(Cow::Owned(middleware.formatter_reponse(e, None)?)));
                    Err("Aucun role")?
                }
            }
        } else {
            error!("valider_demande_signature_csr Demande de CSR signee par une un certificat MaitreDesClesConnexion de niveau != 4.secure, REFUSE");
            // let e = json!({"ok": false, "err": "niveau != 4.secure"});
            // return Ok(Some(Cow::Owned(middleware.formatter_reponse(e, None)?)));
            Err("niveau != 4.secure")?
        }
    } else if certificat.verifier_delegation_globale(DELEGATION_GLOBALE_PROPRIETAIRE)? {
        debug!("valider_demande_signature_csr Demande de CSR signee par une delegation globale (proprietaire), demande approuvee");
        // message = Some(Cow::Borrowed(&m.message.parsed));
        return Ok(())
    } else if certificat.verifier_exchanges(vec![Securite::L4Secure])? {
        debug!("valider_demande_signature_csr Demande de CSR signee pour un domaine");
        if request.domaines.is_some() || request.exchanges.is_some() {
            Err(String::from("domaines/exchanges/roles doivent etre vide"))?;
        }

        let extensions = certificat.extensions()?;
        let domaines = match &extensions.domaines {
            Some(d) => d,
            None => {
                Err(format!("valider_demande_signature_csr Demande de signature de CSR refusee, demandeur sans domaines : {:?}", certificat))?
            }
        };

        let roles = match &request.roles {
            Some(r) => r,
            None => {
                Err(format!("valider_demande_signature_csr Demande de signature de CSR refusee, demande sans roles : {:?}", certificat))?
            }
        };

        // S'assurer que tous les roles demandes sont l'equivalent de domaines en minuscules.
        let domaines_minuscules: Vec<String> = domaines.iter().map(|d| d.to_lowercase()).collect();
        for role in roles {
            debug!("Role {} in roles : {:?}?", role, domaines_minuscules);
            if ! domaines_minuscules.contains(role) {
                Err(format!("valider_demande_signature_csr Demande de signature de CSR refusee, role {} non autorise pour domaines : {:?}", role, certificat))?
            }
        }

        return Ok(())
    } else {
        Err(format!("valider_demande_signature_csr Demande de signature de CSR refusee pour demandeur qui n'est pas autorise : {:?}", certificat))?;
    }

    Err(String::from("Acces refuse (default)"))?
}

pub async fn sign_with_certissuer(
    config: &dyn ConfigService,
    wrapper: &MessageValidated
) -> Result<ReponseCertificatSigne, CommonError> {

    let certissuer_url = match config.get_configuration_instance().certissuer_url.as_ref() {
        Some(url) => url,
        None => return Err(CommonError::Str("Certissuer URL is not defined"))
    };

    let client = reqwest::Client::builder()
        .timeout(core::time::Duration::new(7, 0))
        .build()?;

    let mut url_post = certissuer_url.clone();
    url_post.set_path("signerModule");

    info!("Resquesting CSR signature from {}", url_post);
    let buffer: MessageMilleGrillesBufferDefault = (&wrapper.message).try_into()?;
    let response = client.post(url_post).body(buffer.buffer).send().await?;

    // Raise error if required
    response.error_for_status_ref()?;

    let response: ReponseCertificatSigne = response.json().await?;
    debug!("commande_signer_csr Reponse certificat : {:?}", response);
    Ok(response)
}
