# Archives techniques de l’adaptateur T3

Document d’origine : [specs/136-messages-utiles/adversarial-review-opus-city-coder-1.md](../../../../../../specs/136-messages-utiles/adversarial-review-opus-city-coder-1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date2026-10-06. Agent opus-city-coder-1, UUID29aaed9b-9f6f-4849-87a5-1a23bbe01948.
Annuaire: harnais Claude/T3 ; modèle auto-déclaré GLM5.3 (Z.AI), pas Claude.
Demande mcp-24398-6ac45408-1, borne180s, lecture seule/sans dépenses.
Réponse reçue t3-3f1bf67c-1509-42d0-85a3-4cdeeaaf3f3c, verdict APPROVE_WITH_CHANGES.
Réponse complète consultée en deux fragments16384/8031octets.

<a id="extrait-2"></a>

## Extrait 2

Demande mcp-24398-6ac45699-2, réponse t3-27c68ab1-8aae-4610-8079-0fafb62555b5.
11 615 octets relus intégralement. APPROVE, aucun défaut prouvé. GLM glm-5.3,
Z.AI, harnais Claude/T3 selon auto-déclaration ; pas de mesure externe du modèle.
Lecture seule, aucun test rejoué par le reviewer. Les résultats runtime sont
ceux de notre validation, pas de cette revue.
Audience stockée et History dédié confirmés. La réserve sur membres immuables
est levée par lecture de l'enum complet ThreadAction: aucun ajout/retrait.
La formulation « même séquence que la tête refusée » du retour n'est pas exacte :
remplacer last_seq est légitime, car la nouvelle entrée vaut last_seq+1.
La garantie vérifiée est cible antérieure à la nouvelle, actuelle et non remplacée.
Après réception, correction de l'attente V1 d'un test ancien et renforcement
(dix remplacements + aucun remplacement déduit du texte). Aucun code source
fonctionnel changé après revue.
