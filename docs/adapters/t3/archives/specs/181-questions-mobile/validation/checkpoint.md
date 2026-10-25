# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/validation/checkpoint.md](../../../../../../../specs/181-questions-mobile/validation/checkpoint.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Rust : dernier lot181 17PASS ; transports3PASS ; HTTP13PASS ; scopesMCP4PASS ; contratT3 19PASS/1ignored. Régression155 :62PASS/1FAIL initial, cas de corrélation isolé PASS après garde de session. Aucune FULL et aucun GREEN global déduit.
- JVM23PASS ; lint, assemblages debug/application de test PASS. Dernier changement limité à la sélection exacte de la notification dans la fixture, assemblage de test PASS.
- Android réel : choix simple+complément, choix multiples, fermeture forcée/reprise du brouillon et confirmation MCP par le parcours natif Bridget. Réponse TTY depuis un autre client, même demande appliquée.
- FCM réel en arrière-plan : choix via bouton système UiAutomation, une seule commande applied. Hors réseau : bouton système Valider, sauvegarde locale avant accès réseau, test1PASS4,124s ; rétablissement puis appui Envoyer la réponse conservée : même commande7af1bf1f-972f-4563-8f28-41eb1d6df779 applied. Aucune seconde commande.
- RemoteInput : vrai PendingIntent de la notification, résultat texte SDK ajouté dans le test ; une commande, sélection vide et texte exact, confirmation producteur,1PASS0,811s. Ce test vérifie le traitement Android, pas la frappe manuelle du clavier de notification.
- Reprise daemon isolé : demande8f0d762c-66c3-4c70-94ae-26550181d1ad pending avant/après arrêt normal et reprise avec même instance ; réponse TTY ultérieure applied. Production intacte.
- Lien de question : formulaire ciblé dans la conversation de l’agent attesté, provenance/machine affichées ; capture finale. La fixture ne fournit pas de journal d’agent : suivi interrompu attendu, questions fonctionnelles.
- Dictée contrôlée :1PASS3,494s, intention système observée, texte éditable, aucun envoi automatique. Reconnaissance vocale réelle NON VALIDÉE : transfert audio refusé par Android puis injectAudio officiel a fait crasher QEMU. Après deux essais, voie abandonnée et passage Android15 ; aucune capture du micro du Mac.
- Adaptateur T3 : worker réel, serveur HTTP/WS contractuel contrôlé, publication/réponse/réconciliation sans prompt,1PASS. Pas d’essai d’une question réellement générée dans l’application T3 active ; aucun redémarrage T3/fournisseur.

<a id="extrait-2"></a>

## Extrait 2

RouteCaddy temporaire @id bridget-questions181-qa vers127.0.0.1:19581 à retirer seule par CAS/ETag. NamespaceQA /home/moi/.cache/bq181 ; producteurs/core/mobile181 propres à arrêter normalement après recette. Émulateur35 à conserver utilisable, AVD36 instable à examiner sans perte des preuves. Budget caches10Gio/7j, nettoyage automatique différé car observation processus incomplète ; garde non contournée. Tâche historiquee595 non réconciliée préservée. T3 et autres travaux protégés.
