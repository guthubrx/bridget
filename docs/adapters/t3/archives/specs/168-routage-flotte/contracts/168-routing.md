# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/contracts/168-routing.md](../../../../../../../specs/168-routage-flotte/contracts/168-routing.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Commande exacte de lecture : bridget delegate --catalogue --json [--machine ID].
Sans --json, afficher une table lisible avec les mêmes faits. Aucun argumenttask ni lancement.
Opération native décidée : routing_catalogue_v1, sur le même canal, exclusivement en lecture.
Exiger Role::Client, version négociée et UID du pairUnix égal à celui du daemon : unix_peer_uid
(Linux/Mac), peer_uids de handle_connection et pattern de WakeAction::CliOnce.
Cette garde accepte l’humain local sans session d’agent ; elle n’utilise jamais human_authority_guard,
qui dépend de T3. Aucun --from arbitraire, endpoint réseau nouveau ou autorité de mutation.
La projection est publique et allowlistée : version=1, observed_at, options et inconnus.
Un registre dépassant128est refusé, jamais un succès partiel complete:false.
Chaque option montre engine, model réel, machine_id, connection_id, supported_efforts/default_effort,
provenance/âge et entitlement=unknown pour cette simple lecture. Ne pas exiger un appel à chaque
modèle pour qualifier le catalogue ; un refus fournisseur final est rendu explicitement sans bascule.
Un profil nommé pro n’est jamais le nom public du modèle.
La lecture ne lance pas un fournisseur pour rafraîchir son catalogue : configuration et observations déjà obtenues seulement.
Le catalogue168 est versionné et rend128options au maximum, connexions et faits sans secret.
Une ancienne lecture163 conserve son schéma ; elle n’est pas présentée comme inventaire complet168.

<a id="extrait-2"></a>

## Extrait 2

Premier adaptateur : ClaudeStreamJsonTransport existant avec profil interne de décision.
Exiger le support réel des flags sur chaque binaire installé : --safe-mode, --restricted, --tools "",
--strict-mcp-config avec configuration vide, --setting-sources "", --disable-slash-commands,
--no-session-persistence et --json-schema en mode print/stream-json réel.
--safe-mode est attesté par les aides locales Mac2.1.300 et Cartae2.1.251 : il retire
CLAUDE.md/skills/plugins/hooks/MCP/personnalisations mais conserve auth et politiques administrées.
--bare est interdit pour ce rôle OAuth : l’aide des deux versions limite son auth à APIkey/helper.
--disallowedTools "*" peut renforcer les gardes si son support est attesté, sans être leur unique protection.
Pas de --dangerously-skip-permissions, pas de nouvelle cléAPI requise.
Les politiques administrées restent applicables ; si l’isolement requis ne peut être attesté, niveau indisponible.
Aucune injection MCPBridge/T3, aucun add-dir, hooks/skills/plugins/projet/autoreprise.
Environnement enfant env_clear puis liste minimale issue de la connexion autorisée,
cwd neutre privé et pas de session_store_root/resume de l’exécutant.
Les écritures natives auth/state du moteur restent possibles ; aucune écriture métier ni outil.
Un prompt « ne pas utiliser les outils » seul n’est pas une protection suffisante.
Exception de formatage strictement bornée : le seul outil natif StructuredOutput imposé par
json-schema est autorisé, sans droit métier/système/réseau/MCP ni droit d’exécution.
Le mode rejette toute autre capacité annoncée, tout autre outil ou tool_use. Un tool_use
StructuredOutput doit porter uniquement l’objet fermé prévu, validé localement parmi les ID/efforts
autorisés ; sa sortie structured_output et son resultJSON doivent concorder. Aucune instruction
ni clé supplémentaire ne gagne d’autorité par ce formateur. La qualification CLI observée
a init.tools=[StructuredOutput], pas init.tools=[] ; le transport168 reste à qualifier.
