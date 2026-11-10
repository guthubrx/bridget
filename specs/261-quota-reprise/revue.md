# Revue indépendante 261 et dispositions

Revue4c4916be-07e7-4ed0-9259-75827be1bc91 : APPROVE_WITH_CHANGES, lecture du
patch autonome, aucun test ou build indépendant revendiqué.

- P1 hypothétique, clé d'instance vide : non confirmé. ProviderLimits::scoped_quota
  appelle exactement BTreeMap::get(&(agent_type.to_owned(), instance.to_owned())).
  Aucun joker ni repli vers un compte voisin. Une instance absente ne permet pas
  au contrôle distant de trouver un quota valide. Le contrôle propriétaire avant
  la projection est testé. Aucun assouplissement de garde introduit.
- Verrou du premier chemin d'erreur : le garde du bloc qui lit la tâche est
  détruit avant remote_dispatch_proof ; cette fonction termine ses propres
  gardes avant retour. La projection ne réentre donc pas dans un verrou détenu.
- Lecture initiale et ordre de publication : confirmés dans le pilote existant.
  Le lecteur appelle push_source pour result/rateLimits avant waiter.send.
  Le test complet utilise le vrai pilote et confirme trois lectures au total.
- Projection quota : fenêtres, dates, état de blocage et booléens de crédits.
  Aucun identifiant de compte, jeton ou donnée d'authentification.
- Deux secondes de délai au repos : compromis borné assumé ; cadence30s et
  exclusion d'un tour déjà actif. Une course vers un nouveau tour ne transforme
  jamais account/rateLimits/read en inférence.
- Limite de chargement : acceptée comme exigence de livraison. La mise à jour
  des fichiers et du core ne change pas un ancien pilote déjà en mémoire.

Contrôles locaux : deux scénarios réels de transport, une projection autorisée
et interdite à une autre connexion, table de qualification160. Tous passent.
La table160 vérifie aussi l'autre instance, le statut global seul et les crédits
utilisables. Aucun test général supplémentaire n'est nécessaire pour ce lot.
