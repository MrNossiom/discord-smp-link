## Common

done = Fini!
continue = Continuer
cancel = Annuler
and = et

## Commands

# Classes

classes = classes

classes_add = ajoute
    .description = Ajoute une nouvelle classe au serveur
    .class_name = nom
    .class_name-description = Nom de la nouvelle classe à ajouter
    .maybe_role = role
    .maybe_role-description = Un rôle à assigner à la classe, sinon un rôle est créé

classes_remove = retire
    .description = Enlève une classe du serveur
    .class_name = nom
    .class_name-description = Nom de la classe à retirer

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

groups_add = ajoute
    .description = Ajoute un nouveau groupe au serveur
    .group_name = nom
    .group_name-description = Nom du nouveau groupe à ajouter
    .maybe_role = role
    .maybe_role-description = Un rôle à assigner au groupe, sinon un rôle est créé

groups_remove = retire
    .description = Enlève un groupe du serveur
    .group_name = nom
    .group_name-description = Nom du groupe à retirer

groups_list = liste
    .description = Liste les groupes du serveur
    .filter = filtre
    .filter-description = Filtre les groupes avec un nom

# Setup
setup = installation

setup_message = message
    .description = Met en place le message de connexion et de déconnexion.
setup_message-message = Connecte toi ou déconnecte toi.

setup_role = role
    .description = Met en place le rôle verifié.
    .role = role
    .role-description = Rôle verifié

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

# Buttons
event-setup-login-button = Connexion
event-setup-logout-button = Déconnexion

event-logout-warning = Après vous être déconnecté, vous perdrez l'accès au serveur et devrez vous reconnecter.
event-logout-disconnect-button = Déconnecter son compte
event-logout-success = Votre compte a bien été déconnecté.
