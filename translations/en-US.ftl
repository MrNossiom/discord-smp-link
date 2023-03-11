## Common

done = Done!
continue = Continue
cancel = Cancel
and = and

## Commands

# Classes
classes = classes
classes_add = add
    .description = Add a new class to the guild
    .name = name
    .name-description = Class name to add
    .level = level
    .level-description = Parent level of the class
    .role = role
    .role-description = A role to assign to this class, if not provided a role will be created
classes_add-success = Class `{ $class }` has been created in the level `{ $level }`.
classes_add-no-such-level = There is no such level with the name of `{ $level }` on this guild.
classes_remove = remove
    .description = Remove a class from the guild
    .name = name
    .name-description = Class name to remove
classes_list = list
    .description = List the classes of the guild
    .filter = filter
    .filter-description = Filter the classes with a name
classes_list-title = List of classes
classes_list-title-with-filter = List of classes with the filter `{ $filter }`
classes_list-none = There is no classes in the guild.
classes_list-none-with-filter = There is no classes in this guild with the filter `{ $filter }`.
# Groups
groups = groups
groups_add = add
    .description = Add a new group to the guild
    .name = name
    .name-description = Group name to add
    .role = role
    .role-description = A role to assign to this group, if not provided a role will be created
    .emoji = emoji
    .emoji-description = An associated emoji to display in select menu
groups_add-success = Group `{ $group }` has been created.
groups_remove = remove
    .description = Remove a group from the guild
    .name = name
    .name-description = Group name to remove
groups_list = list
    .description = List the groups of the guild
    .filter = filter
    .filter-description = Filter the groups with a name
# Levels
levels = levels
levels_add = add
    .description = Add a new level to the guild
    .name = name
    .name-description = Level name to add
    .role = role
    .role-description = A role to assign to this level, if not provided a role will be created
levels_add-success = Level `{ $level }` has been created.
levels_remove = remove
    .description = Remove a level from the guild
    .name = name
    .name-description = Level name to remove
levels_list = list
    .description = List the levels of the guild
    .filter = filter
    .filter-description = Filter the levels with a name
# Setup
setup = setup
    .description = A set of commands to setup the bot.
setup_login_message = login
    .description = Sets the login and logout message.
setup_login_message-message = Login or Logout?
setup_groups_message = groups
    .description = Sets the groups selection message.
setup_groups_message-message = Select your groups here.
setup_groups_message-placeholder = Select a group...
setup_groups_message-not-enough-groups = You need at least one group to create the message.
setup_role = role
    .description = Setup the role to apply to verified members.
    .role = role
    .role-description = Which role to give
setup_pattern = pattern
    .description = Sets the pattern of the autohrized domain.
    .pattern = pattern
    .pattern-description = The pattern of the autohrized domain
# Information Context Menu
information = information
    .description = Gives informations about a verified member.
    .user = user
    .user-description = The user to get informations about.
# Dev
debug = debug
    .description = Debug related commands.
debug_force = force
    .description = Force an action on an other user.
debug_force_logout = logout
    .description = Force disconnect a verified member.
    .user = user
    .user-description = The user to force disconnect.
debug_force_logout-done = { $user } has been unregistered.
debug_refresh = refresh
    .description = Loads elements in the database.
debug_refresh_member = member
    .description = Refresh a member.
    .member = member
    .member-description = The member to refresh.
debug_refresh_member-already-in-database = { $user } is already in the database.
debug_refresh_member-added = { $user } successfully added.
debug_refresh_members = members
    .description = Loads every members in the database.
debug_refresh_members-added = { $count } members have been added to the database.
debug_register = register
    .description = Register slash commands to Discord.
    .register = register
    .register-description = Register or Unregister?
    .global = global
    .global-description = Guild or Global?

## Login

did-not-finish-auth-process = You didn't finish the authentication process under 5 minutes.
authentication-successful = You successfully authenticated with Google!
use-google-account-to-login = Use your Google account to connect yourself.

## Errors

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
error-member-not-verified = Member { $user } isn't verified.

## Events

# Setup
event-setup-login-button = Login
event-setup-logout-button = Logout
# Login
event-login-select-level = Select your level
event-login-select-class = Select your class
event-login-email-domain-not-allowed = Your email is not authorized.
# Logout
event-logout-warning = After you disconnected your accounts, you will lose access to the server and have to autenticate again.
event-logout-disconnect-button = Disconnect your account
event-logout-success = Your account has been disconnected.
