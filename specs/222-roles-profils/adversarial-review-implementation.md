# Contre-revue222 R1 — lecture adverse et traitement
Agent Claude/Haiku SYS, tâche cc02dca4-3892-4f29-ba32-e5b47724a484.
Patch b34e0ed69d8b72b440d7c9c4b57dc989dd4baf0e4ba2758547c21d025375f941.
Verdict reçu BLOCKED ; revue partielle, sans exécution ni qualification.
Le complément temporaire est arrivé après sa fin ; son envoi unknown_recipient
n'a pas été rejoué. Une nouvelle revue ciblée doit lire ce complément et les fixes.

| Objection | Contrôle et décision |
|---|---|
| P1 CLI local | Le contrôle local réutilise l'autorité administrative même UID du produit (Role Client nu, UID vérifié, capacité et acteur natif). Le hash corrèle une commande ; il n'est pas présenté comme signature ni preuve de présence physique. Un processus administrateur de même UID peut aussi écrire directement SQLite : le produit ne prétend pas isoler un administrateur OS hostile. Conserver ce contrat existant, ajouter confirmation explicite en terminal et documenter la frontière. Ne pas réduire l'administration CLI à un produit Android uniquement sans demande utilisateur. La voie MCP d'agent reste interdite aux mutations humaines. Cette objection conditionnelle reste un risque du modèle local, pas une nouvelle preuve de contournement des droits Unix. |
| P1 portée auto-déclarée | Retenue : validate_scope ne bornait pas au principal. Les claims sont désormais inclus dans le projet attesté ou un grant humain courant ; transversal exige une autorisation transversale humaine. Ne pas imposer un grant à tout contact vivant : ce serait contredire le contrat de rôle déclaré sans lancement ni droit nouveau. |
| P2 rappel sans pertinence | Retenue : avec des termes, filtrer relevance>0. Une requête vide reste la lecture de mémoire. |
| P2 partage interprojets | Le partage est un choix explicite et le rappel non propriétaire exige aussi scope du spécialiste sur le projet courant. L'utilisateur veut des connaissances réutilisables entre projets ; ne pas imposer l'accès au dépôt source pour relire une connaissance volontairement partagée. Une source n'accorde jamais accès au fichier source. |
| P2 vue non propriétaire | Retenue sur les preuves d'autorité : masquer grant/owner aux autres agents ; mandat et budget publics aident à choisir un rôle et ne constituent aucun droit. |
| P2 oubli conserve source/type | Non retenue : tombstone de source nécessaire pour empêcher la recapture. Les textes et anciennes révisions sont retirés. L'oubli de mémoire ne réécrit pas les journaux et canons d'une mission déjà admise. |
| P2 expect grant | Retenue : retour typé human_principal_required au lieu de panique. |
| P3 clés humaines | À contrôler : les UUID émis par interfaces évitent les collisions usuelles, mais les scopes distincts ne doivent pas partager leur résultat de commande. |
| P3 source volumineuse | Les bornes de journal et de résultat limitent les entrées ordinaires ; un échec de capture doit rester visible et pas seulement journalisé. À compléter. |

Aucun verdict d'activation. Les preuves et la revue ne remplacent pas les vérifications finales.


Précisions vérifiées : user_input conserve bien agent_id=owner, donc ne partage
pas le scope human. CLI/mobile représentent l'unique opérateur humain de cette
installation, avec UUID de commande et autorisation recontrôlée par appareil ;
une même clé vise intentionnellement la même décision entre ces interfaces.
Aucun autre profil utilisateur humain partagé n'est annoncé. La correction de
portée des questions utilise maintenant la CommunicationProject attestée dans
le format attendu par mobile_questions, au lieu du seul identifiant de mapping.
Les échecs de capture de réponse laissent un constat unavailable consultable
quand le magasin peut encore l'écrire ; la source native n'est jamais effacée.


Preuve de contrat antérieur pour le P1 conditionnel local : ADR027, section
« Borne de la garde », accepte explicitement l'attribution native/TTY qui arrête
les outils déclarés, pas le processus hostile du même compte. referent_control.rs
lignes20–35 reprend ce contrat. L'ADR011 du24août nomme l'exposition et le second
facteur différé. 222 ne crée aucun type exécutable dans le registre et réutilise
ce contrôle accepté ; le hash n'est pas appelé preuve cryptographique.
La preuve conversationnelle prend désormais la portée de l'acteur humain réel :
CLI/UI natifs, ou appareil mobile actif avec SpecialistsManage. Cela autorise une
décision multiprojets réellement humaine sans confondre la portée de l'agent et
celle du décideur. approved_by cite le canal de décision, pas l'agent demandeur.
