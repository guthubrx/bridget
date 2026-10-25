# Archives techniques de l’adaptateur T3

Document d’origine : [specs/113-fiabilite-remises/spec.md](../../../../../../specs/113-fiabilite-remises/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Fiche synthèse
Spec: 113-fiabilite-remises | Statut: Implemented (livré 2026-09-19) | Priorité: P1 | Date: 2026-09-19
Branche: session-113-tete-de-file | Dépend de 111 et 112 (attente en file du pont T3).
Origine : contre-revue adverse d'evols-t3 (Codex) du 2026-09-19, voir `adversarial-review-evols-t3.md`.

<a id="extrait-2"></a>

## Extrait 2

## Problème observé
Le pont T3 écarte une remise dans quatre cas : file saturée (64 remises), échéance passée avant
démarrage, demande suivie close côté daemon, annulation. Dans les quatre, il ne dit rien au daemon.
La saga d'envoi idempotent reste en phase `dispatching` jusqu'à son expiration (7 jours) et
l'expéditeur voit « remise en vol » pour un message que le pont a jeté. Le 2026-09-19, la base de
production comptait 40 sagas `dispatching`, dont 33 d'août ; 14 remises « périmées avant démarrage »
ont été jetées en silence dans la journée. Le wrapper ACP, lui, signale déjà une injection ratée
(`DeliveryRejected` + `DeliveryIndeterminate`).

<a id="extrait-3"></a>

## Extrait 3

## Hors périmètre
- Regrouper plusieurs remises dans un même tour (proposition écartée par la contre-revue pour les
  `reply=true` : une réponse par tour).
- `SteerCurrent` sur le pont : dépend du support T3.
- Balayage des 33 sagas d'août : elles expirent d'elles-mêmes à 7 jours.
