# Carte de reprise — essai-claude-distant-flux

> **PREMIER GESTE OBLIGATOIRE, AVANT TOUTE AUTRE LECTURE**
> 1. LIRE d'abord `/home/moi/.cache/bridget/agent-domains/essai-claude-distant-flux` et
>    `bridget who`, et RAPPORTER les deux **avant** de corriger quoi que ce soit.
>    `bridget domain` réécrit ce fichier à chaque appel, même sans changement : le geste
>    de correction détruit la trace qu'il faudrait observer. Je n'ai pas fait cette
>    lecture préalable le 28/08 et je n'ai donc **jamais pu attester mon propre delta**.
> 2. ENSUITE seulement : `bridget domain bridget`, puis vérifier la sortie.
> 3. Accuser par **envoi Bridget explicite** (`bridget send --to bridget "…"`) — répondre
>    dans le terminal seul ne parvient à personne.

Carte **écrite par son agent**, le 28/08 vers 16h05, sous mandat
`objective_id d1cabab5-b873-4453-b46e-8fd01e834f82` /
`delegation_id 177fc1af-62bf-46f2-a005-55b95b10d7ed` /
`message_id 8a72d784-1b96-413a-aaf3-c052f1d90c67`.

---

## 1. Identité et état — MESURÉ par moi

- Agent : `essai-claude-distant-flux` — type `claude`, transport `claude_stream_json`, canal `cli`
- Modèle : `claude-opus-5` — `spawn_commands` : 1 ligne, **génération 453, `persistent=1`, `connected`**
- Domaine : `bridget`, surcharge écrite le **28/08 à 15:09:39** dans
  `/home/moi/.cache/bridget/agent-domains/essai-claude-distant-flux`
- Prédécesseur : `essai-claude-distant`, **`claude`/tmux** (pas codex), pane `essai-claude-distant:1.1`
- Aucune mission exécutée. Gel humain respecté de bout en bout : rien entrepris sans mandat.

## 2. Chemins absolus

| Objet | Chemin |
|---|---|
| Poste (répertoire courant, **hors dépôt**) | `/home/moi/revue/essai-claude-distant` |
| **Point de travail réel** (dépôt git) | `/home/moi/revue/essai-claude-distant/bridget` — branche `main`, HEAD `e75aa3b`, **1117 commits**, arbre propre |
| Mémoire projet (créée par moi) | `/home/moi/.claude/projects/-home-moi-revue-essai-claude-distant/memory/` |
| Surcharges de domaine | `/home/moi/.cache/bridget/agent-domains/` |
| Base du daemon (ledger, usage, spawn) | `/home/moi/.cache/bridget/bridget.db` |
| Cartes de reprise | `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` |
| Autres checkouts bridget (6 au total) | `/home/moi/bridget`, `/home/moi/bridget-main-20260824`, `/home/moi/bridget-referent`, `/home/moi/bridget-registre`, `/home/moi/bridget-ui-resize-20260828` |

## 3. Ce que j'ai ÉTABLI — mesuré par moi, reproductible

1. **Le domaine dérive du nom du répertoire de lancement.** `derive_domain` :
   `basename(git rev-parse --show-toplevel)`, repli sur `current_dir()`. Mon poste n'étant
   pas un dépôt, mon dérivé vaut `essai-claude-distant` — **faux**. Mon prédécesseur, lui,
   tournait dans `.../essai-claude-distant/bridget` : son dérivé valait `bridget`, juste
   **par coïncidence de nommage**.
2. **Trois tmux n'ont aucune surcharge** : `essai-claude-distant`, `essai-distant`, `rc7`.
   Deux portent `bridget` par accident de répertoire ; `essai-distant` non. Ce n'est donc
   **pas un oubli isolé** mais une dépendance silencieuse au nom du répertoire — deux
   pièges restent armés si l'on ne corrige qu'`essai-distant`.
3. **`connected` sur un tmux atteste l'existence d'un pane, pas la disponibilité.**
   Preuve : mon prédécesseur, saturé et muet, affiché `connected`.
4. **L'état du parc est un échantillon volatil** : la ronde du référent occupe ceux qu'elle
   mesure. Trois relevés en quinze minutes, trois résultats, aucun faux.
5. **L'attribution des messages est complète en base et non exposée par le CLI.**
   `bridget ledger` rend 20 messages et refuse tout argument (`all`, `requests`, `full`,
   `100` rejetés). La table `ledger` (`id, ts, sender, target, body`, `sender` NOT NULL)
   conserve tout. **Débit mesuré : 21 messages vers `bridget` en 12 minutes** — sa fenêtre
   de lecture couvre moins de douze minutes, d'où des rapports perdus sans que personne
   ne soit fautif.
6. **`bridget who` n'expose aucune consommation.** La fenêtre 5h n'a qu'une échéance, sans
   pourcentage, et reste affichée **après son terme** (« rst 15:50 » lu à 15:53:00 UTC,
   figée sur 20 s). La veille se fait sur la table **`usage_samples`**, fraîche à la seconde.
   Mesures : fenêtre 5h précédente = 11 agents / **805 225 tokens** ; début de fenêtre en
   cours = **115 603 tokens en 3,3 min** (~35 k/min, ~13× le régime précédent) ;
   fenêtre 7 j = **2 435 336 tokens pour « 1 % »** affiché.
7. **Asymétrie de survie, vérifiée agent par agent** : les dix tmux ont **zéro** ligne
   `spawn_commands` ; les dix `-flux` en ont une ou plusieurs, toutes `persistent=1`.

## 4. Ce que je tiens D'AUTRUI — non mesuré par moi

- Saturation du prédécesseur à 97 % d'un contexte de 1M, et son silence de dix-sept heures
  (référent). Je n'ai mesuré ni l'un ni l'autre.
- Le code réellement servi serait `16be24f` malgré le build-id `f7658d4d9746-dirty`
  (référent). **Ce SHA n'existe pas dans mon dépôt** (`Not a valid object name`).
- Règle « une délégation `créée` ne dit rien du travail réel » (rc5) — adoptée, non vérifiée.
- « Population mesurée changée » plutôt que déterminabilité acquise : formulation
  d'`essai-distant-flux`, meilleure que la mienne. Je ne me l'attribue pas.
- Témoin `temoin-persistance`, génération 454, `persistent=1` (geste de rc5-flux) —
  **celui-là, je l'ai vérifié en base : il existe**.
- Le mandat ci-dessus : **vérifié au greffe à 16h10** (réserve initiale levée). Objectif
  `d1cabab5…` présent, état `en_coordination`, champ `but` identique au message reçu ;
  délégation `177fc1af…` présente, participant `essai-claude-distant-flux`, état `creee`
  **alors que le livrable était déjà rendu** — illustration directe de la règle rc5.
  Forme qui fonctionne : `maicie status --config /home/moi/.config/maicie/config.json`.
- **DEUX GREFFES, dont un périmé — piège vérifié.** Le chemin le plus naturel,
  `/home/moi/.local/state/maicie/maicie.sqlite3`, est **figé au 27/08 14:16** : les quatre
  identifiants du 28/08 cherchés (`d1cabab5`, `177fc1af`, `587da26d`, `bedaf54d`) y sont
  **absents**. Le greffe actif est `/home/moi/.cache/bridget/maicie-state/maicie.sqlite3`,
  désigné par `database_path` dans `/home/moi/.config/maicie/config.json`. Conclure sur le
  premier mène à rapporter, à tort, que les mandats ne sont pas inscrits.

## 5. Ce que je NE SAIS PAS, et qu'il faut cesser de deviner

- **L'objet des délégations de mon prédécesseur — et même leur nombre.** Ma carte d'arrivée
  en annonçait **trois** à l'état créée ; sa carte de reprise en compte **cinq**. Divergence
  non résolue. Ne pas inférer leur avancement de leur état.
- Si ma surcharge de domaine a **corrigé** quoi que ce soit : le mtime atteste une écriture,
  pas un delta. Le faisceau penche vers une correction réelle (mon dérivé aurait été faux) —
  ce n'est pas une preuve.
- Le plafond réel de la fenêtre 5 h. Aucun compteur ne l'expose.
- Si `persistent=1` protège quoi que ce soit : **jamais éprouvé**. C'est l'objet du témoin.
- Ce que valent les neuf autres cartes : **je ne les ai pas lues**, délibérément — charger du
  contexte sans emploi est le mode d'échec qui a tué mon prédécesseur.

## 6. Pièges rencontrés — à ne pas repayer

- **Mesurer au mauvais niveau.** J'ai déclaré « pas de dépôt git » après avoir testé la seule
  racine. Le dépôt était un cran plus bas. Tester le poste **et** ses sous-répertoires.
- **Citer une coordonnée de code sans son arbre.** `wrapper.rs:983` n'est reproductible dans
  aucun des six checkouts (ligne 630 dans `/home/moi/bridget-main-20260824`, **945** dans
  `/home/moi/revue/essai-claude-distant/bridget`). Toujours citer l'arbre avec la ligne.
- **Deviner une syntaxe CLI.** Après deux formes refusées, lire le binaire plutôt que tenter
  une troisième fois.
- **Ne pas exécuter d'action d'écriture pour « vérifier »** : `maicie profile --definition`
  pose une définition, ce n'est pas une lecture.

## 7. Corrections dues au registre

1. **`essai-claude-distant.md`, §« Ce que son successeur a établi », est faux sur un point,
   et j'en suis la source** : « `/home/moi/revue/essai-claude-distant` n'est pas un dépôt
   git : ni branche ni historique. Toute mission de code exige de lui donner un point de
   travail d'abord. » → le poste réel `/home/moi/revue/essai-claude-distant/bridget` **est**
   un dépôt (`main`, `e75aa3b`, 1117 commits). Mon rapport initial a induit cette ligne ; le
   référent a ensuite fondé une décision dessus. **À corriger dans ce fichier.**
2. Même fichier, l'en-tête cite `wrapper.rs:983` — coordonnée non reproductible (cf. §6).
3. Divergence non tranchée sur le nombre de délégations du prédécesseur : **3** (carte
   d'arrivée du successeur) contre **5** (carte de reprise).

## 8. Ce qui reste à faire

Rien n'est en cours. Aucune mission entreprise, aucun code écrit, aucun fichier du parc
modifié hors cette carte et ma mémoire projet. Proposition déposée et **non exécutée**, en
attente des trois identifiants : mesurer la vitalité réelle des dix tmux par sollicitation
minimale (qui accuse, en combien de temps) — lecture seule, réversible, sans arrêt.
