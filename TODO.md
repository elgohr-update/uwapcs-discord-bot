# TODO

## Reaction Roles

_Involves: File r/w + parsing, discord reactions_

So, for reaction roles, afaict this is what needs to be done

-   Migrate config.rs to something like `config.toml`
-   Complete `reaction_roles.rs`
    -   Load from config (roles, and the rr msg, if they exist)
    -   Code up the `!rr init` command
    -   Monitor reactions, update user roles etc.
    -   On updated to `config.toml` (and on bot load, now that I think of it) overwrite the rr msg with content based on roles in config
    -   **Bonus:** add command like `!rr add :emoji: role name`

## LDAP Integration

_Involves: LDAP r/w, email,discord nicknames+roles_

-   Add a `discord_ID` or similar field to the LDAP database
-   Add a `!link <ucc-username>` command, which
    -   Created a verification token
        -   Perhaps just hash discordID+username with hardcoded salt (is this bad?)
    -   Sends an email with a verification token (and instruction to `!register`)
    -   Goes back to discord and run `!register <token>`
-   Add a `!register <token>` command
    -   Do hash, compare, if same update LDAP database
    -   Either add `registered` role or remove `unregistered`
        -   End result, unable to change nickname
    -   Set nickname to something like `<tla> (<first-name>)`
-   Add a `!unlink` command
    -   Remove LDAP entry
    -   Make able to change nickname again
-   **Bonus:** be able to specify nickname format for registered users in `config.toml`, and add way to update

## Fun with accounts

-   `!dispense <item name>`
    -   Dispense the item