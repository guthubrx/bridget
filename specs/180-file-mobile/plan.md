# Plan 180

Référence lue : /home/moi/11.Repositories/t3code-local/apps/web/src/components/chat/QueuedRunsControl.tsx. Le composant attache un volet avec compteur/chevron au composeur, garde le texte visible, puis édite, annule ou promeut une entrée précise. Le chemin Mac demandé /Users/moi/11.Repositories/t3code est absent.

Réemploi Bridget : file native send_deliveries, rétention, remise au repos, annulation/promotion idempotentes et identité mobile existantes. Différence nécessaire : retenir aussi les messages du téléphone avant leur transfert au connecteur T3, afin que les actions restent natives et indépendantes de T3. Le connecteur annonce le tour réellement observé. La modification sous verrou et transaction exige une empreinte du texte courant ; le canon initial de création reste inchangé pour les rejeux.

Aucune nouvelle dépendance. Aucun message utilisateur ancien rejoué pour diagnostiquer la perte supposée. Tests ciblés Rust/JVM ; recette Pixel et preuves-correction ; revue indépendante ; livraison et nettoyage avec conservations documentées.
