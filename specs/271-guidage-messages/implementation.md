# 271 : faits et décisions d’incident

Les deux revues 04e70286 et ba63377c sont des doublons déjà traités en 270.
Les anciens steer Claude non confirmés correspondent au défaut corrigé en 269.
Les tâches 0ff8e257 et 7835103f sont désormais annulées et cleanup_done=true.
9be88de5 poursuit sa ronde 8 : ne pas rejouer les anciennes consignes.

M1 a3efaa16 : provider_result success à 17:26:00Z, nouveau provider_result puis
turn_end à 17:32:38Z. Résultat durable disponible à 17:32:43Z, 5595 octets,
result_sent=true et delivery_acked=true. Aucun résultat ni code métier modifié.
Le statut working ne se déduit pas d’un seul résultat fournisseur : le pilote
attend les travaux de fond avant de terminer son tour. Les queues admises avant
la fin de mission ont ensuite été traitées aux tours suivants. Le worker M1
utilise encore 0.8.35 ; sa mise à niveau doit être qualifiée sans toucher T3.

Défaut prouvé : le refus précoce nommant task_continue après résultat ne traite
que steer. Une queue vers le même enfant tombe dans le contrôle des projets
physiques et rend cross_project_reason_required. La correction doit nommer la
fin de ronde, sans ouvrir de mission ni élargir les droits.

Économie : e2d6d3bf est annulée, cleanup_done=true. La trace worker conservée est refusée après la demande d’annulation, sans marker
ni native_ready. Elle ne prouve pas un refus initial du worker. Aucune remise de
mission ni exécution ne sont attestées par ce reçu.
Le motif amont de WorkerUnavailable n’est pas conservé dans ce reçu historique.
Le code de sortie du processus de préparation de l’autre refus est actuellement
perdu : le helper rend le même code pour un retour non nul et une réponse trop
grande. Le nouveau diagnostic doit conserver seulement opération, code de sortie
et cause fermée, jamais stderr ni sortie privée. L’ancien refus reste inconnu.

Réponse relayée t3-ac7b7169 : demande answered à 17:08:57Z, mais aucune entrée
de réception ou de steer de cette réponse dans le journal du tour Codex toujours
actif. La réponse part sans intent de contrôle ; les messages ordinaires sont
mis en file. Les 22 autres steer ont été injectés dans ce même long tour.
La règle existante réserve le pilotage au contrôle explicite ; ne pas convertir
silencieusement une réponse ordinaire en steer ou interrupt. Informer du parcours
lecture du ledger, ou guidage explicite autorisé, et préserver l’accord conservé.

Décision supplémentaire de François : plafond des agents à 128. Le service maître
portait une surcharge BRIDGET_FLEET_QUOTA=24 ; le défaut produit était 16.
La surcharge et le défaut portable sont maintenant à 128 (commit cfdf1a82 poussé).
Le maître PID4104168 confirme la variable chargée ; redémarrage de 3,77 secondes,
11 superviseurs conservés avec les mêmes PID et dates de naissance. Les sources
Mac, Cartae, SYS et KS sont alignées sur ce commit pour ce changement. Les autres gardes et
les limites des fournisseurs ne changent pas. La variable est lue au démarrage :
un simple export dans le terminal ne suffit pas. Préserver les missions lors du
chargement de ce réglage, sans arrêt de T3.

Le protocole entre workers refuse les suffixes JSON du champ preparation_error.
Le détail opération/exit reste donc dans le journal du worker source, corrélé par
l’identifiant de contrôle. Le code transporté reste fermé, sans changement de
schéma ni élargissement du validateur. La première proposition JSON a été
écartée avant publication. Un test exécute réellement le helper et valide la
forme de réponse, ainsi que le refus de l’ancienne proposition incompatible.

Le test historique de préparation locale attendait à tort une liaison SSH
absente après passage à une vue locale. Le résultat réel est machine_unknown
car son registre de fixture est vide. L’assertion suit désormais ce parcours
254 ; le comportement de production n’a pas changé.

Validation ciblée sur Mac et Debian 12 : guidage de bout en bout (92 tests),
flux fournisseur et attente des tâches de fond (25 tests), nouvelles régressions
271 (4 tests), garde 269 (4 tests), spool (4 tests), flotte (18 tests), ledger
(28 tests), préparation (4 tests), preuve d’origine (8 tests), guidage admission
(4 tests). Ces sélections peuvent se recouvrir et ne représentent pas un total
de tests distincts. Les 10 tests Python de préparation passent aussi. Une seule
régression de guidage a été rejouée après précision de son texte et rangement
dans le module de tests existant. Clippy strict workspace/all-targets passe sur
les deux plateformes, après corrections syntaxiques ciblées des fixtures.

Quatre revues indépendantes : exemptions de la garde et rejeu conservé vérifiés,
canonicalisation du spool approuvée, compatibilité du code fermé approuvée. La
dernière remarque de corrélation est traitée dans le guidage : il donne le
lecteur de journal configuré et le rapprochement par heure/opération/exit, sans
prétendre que correlation_id est présent dans le reçu du demandeur.

La lecture ciblée du journal KS de 15:50 à 16:00Z ne contient aucune corrélation
avec e2d6d3bf. Le refus historique ne permet donc pas de nommer sa cause amont.
Aucune tâche annulée n’a été rejouée pour compléter ce diagnostic.
