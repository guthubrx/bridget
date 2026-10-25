# Archives techniques de l’adaptateur T3

Document d’origine : [specs/135-controle-missions-silencieux/implementation.md](../../../../../../specs/135-controle-missions-silencieux/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- notify_mission_events constitue le seul moteur de remise pour v2, legacy et
  l'adaptateur Psychologie. Celui-ci garde ses contrôles de hash ROOT, de budget
  total, de dépendances et d'inventaire. Aucun verdict métier n'est modifié.
- Digest unique par destinataire ; worker, coordinateur puis ROOT ; preuve
  différente, nouvelle tentative et nouvelle occurrence réactivent le suivi.
- Génération d'occurrence durable et reprise du token/corps/portée d'une ancienne
  outbox en vol, même si issues était vide au crash.
- Une migration ne déclare un fait déjà remis que si l'ancien dépôt, son hash,
  sa tentative et le propriétaire concordent. L'ambiguïté produit une demande
  groupée, jamais une fausse acceptation.
- Les commandes et l'installateur Claude chargent l'implémentation Codex, sans
  copie du moteur. Les instructions et les tests suivent aussi cette source.
- busy reste joignable ; le transport utilise le client idempotent sans emprunter
  d'identité T3. Le nom du programme et du run restent visibles dans le digest.
  Ce correctif ne crée pas un nouveau profil humain pour un programme autonome.
- Psychologie lit aussi les preuves progress canoniques ; répéter une preuve
  avec un nouvel horodatage ne remet pas son délai à zéro.

<a id="extrait-2"></a>

## Extrait 2

Inventaire contrôlé : LaunchAgents utilisateur, cron utilisateur, variantes
Politique et copies Codex/Claude. La ronde et la relève Maicie sont désactivées ;
le runner de fédération chargé n'émet pas de rappels. Pas de contrôle exhaustif
des services distants, LaunchDaemons système ni de chaque programme indirect.
Les sources déjà modifiées dans dotfiles sont conservées ; pas de reset, pas de
commit global, pas de push ou de redémarrage T3.

<a id="extrait-3"></a>

## Extrait 3

Cache debug inutilisé nettoyé par cargo clean --profile dev. Les sept anciennes
sauvegardes d'installation sont déplacées dans retired-builds, avec une archive
tar vérifiée. Le binaire release conserve son SHA256
02cfdf77ad5b6971572e328bae551dbc1028e74a993abc39778a5263611211e6.
Le daemon et l'adaptateur T3 ne sont pas arrêtés. Les données des missions,
les historiques et les sauvegardes de reprise ne sont pas supprimés.

<a id="extrait-4"></a>

## Extrait 4

SPEC137 : 200 tests PASS frais ; code 3d30a4836b fusionné et poussé dans le fork
T3 local/v0.0.45. Documents fusionnés dans Bridget au commit 79a3c523.
Ses deux worktrees terminés sont retirés après vérification d'ascendance.
L'application T3 installée reste inchangée pour préserver les agents ouverts.
