## General

done = Fini!


## Commands

# Setup
setup = installation
    .description = Met en place le message de connexion et de deconnexion.
setup-message = Connecte toi ou déconnecte toi.

# Information Context Menu
information = information
    .description = Donne des informations sur un membre verifié.
    .user = Utilisateur
    .user-description = L'utilisateur dont vous voulez voir les informations.

# Dev
dev = dev
    .description = Un groupe de commandes pour les développeurs.

dev-force-login-already-verified = { $user } est déjà vérifié.
dev-force-login-added = { $user } à bien été ajouté à la base de données.
dev-force-login-no-member = Le membre { $user } n'existe pas.

dev-refresh-user-already-in-database = { $user } est déjà dans la base de donées.
dev-refresh-user-added = { $user } à bien été ajouté.


## Login

did-not-finish-auth-process = Vous n'avez pas fini le processus d'autentification en moins de 5min.
authentication-successful = Vous vous êtes correctement authentifié.
use-google-account-to-login = Utilisez votre compte Google pour vous connecter.


## Events

# Errors
error-bot-missing-permissions = Le bot requiert les permission suivantes : { $permissions }.
error-user-missing-permissions = Il vous manque les permission suivantes : { $permissions }.
error-user-missing-unknown-permissions = Il vous manque des permissions pour effectuer cette action.
error-cooldown = Vous pourrez réutiliser cette commande dans { $seconds } secondes.
error-not-an-owner = Vous devez être un administrateur pour effectuer cette action.
error-guild-only = Cette commande ne peut être utilisée que dans un serveur.
error-dm-only = Cette commande ne peut être utilisée que dans mes message privés.
error-internal-with-id = Une erreur interne est survenue. Si l'erreur persiste, merci de contacter un administrateur : `{ $id }`.

# Buttons
setup-button-login = Connexion
setup-button-logout = Déconnexion
logout-warning = Après vous être déconnecté, vous perdrez l'accès au serveur et devrez vous reconnecter.
