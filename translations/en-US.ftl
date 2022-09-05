## General

done = Done!


## Commands

# Setup
setup = setup
    .description = Sets up the login and logout message.
setup-message = Login or Logout

# Information Context Menu
information = information
    .description = Gives informations about a verified member.
    .user = User
    .user-description = The user to get informations about.

# Dev
dev = dev
    .description = A set of commands for developers.

dev-force-login-already-verified = { $user } is already verified.
dev-force-login-added = { $user } has been added to the database.
dev-force-login-no-member = Member { $user } does not exist.

dev-refresh-user-already-in-database = { $user } is already in the database.
dev-refresh-user-added = { $user } successfully added.


## Login

did-not-finish-auth-process = You didn't finish the authentication process under 5 minutes.
authentication-successful = You successfully authenticated with Google!
use-google-account-to-login = Use your Google account to connect yourself.

## Events

# Errors
error-bot-missing-permissions = The bot is missing the following permissions: { $permissions }.
error-user-missing-permissions = You are missing the following permissions: { $permissions }.
error-user-missing-unknown-permissions = I don't have the permissions to send messages in this channel.
error-cooldown = You will be able to use this command again in { $seconds } seconds.
error-not-an-owner = You must be the owner of this bot.
error-guild-only = This command can only be used in a guild channel.
error-dm-only = This command can only be used in a DM channel.
error-internal-with-id = An internal error has occurred. If the error persist, please contact an administrator : `{ $id }`.

# Buttons
setup-button-login = Login
setup-button-logout = Logout
logout-warning = After you disconnected your accounts, you will lose access to the server and have to autenticate again.
