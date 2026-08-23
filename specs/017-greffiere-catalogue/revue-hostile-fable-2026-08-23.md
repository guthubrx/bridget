# Revue hostile de conception — session 017 (2026-08-23 soir)

**Croisement** : relecture par un moteur distinct de celui de l'auteur.
**Objet** : spec.md + plan.md + checklist au commit f6648c7, jugés contre le
cadre-loi revue-adverse-boucle-amelioration-2026-08-23.md.
**Verdict : AMENDER** — fidèle au cadre-loi sur toutes les frontières de
jugement ; le trou est factuel, pas doctrinal.

## À amender
- **G1 [MAJEUR]** Le lien constat↔objectif n'est JAMAIS déclaré : add ne
  porte aucun champ objectif, transition exige (constat_id, objective_id),
  et aucun fait ne crée la paire — toute déduction serait l'inférence
  interdite. Chaîne saine désignée : la délégation d'arbitrage FR-1708
  porte constat_id comme fait déclaré journalisé (ou kind fermé `link`).
- **G2 [MOYEN]** JSON en commentaires Markdown = entrées INVISIBLES au rendu
  — le pire hybride. Sorties : grammaire fermée VISIBLE (ligne/tableau à
  syntaxe figée) OU journal assumé avec `registre list` comme vue humaine ;
  si prose visible par entrée : dire laquelle fait foi.
- **G3 [MOYEN]** Après migration, ~50 entrées « en attente de qualification »
  sortent de la vue ET du pied de page → la feature recrée l'oubli qu'elle
  corrige. Exiger un 4e décompte « P en attente » (une ligne).
- **G4 [MOYEN]** Idempotence de `constat add` non spécifiée (id dupliqué :
  refus ou no-op ? retry après crash ?). Une phrase.
- **G5 [MINEUR]** SC pour les filets skill et rituel (seul le pied de page
  en a un).
- **G6 [MINEUR]** Sémantique d'« append atomique » (O_APPEND/lock vs
  temp+rename interdit) — au data-model.

## Conforme
- **G7** FR-022 : id-only partout, homonymie lot C citée, vue pure golden.
- **G8** Interdits du cadre-loi TOUS hors périmètre, réhabilitation du
  planifié respectée.
- **G9** Migration verbatim, rien d'inventé, ambiguïté exposée jamais résolue.
- **G10** XIX : noyau minimal réel, catalogue par projet en config, incidents
  fondateurs cités, délégation obligatoire (qui résout G1 gratuitement si
  elle porte constat_id). Phasage séquentiel approprié.
