# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/plan.md](../../../../../../specs/102-fils-inter-agents/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

État : préparation documentaire, sans implémentation. Spécification : spec.md.
Racine de travail : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents
Base Git examinée : 1738a072 ; complément examiné en lecture seule :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3

<a id="extrait-2"></a>

## Extrait 2

Socle fonctionnel : 089 (identités/routage/journaux), 094 (parité), 099 (preuve
auxiliaire et idempotence de remise), 100 (lecture bornée et observations).
Compatibilité obligatoire : 097 (Claude interactif), 098 (T3), 101 (identité MCP
T3 et observations attestées). Les modifications non commitées de 101 ne sont PAS
présentes dans cette branche. T001 doit identifier leur commit intégré ou obtenir
un arbitrage d'intégration avant de modifier les composants communs. Ne pas
réimplémenter 101, copier son arbre, lancer un rebase sur une branche sale ou
confondre version installée et version Git. Cela n'empêche pas la préparation.

<a id="extrait-3"></a>

## Extrait 3

Pour T3 (101 t3code.rs:1659 et :2375), ne pas ajouter cette remise au `pending`
des DM qui relayent automatiquement la réponse finale. Produire une enveloppe
« Consulte ce fil via bridget_thread/read ; publie dans le fil si utile ; ne fais
pas de réponse directe à cette alerte ». Même exigence dans les injections Codex
gérées/interactives et Claude gérées/interactives. Une réponse finale ordinaire
reste dans la conversation fournisseur ; elle n'est pas extraite et publiée.

<a id="extrait-4"></a>

## Extrait 4

Négociation concrète, sans nouveau handshake : Register.thread_notice_versions
(Vec<u16>, default vide) ; Registered.thread_notice_version (Option<u16>, absent
par défaut). Adaptateurs implémentés annoncent [1] ; daemon choisit1 ouNone.
Sélection par connexion wrapper, effacée à déconnexion, revérifiée avant chaque
remise ou récupération. Adapter connect_and_register_at (wrapper.rs:1570) et son
appel T3 (101:1070). Ne pas utiliser ClientCapability (client auxiliaire) ni
ObservationCapabilities101 (sources de faits) pour cette capacité de réception.
