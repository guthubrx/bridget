# Contrat - SandboxRuntimeV1

## Précondition

Le runtime est créé exclusivement à partir d'un reçu `kind: html` de SPEC-082.
Bridget a validé la version, la provenance, le projet et les blobs. Le cadre ne
reçoit aucun secret, chemin, cookie, capacité Tauri ou URL de collecte.

## Enveloppe obligatoire

~~~html
<iframe sandbox="allow-scripts" referrerpolicy="no-referrer"></iframe>
~~~

L'hôte applique une CSP fixe au document enveloppe : politique par défaut
refusée, `connect-src 'none'`, aucune frame, aucun formulaire, aucune base URL,
aucun média ou script externe. Les valeurs exactes seront testées dans la cible
WKWebView supportée. L'artefact ne peut ni compléter ni relâcher cette politique.

## Messages autorisés du cadre vers l'hôte

| Type | Charge utile | Validation hôte | Effet possible |
|---|---|---|---|
| `sandbox.ready` | `frame_instance_id` | correspond au ticket en cours | afficher le rendu |
| `sandbox.resize` | hauteur entière 0..1200 | borne et coalescence | hauteur inline seulement |
| `sandbox.state` | JSON sérialisable <= 128 KiB | schéma et taille | mettre à disposition pour une sauvegarde explicitement demandée |
| `sandbox.open_source` | `source_ref_id` | référence déjà présente dans le manifeste | proposer l'ouverture dans Browser après geste opérateur |
| `sandbox.error` | code fermé, texte <= 2 KiB | code connu et taille | carte d'erreur explicite |

Tout autre message est ignoré, journalisé comme refus de protocole et ne peut
pas déclencher d'IPC, d'accès fichier, de navigation ou de réseau.

## Messages autorisés de l'hôte vers le cadre

| Type | Charge utile | Moment |
|---|---|---|
| `sandbox.bootstrap` | données déclarées, métadonnées visuelles, limites | ouverture initiale |
| `sandbox.save_result` | version enfant ou erreur structurée | après action explicite Enregistrer |
| `sandbox.close` | aucune | destruction du cadre |

Le cadre ne reçoit pas l'URL source originelle si des octets canoniques sont
disponibles. Il reçoit une référence opaque de provenance uniquement.

## Échecs

- Blocage CSP ou sandbox : carte avec code, cause, accès au manifeste et action
  de restauration si applicable.
- Données non valides ou trop volumineuses : rendu refusé avant exécution, sans
  placeholder muet.
- Demande d'ouverture ou sauvegarde invalide : aucune modification d'historique
  et retour d'une erreur structurée.
