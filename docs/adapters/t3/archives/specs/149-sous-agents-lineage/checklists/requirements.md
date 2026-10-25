# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/checklists/requirements.md](../../../../../../../specs/149-sous-agents-lineage/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [ ] Besoin, acteurs et scénarios définis.
- [ ] Lineage T3 : parent/enfant/statut/résultat/journal consultables sans doublon d'exécution ni conversation de premier niveau.
- [ ] Héritage d'autorisations parent sans gain de droits ; GLM et Codex en lecture/écriture dans ces bornes.
- [ ] Aucun grant humain Bridget nouveau ni dialogue de validation de permission ; la garde du grant supplémentaire n'est retirée que dans l'héritage autorisé.
- [ ] Sans posture explicite, l'enfant hérite du parent ; la posture découverte réduit à la lecture seule.
- [ ] Politique sans équivalent sûr chez la cible : refus nommé ; aucune assertion de sandbox que le fournisseur n'applique pas.
- [ ] Droits figés au lancement : rejeu sans élargissement ; changement d'UI en cours de mission sans effet implicite sur l'enfant.
- [ ] Capacités identiques dans un terminal ; permission T3 contrôlée à l'admission seulement, survie du moteur indépendante.
- [ ] Opt-outs explicites existants de Bridget et MCP conservés ; aucun réglage nouveau de projection ; aucun fallback PID ni identité déduite.
- [ ] Scénarios retry, cancel, reprise, parent extérieur, identité inconnue et refus fournisseur documentés.
- [ ] Aucune restriction nouvelle et aucun contournement global des permissions.
- [ ] Livraison par remplacement de fichiers sans redémarrage ; processus courants inchangés ; activation à la prochaine relance humaine ; aucune activation différée.
- [ ] Formalisme canonique présent : Fiche Synthèse, Statut Draft, Priorité P1, Tâches 0/0 (0%), Tests 0/0 (0%).
- [ ] Clarifications bloquantes : aucune identifiée ; la forme exacte des contrats d'attestation et de lecture reste à verrouiller en planification.
