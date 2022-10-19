## Common

done = Fini!
continue = Continuer
cancel = Annuler
and = et

## Commands

# Class

class = class

class-add = add

class-remove = remove

class-list = liste
    .description = Liste les classes de ce serveur
    .filter = filtre
    .filter-description = Filtre les classes avec un nom
class-list-title = Liste des classes
class-list-title-with-filter = Liste des classes avec le filtre `{$filter}`
class-list-none = Il n'y a pas de classes sur ce serveur.
class-list-none-with-filter = Il n'y a pas de classes sur ce serveur avec le filtre `{$filter}`.


# Setup
setup = installation

setup-message = message
    .description = Met en place le message de connexion et de déconnexion.
setup-message-message = Connecte toi ou déconnecte toi.

setup-role = role
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

debug-force = force

debug-force-logout = connextion
    .description = Force un utilisateur ·à se connecter.
    .user = utilisateur
    .user-description = L'utilisateur à forcer à se connecter.
debug-force-logout-done = { $user } à été déconnecté.

debug-refresh = recharge
    .description = Recharge des éléments dans la base de données.

debug-refresh-member = membre
    .description = Charge un membre dans la base de données.
    .member = membre
    .member-description = Le membre à charger.
debug-refresh-member-already-in-database = { $user } est déjà dans la base de donées.
debug-refresh-member-added = { $user } à bien été ajouté.

debug-refresh-members = membres
    .description = Charge tout les membres dans la base de données.
debug-refresh-members-added = { $count } membres ont étés ajoutés à la base de donées.

debug-register = enregistrer
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
