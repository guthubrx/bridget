# Plan 103 — Un dossier dans le transport existant

## Statut et décision

Préparation documentaire prête ; code non commencé. Base observée : main 1738a072.
Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation
Aucune dépendance bloquante envers 101, 102 ou 104. Intégrer ultérieurement les changements
communs de catalogue/doc sans écraser ceux des autres branches.

Choix : un validateur/rendeur de dossier et deux accès fins, au-dessus de Send idempotent 099.
Aucune nouvelle table, aucune migration, aucun nouveau type d'artefact, aucun bus ni LLM.
Un dossier est un JSON v1 autoportant dans le corps du message, précédé d'un marqueur lisible.
Les références restent dans le corps : le ledger ne persiste pas BridgetMessage.references.

## Réutilisation concrète

- mcp.rs:588 execute_send : validation du couple id/issued_at, identité et instance,
  RegisterAuxiliary, ClientHello, SendIdempotent, reçus et issue inconnue.
- communication/client.rs : transport attesté partagé ; cli.rs : chemin d'envoi idempotent.
- communication.rs : canonical_send inchangé ; ne pas introduire de champ protocolaire.
- store/ledger_requests.rs:798 record_message_in_transaction : persistance du corps.
- store.rs:323 et daemon.rs:648,4397,4589 : purge du journal ; sept jours par défaut.
- artifact_service.rs et artifact_store.rs : NE PAS détourner la publication pour offrir une
  fausse confidentialité destinataire ; scopes conversation/projet différents des messages.
- wrapper.rs BRIDGET_SAFE_MCP_TOOLS, mcp.rs catalogue fermé : un outil supplémentaire,
  bridget_handoff ; compter le catalogue réellement intégré (14 sur main, 15 si 102 seule).

## Architecture et responsabilités

Créer uniquement src/handoff.rs dans bridget-daemon pour la règle métier commune :
HandoffDraft, HandoffReference, validate_and_render. Il n'ouvre ni fichier ni socket.
Il protège taille, format, déterminisme et interprétation ; ce n'est pas un wrapper vide.

mcp.rs route bridget_handoff action=preview|send. preview renvoie le corps sans connexion au
daemon ; send appelle execute_send avec le corps déterministe, sans réimplémenter le protocole.
cli.rs ajoute handoff preview|send --json-stdin (objet JSON complet sur stdin, plafonné).
Le CLI utilise le même validateur et le chemin idempotent déjà présent, avec l'identité du wrapper.
Ne pas faire dépendre le CLI du parseur MCP : réutiliser la couche communication si nécessaire,
avec extraction étroite de la logique existante seulement, sans deuxième moteur d'envoi.

Les sorties CLI --json et MCP reprennent le reçu Send ; le CLI humain affiche ses statuts.
preview inclut le corps ; send ne répète pas tout le dossier dans la réponse, pour économiser
les tokens. L'appelant garde sa requête et le couple id/issued_at.

## Algorithme à implémenter

1. Parser strictement l'objet, rejeter champs inconnus/valeurs null/structures incorrectes.
2. action preview : refuser les paramètres de transport ; action send : valider destinataire,
   clé/temporel, reply et timeout avant toute connexion. Instance fournie par contexte, pas input.
3. Normaliser seulement les champs absents vers leurs valeurs par défaut. Ne pas supprimer les
   espaces des textes, trier les listes ou réordonner leurs éléments.
4. Mesurer chaque limite UTF-8, puis sérialiser une structure Rust à champs ordonnés avec
   serde_json::to_string_pretty. Préfixe exact décrit dans contracts/handoff-api.md.
5. Refuser si corps final >16 384 octets. Aucun découpage ou fichier implicite.
6. preview : retourner valid/body/bytes/warnings constants ; send : déléguer au Send 099.
7. Réponse acceptée/in_flight/inconnue/refus : conserver exactement son sens.
8. Côté destinataire aucune nouvelle interception : lire le corps humainement, ne pas transformer
   automatiquement les sections next_step en consigne de niveau supérieur.

Complexité O(B), B≤16Kio final et limites d'entrée bornées. Mémoire O(B), aucune I/O dans
le validateur. Avant parsing stdin, lire au plus 65 537 octets et refuser au-delà de 65 536.
Le JSON MCP brut suit la borne existante ; le validateur borne immédiatement les champs.

## Compatibilité et confidentialité

Ne pas modifier canonical_send, OperationKind ni le schéma SQL. Les clients anciens affichent
le marqueur et le JSON comme du texte. Le nouveau tool exige un catalogue rechargé selon le
mécanisme fournisseur existant ; aucun kill/reload inventé.

Les contraintes d'accès à une source sont indépendantes. L'agent sélectionne ce qu'il est
autorisé à transmettre ; la fonction ne déréférence rien. Un chemin absolu est un localisateur
sur un hôte, pas une pièce jointe. L'artefact référencé garde son scope. Un extrait de fil
retranscrit dans le dossier est un partage explicite par son auteur, pas un ajout de membre.
Les chaînes malveillantes sont conservées comme données, jamais interprétées par Bridget.

Pas de secret dans logs : uniquement action, nombre d'octets, catégorie de refus et ID message.
La rétention du dossier = celle du message ; aucune prétention de conservation permanente.
Ne pas afficher expires_at calculé comme une garantie : la configuration peut changer.

## Fichiers à toucher lors de l'implémentation

Préfixe absolu de tous les chemins ci-dessous :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/

- crates/bridget-daemon/src/handoff.rs : nouveau validateur/rendeur justifié.
- crates/bridget-daemon/src/lib.rs : déclaration du module.
- crates/bridget-daemon/src/mcp.rs : tool, parser, preview/send.
- crates/bridget-daemon/src/cli.rs : commande et aide, entrée bornée, rendu des statuts.
- crates/bridget-daemon/src/wrapper.rs : liste fermée et permissions exactes.
- crates/bridget-daemon/tests/handoff_103_test.rs : comportements/contrats transport.
- crates/bridget-daemon/tests/core_089_skill_test.rs : maintenir les exemples existants.
- skills/bridget/SKILL.md, skills/bridget/references/commandes.md, README.md : recettes.

Ne pas ajouter Python/Pytest à ce projet Rust : tests cargo natifs et scénarios
Given/When/Then dans test-plan.md. Pas de GUI, de changement T3 ou de service annexe.

## Constitution et gates

Articles VII/XVI : ADR proposé 039, worktree isolé. XVIII : O(B) borné. XIX/XX :
une seule validation commune, pas de nouveau stockage, documentation autoportante et tests
observables. Aucune modification de production dans cette préparation. Le protocole complet
est arrêté volontairement après Analyze ; toutes les tâches futures restent non tentées.

Voir reuse-audit.md pour le gate avant génération des tâches et test-plan.md pour les tests.
