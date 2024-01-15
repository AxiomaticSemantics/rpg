# A prototype hack & slash RPG

An experimental RPG prototype built with [Bevy Engine][bevy].

_This project is in heavy development and targets a fork of Bevy with cherry-picked syncs of Bevy's main branch._
 
# Current status:
**(Currently not much is working in rpg_client as I'm in the process of comverting it to use server authoratative networking)**
* minimal working splash screen
* basic main menu
* basic in game HUD
* looping background and ambient audio tracks
* audio playback triggered by game events(attack, block, hit, item drop, item pickup)
* server side account and character creation
* data-driven generation of items, monsters etc.
* simple monster AI
* movement and combat

# Building
This game is meant to run on desktop systems only, web support will not be forthcoming.

The server parts are very young and require manual creation of a few directories

`mkdir -p /full/path/to/repo/save/server/accounts`

The following environment variables should currently be set:

`export BEVY_ASSET_ROOT=/full/path/to/repo`

`export RPG_SAVE_ROOT=/full/path/to/repo/save`

## rpg_client
- should build and work on most modern desktop systems with an Intel or AMD iGPU and most systems with a discrete graphics adapter.
## rpg_server
- this is meant to only ever be deployed on Linux/Unix like systems and is envisioned to be split into a suite of backend services.

# License

This project is dual licensed under either the [MIT](LICENSE-MIT) OR [APL](LICENSE-APL) license(s) with exceptions which are listed in the [Credits](credits/CREDITS.md).

[bevy]: https://bevyengine.org/
