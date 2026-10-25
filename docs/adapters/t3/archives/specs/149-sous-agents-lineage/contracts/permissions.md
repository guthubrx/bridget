# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/contracts/permissions.md](../../../../../../../specs/149-sous-agents-lineage/contracts/permissions.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Statut : proposé pour gate GLM. Date :2026-10-10.
Les permissions appartiennent au moteur natif. T3 fournit une attestation de
son fournisseur. Les wrappers standalone fournissent leur observation propriétaire.

<a id="extrait-2"></a>

## Extrait 2

Le snapshot est figé à admission, avant effet. Il conserve la politique d'origine,
le mappage appliqué et la définition enfant. Lancement, retry et reprise utilisent
ce même snapshot. Un changement ultérieur UI/registre n'élargit aucune mission.
Une preuve d'identité fraîche autorise l'accès; elle ne remplace pas le snapshot.
Le moteur admis continue si T3 devient indisponible. L'annulation humaine reste native.

<a id="extrait-3"></a>

## Extrait 3

Source T3 : paramètres finaux validés de turn/start après defaults et overrides.
Les flags launchArgs de sécurité non représentés rendent l'attestation indisponible.

<a id="extrait-4"></a>

## Extrait 4

Chaque liste est bornée à128entrées. Une règle outil est non vide, sans contrôle
interdit et au plus512octets. Additional_directories contient des chemins absolus.
Les valeurs facultatives ne deviennent pas des listes vides inventées.
`manual` observé en CLI est normalisé en default, alias documenté par le fournisseur.
T3 ne passe actuellement aucun sandbox_settings dans queryOptions. Il n'en publie
donc aucun. Un futur champ exige une version ou extension contractuelle validée.

<a id="extrait-5"></a>

## Extrait 5

Le callback T3 existe réellement. Tool_approval reflète sa décision de politique,
pas le seul mode UI. Plan_exit=deny préserve la politique actuelle. Bridget ne
transporte pas la fonction callback T3 dans un enfant standalone.

<a id="extrait-6"></a>

## Extrait 6

Source T3 : queryOptions finales et mode mutable observé après init/status ou
setPermissionMode réussi. `settings_sources=provider_default` décrit honnêtement
les réglages chargés implicitement par le fournisseur. Il ne prétend pas exposer
les règles fusionnées de ces fichiers. Un mappage qui exige leur connaissance
doit obtenir un snapshot de permission complet de la source propriétaire ou
utiliser la voie de mêmes inputs décrite ci-dessous. Un mappage inter-driver
sans preuve reste permission_mapping_unavailable. Aucun JSON settings/env brut
n'est publié.

<a id="extrait-7"></a>

## Extrait 7

La résolution appartient au launcher natif ou à l'adaptateur T3 propriétaire.
Elle utilise l'environnement final de lancement, dont son PATH effectif, sans
publier cet environnement. Elle est fraîche, sans cache de résolution. Le cas
gclaude connu termine par exec claude "$@", sans changer PATH/cwd ni charger
un autre script. L'owner résout donc claude sous ce PATH précis, puis realpath
et SHA256. Un nom de wrapper ne constitue jamais cette preuve. Une chaîne non
observable, ou BASH_ENV/ENV/fonction shell exportée non attestée qui peut changer
la résolution, rend permission_source_unavailable pour cette voie seulement.

<a id="extrait-8"></a>

## Extrait 8

Cas réel identifié : l'instance T3 claude_glm et la cible native glm utilisent
`/Users/moi/.local/bin/gclaude` et `/Users/moi/.claude-glm`. Le launcher transmet
les arguments à Claude et n'ajoute aucun override permission/settings/tools.
La recette isolée GLM doit utiliser ce même couple launcher/profil attesté.
La cible conserve son branchement d'auth propre. Aucun credential parent n'est
transmis. Le modèle exact et la politique finale de mission restent ceux figés
par Bridget, avec aucune substitution automatique.

<a id="extrait-9"></a>

## Extrait 9

Le registre privé T3 lie le fait à l'objet credential courant, environnement,
fil, session/instance fournisseur et run_id. L'adaptateur publie le fait juste
avant l'envoi réel au fournisseur. Un échec/end/close retire seulement le fait
du run propriétaire; un cleanup ancien ne retire pas le nouveau tour.
Rotation ou retrait de session MCP efface le fait. Revision augmente lorsque
le fait courant change. Elle ne prouve ni durée de validité ni TTL arbitraire.

<a id="extrait-10"></a>

## Extrait 10

L'adaptateur Bridget privé transporte seulement BRIDGET_T3_MCP_ENDPOINT et la
preuve opaque d'autorisation148. Le daemon réatteste lui-même hors verrou. Il
utilise le runtime local connu, HTTP sans redirection, timeout3s,64KiB maximum,
session transport unique puis DELETE. Sous verrou, il recontrôle la route
primaire, identité/instance, binding T3, projet et absence de révocation.
Une course entre preuve et binding ferme l'admission.

<a id="extrait-11"></a>

## Extrait 11

G-P-07 : union d'introspection v1/v2. Trois branches. (a) Un credential
sans aucun fait de permissions jamais existé reçoit l'enveloppe148 v1,
identité seule, quatre champs camelCase, parseur fermé. (b) Pour un credential
dont le fait de permissions a été retiré (fin, échec ou fermeture du tour),
révoqué ou tourné, la fonction `bridget_session` de T3 rend le refus nommé
`permission_attestation_unavailable`, jamais l'enveloppe v1. La façade MCP
privée de Bridget traite ce refus comme un échec de preuve de session. Tout
appel de ce credential est fermé avant effet avec `t3_session_unavailable` :
`bridget_delegate` (nouvelle requête et rejeu d'une requête admise),
`bridget_task_status`, `bridget_task_cancel` et `bridget_who`. Il n'y a ni
repli v1, ni repli discovery, ni repli par PID. Aucun lancement n'a lieu.
(c) Une admission 149 inherit/development
présentée avec la seule enveloppe v1 rend permission_attestation_unavailable,
sans grant demandé, sans fallback discovery, sans spawn ; status, cancel et
identité148 continuent de fonctionner.
« Lecture et rejeu inchangés » de la branche (b) s'entend ainsi : (i) la
lecture humaine Lineage native (gardes 147), l'annulation humaine native et la
saga native continuent, car elles ne dépendent pas du credential T3 ; (ii) un
credential neuf qui n'a jamais eu de fait reçoit l'enveloppe v1 : identité,
status, cancel et rejeu 148 d'une requête déjà admise restent possibles, sans
nouvelle exécution.
Clarification du 2026-10-10 (revue source Sonnet r1, section 5) : branche (b)
précisée, sans changement de contrat. Preuves réelles du cas : recovery149.md
R8.4 et interop149.md S5.1/S5.1b, sur binaire r5.

<a id="extrait-12"></a>

## Extrait 12

Références officielles : [hooks Claude](https://code.claude.com/docs/en/hooks),
[réglages et priorités Claude](https://code.claude.com/docs/en/settings),
[permissions SDK](https://code.claude.com/docs/en/agent-sdk/permissions).
Le schéma T3 exact vient de l'owner de son attestation. Les faits natifs sont
référencés dans la recherche149. Ces contrats ne constituent pas une recette validée.
