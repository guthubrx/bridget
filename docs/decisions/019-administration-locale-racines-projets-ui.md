# ADR-019 - Administration locale bornée des racines projets depuis l'interface

**Date**: 2026-08-30
**Statut**: Proposé
**SPEC liée**: SPEC-076

## Contexte

SPEC-065 a créé une politique de racines fail-closed chargée depuis un fichier
protégé et limité les mutations projet à la CLI le service compagnon. Cette limite appartenait
à la tranche de fondation sans interface projet.

L'utilisateur veut créer, importer, reconnecter et retirer des projets depuis
l'interface, sans laisser agents ou surfaces génériques parcourir le serveur.

## Décision

SPEC-076 remplace cette limite seulement pour une interface locale
authentifiée, avec intentions typées, versionnées et confirmées.

Les racines restent contrôlées par le daemon. Toute modification est validée,
atomique, versionnée, rechargée et attestée. Une racine excluant un projet actif
ne peut pas être retirée.

le service compagnon conserve ProjectIdentity et la saga. L'UI n'accède jamais à sa base et
ne reçoit aucune commande shell libre. MCP, agents et API générique restent
exclus.

## Conséquences

- Les réglages permettent à l'administrateur d'agir explicitement.
- Les agents ne gagnent aucun droit fichier.
- Le daemon doit prouver un reload sûr, pas seulement écrire une préférence.
- Les mutations réutilisent audit et idempotence SPEC-065.
- Politique absente ou invalide : aucune mutation possible.
