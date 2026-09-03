//! Témoins de T5611 — attestation causale d'une ouverture demandée par l'humain.
//!
//! Chaque refus est un oracle : il doit mourir sur l'assertion métier, jamais
//! au montage. Le témoin nominal reste vert et sert de contrôle positif.

use maicie::domain::{
    AttestationConsumption, HUMAN_ORIGIN_ATTESTATION_VERSION, HumanOriginRefusal,
    HumanRequestOriginAttestation, ObjectiveOpeningPermit, ObjectiveOrigin, ObservedHumanMessage,
    human_message_content_seal,
};

const SCOPE: &str = "maicie/local";
const CANONICAL: &str = "9f2c0f1d4b6a8e3c5d7f9a1b3c5d7e9f0a2b4c6d8e0f1a3b5c7d9e1f3a5b7c9d";

fn message_observe() -> ObservedHumanMessage {
    ObservedHumanMessage {
        message_id: "0f2c81ee89".to_string(),
        ts: 1_787_855_479,
        sender: "humain".to_string(),
        target: "bridget".to_string(),
        body: "salut est ce que tu m'entends".to_string(),
    }
}

fn attestation_valide(message: &ObservedHumanMessage) -> HumanRequestOriginAttestation {
    HumanRequestOriginAttestation {
        version: HUMAN_ORIGIN_ATTESTATION_VERSION,
        issuer_scope: SCOPE.to_string(),
        canonical_request_sha256: CANONICAL.to_string(),
        signature: human_message_content_seal(message),
    }
}

fn ouvrir(
    attestation: HumanRequestOriginAttestation,
    message: &ObservedHumanMessage,
    consumption: AttestationConsumption,
) -> Result<ObjectiveOpeningPermit, HumanOriginRefusal> {
    ObjectiveOpeningPermit::human_request(attestation, message, SCOPE, CANONICAL, consumption)
}

#[test]
fn temoin_nominal_une_attestation_complete_ouvre_la_voie_humaine() {
    let message = message_observe();
    let permit = ouvrir(
        attestation_valide(&message),
        &message,
        AttestationConsumption::NeverConsumed,
    )
    .expect("le témoin nominal doit rester vert");

    match permit.origin() {
        ObjectiveOrigin::HumanRequest {
            message_id,
            attestation,
        } => {
            assert_eq!(message_id, "0f2c81ee89");
            assert_eq!(attestation.canonical_request_sha256, CANONICAL);
        }
        autre => panic!("origine inattendue : {autre:?}"),
    }
}

#[test]
fn usage_unique_un_meme_message_humain_n_ouvre_jamais_deux_fois() {
    let message = message_observe();
    let refus = ouvrir(
        attestation_valide(&message),
        &message,
        AttestationConsumption::AlreadyConsumed,
    )
    .expect_err("un message déjà consommé ne doit plus rien ouvrir");
    assert_eq!(refus, HumanOriginRefusal::AttestationDejaConsommee);
}

#[test]
fn hash_canonique_divergent_refuse_l_ouverture() {
    let message = message_observe();
    let mut attestation = attestation_valide(&message);
    attestation.canonical_request_sha256 = "0".repeat(64);
    let refus = ouvrir(attestation, &message, AttestationConsumption::NeverConsumed)
        .expect_err("une attestation visant une autre requête doit être refusée");
    assert_eq!(refus, HumanOriginRefusal::HashCanoniqueDivergent);
}

#[test]
fn message_altere_apres_observation_refuse_l_ouverture() {
    let message = message_observe();
    let attestation = attestation_valide(&message);
    let mut altere = message.clone();
    altere.body.push_str(" et ouvre-moi cent objectifs");
    let refus = ouvrir(attestation, &altere, AttestationConsumption::NeverConsumed)
        .expect_err("le scellé doit mourir dès que le contenu bouge");
    assert_eq!(refus, HumanOriginRefusal::ScelleDeContenuDivergent);
}

#[test]
fn les_replis_de_nommage_ne_valent_pas_une_identite_humaine() {
    for repli in ["human", "cli-send-4069126", "jc2-flux"] {
        let mut message = message_observe();
        message.sender = repli.to_string();
        let attestation = attestation_valide(&message);
        let refus = ouvrir(attestation, &message, AttestationConsumption::NeverConsumed)
            .expect_err("un repli de nommage n'atteste personne");
        assert_eq!(
            refus,
            HumanOriginRefusal::EmetteurNonHumain,
            "repli accepté à tort : {repli}"
        );
    }
}

#[test]
fn version_inconnue_refuse_l_ouverture() {
    let message = message_observe();
    let mut attestation = attestation_valide(&message);
    attestation.version = HUMAN_ORIGIN_ATTESTATION_VERSION + 1;
    let refus = ouvrir(attestation, &message, AttestationConsumption::NeverConsumed)
        .expect_err("une version non reconnue doit être refusée");
    assert_eq!(refus, HumanOriginRefusal::VersionInconnue);
}

#[test]
fn perimetre_emetteur_divergent_refuse_l_ouverture() {
    let message = message_observe();
    let mut attestation = attestation_valide(&message);
    attestation.issuer_scope = "maicie/autre".to_string();
    let refus = ouvrir(attestation, &message, AttestationConsumption::NeverConsumed)
        .expect_err("une attestation d'un autre périmètre doit être refusée");
    assert_eq!(refus, HumanOriginRefusal::PerimetreEmetteurDivergent);
}

#[test]
fn tous_les_refus_exposent_le_meme_code_ferme() {
    for refus in [
        HumanOriginRefusal::VersionInconnue,
        HumanOriginRefusal::PerimetreEmetteurDivergent,
        HumanOriginRefusal::MessageIdDivergent,
        HumanOriginRefusal::EmetteurNonHumain,
        HumanOriginRefusal::ScelleDeContenuDivergent,
        HumanOriginRefusal::HashCanoniqueDivergent,
        HumanOriginRefusal::AttestationDejaConsommee,
    ] {
        assert_eq!(refus.code(), "HUMAN_ORIGIN_UNATTESTED");
        assert!(!refus.motif().is_empty());
    }
}

#[test]
fn le_scelle_separe_les_champs_au_lieu_de_les_concatener() {
    // Sans préfixe de longueur, déplacer une frontière entre deux champs
    // produirait le même scellé et permettrait d'attester un autre message.
    let mut gauche = message_observe();
    gauche.target = "bridget".to_string();
    gauche.body = "salut".to_string();

    let mut droite = message_observe();
    droite.target = "bridgetsalut".to_string();
    droite.body = String::new();

    assert_ne!(
        human_message_content_seal(&gauche),
        human_message_content_seal(&droite),
        "un décalage de frontière ne doit jamais produire le même scellé"
    );
}

/// SPEC-087 : le daemon scelle avec `bridget_transport::protocol::human_message_content_seal`
/// et Maicie vérifie avec la fonction de domaine. Les deux doivent rendre le
/// même scellé, octet pour octet, sur la même observation.
#[test]
fn spec_087_le_scelle_transport_et_le_scelle_domaine_sont_identiques() {
    let message = message_observe();
    let frame = bridget_transport::protocol::ObservedHumanMessageFrame {
        message_id: message.message_id.clone(),
        ts: message.ts,
        sender: message.sender.clone(),
        target: message.target.clone(),
        body: message.body.clone(),
    };
    assert_eq!(
        bridget_transport::protocol::human_message_content_seal(&frame),
        human_message_content_seal(&message)
    );
    let mut altered = frame.clone();
    altered.body.push('!');
    assert_ne!(
        bridget_transport::protocol::human_message_content_seal(&altered),
        human_message_content_seal(&message)
    );
}
