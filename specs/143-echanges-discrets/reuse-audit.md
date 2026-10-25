# Audit de reutilisation de l'existant — SPEC143

## Decision

Statut: PASS
Date: 2026-10-07
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets

Conclusion courte: Quatre fichiers existants suffisent : logique, rendu et leurs deux suites de tests. La projection sortante porte la compatibilité de deux fournisseurs ; aucun moteur ou framework n'est créé. La lecture principale du gate reste requise avant tâches.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audites | 8 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

Aucun. Le rendu natif et les protections141 sont déjà au plan.

## Duplications evidentes

Aucune. La logique entrante n'est pas un parseur des work entries MCP sortantes.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/AGENTS.md` | Session et non-restart | Essais isolés, pas de déploiement |
| `/Users/moi/.speckit/constitution.md` | Réutilisation et compatibilité minimale | Quatre fichiers, zéro dépendance |
| `/Users/moi/.speckit/ref/standards-frontend.md` | UI et accessibilité | Bouton, clavier, focus ; exception ciblée Next.js |
| `/Users/moi/.speckit/ref/standards-tests.md` | Tests significatifs | RED/GREEN et interactions, pas snapshots seuls |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC139 | Entrées compactes | Réutiliser projection et controls |
| SPEC140 | Familles Bridget reconnues | Pas de reconnaissance par simple mot |
| SPEC141 | Groupes directs et copie brute | Conserver états, fallback et copie exacte |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg 'BridgetMessageBody|PlainWorkEntryRow|projectBridgetEnvelope|onToggleWorkEntry'` | Deux fichiers chat existants | Projection/rendu/états réutilisables confirmés |
| Lecture wrapper2125/2130 et row4915 | Timeline143 | Fond/bordure ciblés ; toggle complet déjà présent |
| Discovery principal des payloads | WorkLog Codex/Claude | Deux formes MCP send attestées, même work entry |
| Revue du contexte et sources WAI/tests | Artefacts et références utilisateur | Formats non confirmés natifs, completed non livré |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Helpers sortants locaux | Créer extension typée minimale | Compatibilité Codex/Claude, sans équivalent sortant identifié ni nouveau fichier | 2026-10-07 |
| Autres formats sortants | Repli natif | Aucune forme non confirmée inventée | 2026-10-07 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Gate distinct du complément US4

Date : 2026-10-07. Statut documentaire : PASS avant code, sur les findings du codeworker transmis par le principal. Ce gate ne requalifie pas l'audit historique ni les tests346 en preuves US4. Le GO d'implémentation relève du principal.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/reuse-audit.md#extrait-2).

Création justifiée : une projection locale typée du texte assistant, car la projection MCP du socle porte sur des work entries et ne doit pas devenir un parser générique. Les quatre fichiers suffisent. Frontière retenue : `---` hors bloc de code, ligne blanche puis `Résumé pour toi :` ou `Pour toi :`, simple ou en gras. Variantes incomplètes, frontières multiples et fences ambigus restent natifs. Le streaming et une citation ciblée gardent le natif.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/reuse-audit.md#extrait-3).

Révision de compatibilité après contre-revue : le flux de citations existant est réutilisé sans changement d'offsets. Le corps d'une réponse mixte reste monté, seulement masqué CSS (`display:none`, pas `hidden`/`aria-hidden`). Ce coût de rendu complet existait avant143. Un message avec références Markdown, notes de bas de page ou HTML hors code susceptibles de traverser la frontière reste natif. Aucun parseur de résolution Markdown ni adaptateur de citation nouveau n'est créé ; le repli sûr évite cette duplication.
