# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/quickstart.md](../../../../../../specs/151-attente-fin-de-tache/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Vue structurée (`work_state`, résumé, détail) : filtre `native151_vue`.
2. Boîte de fin, lecture pure, message après attente : filtre `native151_boite`.
3. Attente de tâche (événement, expiration, curseur, capacité, client fermé) :
   filtre `native151_attente`.
4. Façade MCP (4 attentes, annulation, progression à 64 attentes) : filtre
   `native151_facade`.
5. Regroupement de plusieurs résultats : filtre `native151_groupe`.
6. Attente de la réponse d'un envoi (résumé figé, deux scopes, `reply_unavailable`) :
   filtre `native151_envoi`.
7. Bout en bout avec parent et enfant synthétiques : filtre `native151_e2e`
   (fichier `crates/bridget-daemon/tests/native_delegation_e2e.rs`).
8. Famille de message du pont : filtre `batch` dans `t3code.rs`.
9. Accusé explicite (US7) : filtre `native151_ack`.
10. Non-régression 148 à 150 : filtres `native148`, `native149`, `native150`,
    `delegation149`, `spec_087`.
