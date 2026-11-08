# Livraison245 — 8 novembre2026

## Cause et correction

Le troisième échec Revue146 est postérieur au premier correctif244. Le refus
`seven_day` SYS/Haiku a été observé à1794137600. L'admission Opus commence
à1794137626, puis échoue à1794137675. Les deux lanceurs diffèrent, hors
`--model`, seulement par `capabilities.models`. Cette matrice séparait encore
les empreintes d'un même compte. Le défaut résiduel est confirmé.

245 retire uniquement cette matrice de la clé. Machine, configuration de compte,
worker et révision restent dans la clé. Les observations existantes sont déplacées
sous transaction, en gardant la plus récente par fenêtre et ses dates originales.
Un alias est conservé en mémoire si le déplacement échoue. Une réponse positive
ultérieure solde la fenêtre canonique et ses anciens alias dans une transaction.
Si cette écriture échoue aussi, le refus constaté reste conservé et le défaut
d'effacement est journalisé. Aucun horaire fournisseur n'est inventé.

Le guidage MCP spécialistes précise `version:1`, `search/list/get`, l'identifiant
requis pour `get` et un exemple JSON valide dans le refus. Le schéma strict et les
droits restent inchangés. Une ancienne façade ouverte peut conserver son ancien
guidage jusqu'au prochain chargement. Aucun redémarrage de T3 n'a été effectué.

## Vérifications ciblées

Cinq tests distincts passent sur Mac et Linux, sans inférence fournisseur :
isolement des comptes et alias de modèles ; cache et date inconnue ; migration
des deux alias, dates et absence de résurrection ; panne SQLite puis réponse
positive et redémarrage ; guidage MCP avant connexion. Les répétitions R1/R2 et
les deux plateformes ne sont pas additionnées en tests distincts.

Revue R1 `b8ff093d-78f4-4e23-933f-eaaf469c38a5` terminée, lue et ACKée :
un P2 réel sur la réponse positive après migration échouée, corrigé par
`bc8b9d6db35a71cdc1966605e5b75d5e05821405` avec régression ciblée verte.
Revue R2 `8e149b1b-1c0c-42e2-82ee-a3bf6ad6bbcd` terminée, lue et ACKée,
mais non concluante : lecture contrôlée refusée `invalid_envelope`.
Le fichier demandé existe dans sa vue. La cause de ce refus n'est pas établie ;
son journal après arrêt est indisponible (`AgentStopped`). Aucun verdict favorable
indépendant R2 n'est inventé. Le dernier delta a été auto-relu et testé ; aucune
troisième mission n'a été créée pour prolonger la livraison.

## Activation et état réel

Source exécutable : `bc8b9d6db35a71cdc1966605e5b75d5e05821405`.
Version0.8.11 / image245-quota-r1 chargée et vérifiée dans les sept composants
sur monordinateur, Cartae, SYS et KS. Coupure du maître :1,82s.

- Mac :228bc721bcc23b665cf784125df497f17b09abd57bd75edf5a48db1ac705a0a8
- Linux :9f7404775b3461a7be6ca2d584c3603b8b84e8d3377f5ddf7c554ab77309c4cd

Le cache réel est passé de six empreintes à trois comptes. Les trois faits
canoniques ont leurs dates d'observation originales et le même retour
1794204000. Ce contrôle ne lance pas une nouvelle mission pour vérifier Claude.
Session nativeV2 et annuaire du même projet réussis après activation.
Quatre rapports0.8.11 `up_to_date/fresh` constatés.

Retour244-reference-r5 installé, empreintes et formats aller/retour/inverse
compatibles, pointeurs `rollback` écrits atomiquement sous verrous natifs.
Les images243 restent incompatibles avec les files privées244.
Le retour244 conserve les formats mais réintroduirait le défaut de sélection
des catalogues de modèles ; il n'a pas été activé pour tester.

Sept tâches protégées7024/77ba/702d/2fb et trois Revue : payloads identiques
avant/après. Superviseur Cartae2483992/birth365816122 conservé au passage.
Registre spécial MacSHA4f730d9ae9e07f41a5b67c6cc4bc8f89aa52088fe597d38438118723fb384e24
et neuf processus de l'application T3 inchangés. Les tâches Revue146 échouées et
la revue Codex8d3a9fb3 sont laissées dans leur état ; aucun rejeu métier.

## Publication, avis et limites

Release publique : https://github.com/guthubrx/bridget/releases/tag/v0.8.11.
Six actifs vérifiés par empreinte API puis téléchargement sans authentification.
APK8011, certificat0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588
et lignée de signature publique8010 vérifiés. Interface Android inchangée :
preuves visuelles244 réutilisables, aucun nouveau parcours visible modifié.

Diagnostic et avis d'activation adressés aux deux émetteurs après présence native
vérifiée. Reçus d'activation :52945386-853b-4796-b042-5a6e230d9592 (Revue),
7b9a1b05-70ad-4d2d-befd-91d9a048c07e (Parix). Le reçu initial est `in_flight` ;
il ne prouve pas la lecture métier. Aucun message adressé au nom non routable human.

Le signalement sentinelle concerne la campagne244 déjà arrêtée. Son échec et ses
limites restent conservés ; minuteur inactif, aucune nouvelle campagne Claude.
Le fait `unknown_recipient` n'est pas attribué au quota sans son destinataire et
son reçu exact. Aucune récupération automatique des missions en quota ajoutée.

Preuves privées, reçus, sorties et sauvegardes de maintenance :
/Users/moi/.cache/bridget245-quota-contract/.

## Git et entretien

Code intégré à main et poussé. Bilan documentaire intégré ensuite, puis sources
alignées et propres sur les quatre hôtes. Worktrees/branche propres245 retirés
après intégration et contrôle des usages. Staging SYS nettoyé par `installed` :
3,13Gio retirés ; cache partagé Mac5,72Gio conservé sous son budget10Gio pour les
prochaines compilations. Images active245 et retour244-r5 conservées. Preuves,
vues retenues, travaux étrangers et anciens programmes encore chargés conservés.
Les transferts245 devenus inutiles sont retirés après publication vérifiée.
