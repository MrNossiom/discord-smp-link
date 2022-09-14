## General

done = Done!


## Commands

# Setup
setup = setup
    .description = A set of commands to setup the bot.

setup-message = message
    .description = Sets the login and logout message.
setup-message-message = Login or Logout?

setup-role = role
    .description = Setup the role to apply to verified members.
    .role = role
    .role-description = Which role to give

# Information Context Menu
information = information
    .description = Gives informations about a verified member.
    .user = user
    .user-description = The user to get informations about.

# Dev
debug = debug
    .description = Debug related commands.

debug-force = force
    .description = Force an action on an other user.

debug-force-logout = logout
    .description = Force disconnect a verified member.
    .user = user
    .user-description = The user to force disconnect.
debug-force-logout-done = { $user } has been unregistered.
debug-force-logout-not-verified = Member { $user } is not verified.

debug-refresh = refresh
    .description = Loads elements in the database.

debug-refresh-member = member
    .description = Refresh a member.
    .member = member
    .member-description = The member to refresh.
debug-refresh-member-already-in-database = { $user } is already in the database.
debug-refresh-member-added = { $user } successfully added.

debug-refresh-members = members
    .description = Loads every members in the database.
debug-refresh-members-added = { $count } members have been added to the database.

debug-register = register
    .description = Register slash commands to Discord.
    .register = register
    .register-description = Register or Unregister?
    .global = global
    .global-description = Guild or Global?


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
error-internal-with-id = An internal error has occurred. If the error persist, please contact an administrator: `{ $id }`.
error-member-not-registered = L'utilisateur { $user } n'existe pas dans la base de données.
error-process-timed-out = The process timed out.
error-user-timeout = You took too long to answer.

# Buttons
event-setup-login-button = Login
event-setup-logout-button = Logout

event-logout-warning = After you disconnected your accounts, you will lose access to the server and have to autenticate again.
event-logout-disconnect-button = Disconnect your account
event-logout-success = Your account has been disconnected.
