## Common

done = Fini!
continue = Continuer
cancel = Annuler
and = et

## Commands

# Classes
classes = classes
classes_add = ajout
    .description = Ajoute une nouvelle classe au serveur
    .name = nom
    .name-description = Nom de la nouvelle classe à ajouter
    .level = niveau
    .level-description = Niveau parent de la classe
    .role = role
    .role-description = Un rôle à assigner à la classe, sinon un rôle est créé
classes_add-success = La classe `{$class}` à bien été crée dans le niveau `{$level}`.
classes_add-no-such-level = Il n'existe pas de niveau avec le nom `{$level}` sur ce serveur.
classes_remove = supression
    .description = Enlève une classe du serveur
    .name = nom
    .name-description = Nom de la classe à retirer
classes_list = liste
    .description = Liste les classes du serveur
    .filter = filtre
    .filter-description = Filtre les classes avec un nom
classes_list-title = Liste des classes
classes_list-title-with-filter = Liste des classes avec le filtre `{$filter}`
classes_list-none = Il n'y a pas de classes sur ce serveur.
classes_list-none-with-filter = Il n'y a pas de classes sur ce serveur avec le filtre `{$filter}`.

# Groups
groups = groupes
groups_add = ajout
    .description = Ajoute un nouveau groupe au serveur
    .name = nom
    .name-description = Nom du nouveau groupe à ajouter
    .role = role
    .role-description = Un rôle à assigner au groupe, sinon un rôle est créé
groups_add-success = Le groupe `{$group}` à bien été crée.
groups_remove = supression
    .description = Enlève un groupe du serveur
    .name = nom
    .name-description = Nom du groupe à retirer
groups_list = liste
    .description = Liste les groupes du serveur
    .filter = filtre
    .filter-description = Filtre les groupes avec un nom
groups_list-title = Liste des groupes
groups_list-title-with-filter = Liste des groupes avec le filtre `{$filter}`
groups_list-none = Il n'y a pas de groupes sur ce serveur.
groups_list-none-with-filter = Il n'y a pas de groupes sur ce serveur avec le filtre `{$filter}`.

# Levels
levels = niveaux
levels_add = ajout
    .description = Ajoute un nouveau niveau au serveur
    .name = nom
    .name-description = Nom du nouveau niveau à ajouter
    .role = role
    .role-description = Un rôle à assigner au niveau, sinon un rôle est créé
levels_add-success = Le niveau `{$level}` à bien été crée.
levels_remove = supression
    .description = Enlève un niveau du serveur
    .name = nom
    .name-description = Nom du niveau à retirer
levels_list = liste
    .description = Liste les niveaux du serveur
    .filter = filtre
    .filter-description = Filtre les niveaux avec un nom
levels_list-title = Liste des niveaux
levels_list-title-with-filter = Liste des niveaux avec le filtre `{$filter}`
levels_list-none = Il n'y a pas de niveaux sur ce serveur.
levels_list-none-with-filter = Il n'y a pas de niveaux sur ce serveur avec le filtre `{$filter}`.

# Setup
setup = installation
setup_message = message
    .description = Met en place le message de connexion et de déconnexion.
setup_message-message = Connecte toi ou déconnecte toi.
setup_role = role
    .description = Met en place le rôle verifié.
    .role = role
    .role-description = Rôle verifié
setup_pattern = patterne
    .description = Met en place le patterne du nom de domaine autorisé.
    .pattern = patterne
    .pattern-description = Patterne du nom de domaine autorisé

# Information Context Menu
information = information
    .description = Donne des informations sur un membre verifié.
    .user = utilisateur
    .user-description = L'utilisateur dont vous voulez voir les informations.

# Dev
debug = debug
debug_force = force
debug_force_logout = connextion
    .description = Force un utilisateur ·à se connecter.
    .user = utilisateur
    .user-description = L'utilisateur à forcer à se connecter.
debug_force_logout-done = { $user } à été déconnecté.
debug_refresh = recharge
    .description = Recharge des éléments dans la base de données.
debug_refresh_member = membre
    .description = Charge un membre dans la base de données.
    .member = membre
    .member-description = Le membre à charger.
debug_refresh_member-already-in-database = { $user } est déjà dans la base de donées.
debug_refresh_member-added = { $user } à bien été ajouté.
debug_refresh_members = membres
    .description = Charge tout les membres dans la base de données.
debug_refresh_members-added = { $count } membres ont étés ajoutés à la base de donées.
debug_register = enregistrer
    .description = Enregistre les commandes slash.
    .register = enregistrer
    .register-description = Enregistrer ou Supprimer ?
    .global = global
    .global-description = Serveur ou Global ?

## Login

did-not-finish-auth-process = Vous n'avez pas fini le processus d'autentification en moins de 5min.
authentication-successful = Vous vous êtes correctement authentifié.
use-google-account-to-login = Utilisez votre compte Google pour vous connecter.

## Errors

error-bot-missing-permissions = Le bot requiert les permission suivantes : { $permissions }.
error-user-missing-permissions = Il vous manque les permission suivantes : { $permissions }.
error-user-missing-unknown-permissions = Il vous manque des permissions pour effectuer cette action.
error-cooldown = Vous pourrez réutiliser cette commande dans { $seconds } secondes.
error-not-an-owner = Vous devez être un administrateur pour effectuer cette action.
error-guild-only = Cette commande ne peut être utilisée que dans un serveur.
error-dm-only = Cette commande ne peut être utilisée que dans mes message privés.
error-internal-with-id = Une erreur interne est survenue. Si l'erreur persiste, merci de contacter un administrateur : `{ $id }`.
error-member-not-registered = L'utilisateur { $user } n'existe pas dans la base de données.
error-process-timed-out = Le processus a pris trop de temps.
error-user-timeout = Vous avez pris trop de temps.
error-member-not-verified = Le membre { $user } n'est pas vérifié.


## Events

# Setup
event-setup-login-button = Connexion
event-setup-logout-button = Déconnexion

# Login
event-login-select-level = Sélectionnez votre niveau
event-login-select-class = Sélectionnez votre classe
event-login-email-domain-not-allowed = Votre email n'est pas autorisé.

# Logout
event-logout-warning = Après vous être déconnecté, vous perdrez l'accès au serveur et devrez vous reconnecter.
event-logout-disconnect-button = Déconnecter son compte
event-logout-success = Votre compte a bien été déconnecté.
