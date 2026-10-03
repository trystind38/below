# Below

by The Knights of the Round Table

## Game Description

Below is a 2D, run-and-gun Metroidvania game for PC where the player character must descend deeper into the Below, fighting enemies along the way, to reach an elevator to get back to the surface.

## Team Members

**Advanced Topic Subteam 1: Procedural Generation**
* Jake Biondolillo: jwb141@pitt.edu
* Finn Snyder: fis14@pitt.edu
* Aiden McCoy: agm121@pitt.edu
* Nick Cheddar: ndc45@pitt.edu

**Advanced Topic Subteam 2: Advanced AI**
* Caleb Sarmiento: cts58@pitt.edu
* Trystin DeRemer: tcd27@pitt.edu
* Nicholas Myers: nrm97@pitt.edu

## Advanced Topic Description

### Procedural Generation

Segments of levels between checkpoints will be procedurally generated. This includes the terrain, platforms, enemy locations, and lootable items. The terrain will be appropriate to navigate given our character's physics.

* Generate 12 "rooms" of random size (between 20 and 200 tiles in both dimensions) placed randomly in a 2D plane such that they do not overlap
* Compute a [Delaunay triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation) of the centers of all of those rooms
* Find a minimum spanning tree from the resulting graph
* From the set of edges from the triangulation that were not in the MST, add 20% back to the MST to form the basic connection of the dungeon
* Turn each edge of the resulting graph into a "hallway" (if the rooms overlap in x, draw a horizontal line between them, if they overlap in y, draw a vertical line, if they overlap in neither, create an L connector to join them)
  * Randomly generate 20 additional, smaller rooms (5-50 tiles per dimension) that overlap with the hallways to add additional exploration options
  * Randomly populate all rooms.
    * Platforms, obstacles, enemies, etc.
    * Different approaches will be used for bigger rooms and smaller hallway rooms.

### Advanced AI

Each enemy will have their own behavior according to what unique abilities (attacks) they have.
* Decision trees will model behavior state transitions.
* Gerald, Amoog, and Hive Mind enemies will have unique behavior tree models using decision trees.

Enemies are blind and rely on sound events created when the player walks, jumps, or shoots. Hearing a sound creates a “last heard location” at the source of that sound. Each enemy will have their own unique response, but for a general example:
* Enemies that are in a certain range will automatically know the location of the "last heard location" object and pursue it, given their decision tree allows it.
* If they reach the "last heard location" object and don't collide with the player, they will patrol in that area randomly for some interval.
  * If they collide with the player while traversing to the "last heard location" object, then they attack.

This would be what the Gerald enemy does, but for an enemy like the Amoog, they will not pursue the "last heard location" object and instead shoot in the direction of it. See the table below for futher details on enemies and their general behavior.

### Enemy Overview

| Name      | Health  | Behavior                                                                                                                                     | Ability                                                                                                           | Player Interaction      |
|-----------|---------|----------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|-------------------------|
| Gerald    | Low     | Gerald patrols and chases the player to attack them.                                                                                         | Deal damage by biting the player.                                                                                 | Stomp or shoot to kill. |
| Amoog     | Medium  | Amoog patrols and attacks by shooting at the player when in range. It will chase if the player exits the range while it is shooting at them. | Deal damage by shooting fireballs at the player.                                                                  | Shoot to kill.          |
| Fly Trap  | High    | Fly Trap is immobile. It shoots fireballs in the direction it's facing.                                                                      | Deal damage by shooting fireballs in one direction at a regular interval.                                         | Cannot kill.            |
| Hive Mind | Medium  | A swarm patrols around the hive mind. When the player is in range, the swarm chases and attacks the player.                                  | Deal accumulative damage when swarming the player. If the player kills the stationary hive, the swarm disappears. | Shoot to kill.          |
| Boss      | Highest | **_TBD_**                                                                                                                                    | **_TBD_**                                                                                                         |                         |

## Midterm Goals

* Develop the player's character to be moving and attacking with comfortable controls that are intuitive to learn and aren't difficult to remember. The player should be able to navigate the static environment at a fast pace, without being able to completely avoid enemies.
* Complete one enemy, with a simple state condition, along with the outlines for the rest of the enemies and how they will work.
  * Fully design decision trees for all three enemies and prepare to implement them.
* Set up checkpoints and statically-generated terrain. Begin developing infrastructure for procedural generation.

## Final Goals

### Main Game (80%)
* 30%: Procedural generation using Delaunay triangulation
  * 15%: Hallways generated using the graph edges from the Delaunay triangulation
    * Generate 12 rooms between 20 and 200 tiles in both directions.
    * Compute the Delaunay triangulation of the room centers, then find the minimum spanning tree of the result. Connect all nodes of the new graph.
    * Turn the edges of the graph into hallways, connecting them if they overlap in the x or y directions.
  * 10%: 20 rooms generated per level, each connected to the main hallways.
  * 5%: Each room randomly generated with lootables and enemies.
    * Lootables include light essence (to replenish health), and the weapons/power-ups described below.
    * Different protocols for generating loot in larger rooms vs. smaller rooms.
* 26%: Advanced AI implementation
  * 9%: Gerald
    * Decision tree changes based on sound propagation. Has a short line of sight but strong hearing. Will chase the player if it hears them, and will avoid attacking if the light level is too high (either from the player or another light source). It will be more inclined to attack the player when it perceives that their health is low (i.e., they have a low light level).
  * 8%: Amoog
    * Decision tree changes based on sight. Has a strong, far sight. Will try to keep its distance from the player, aiming its projectiles where it estimates the player will be (using the player's movement).
  * 9%: Hive Mind
    * Generates dozens of enemies at once, all with the same decision tree. Short line of sight and poor hearing, but once one is in sight of the player, all others in the room are notified. If many are dying, it will summon more from the central hive. The central hive has no decision tree but can be destroyed like a normal enemy, stopping the spawning of all enemies.
* 5%: Boss & intro level completed
* 5%: Player movement and animations
* 5%: Health and damage system
* 4%: Enemies
  * 2%: Fly Trap
  * 2%: Boss
* 5%: Power-ups
  * 2.5%: The unmatched power of the sun. Will expel a beam of light that kills every enemy on the screen (won't kill the boss, but will deplete at least a quarter of its health). Extremely rare, only found in small rooms.
  * 2.5%: The unmatched power of darkness. Will cast a shadow around the player, decreasing their visibility to enemies. Effective against every enemy except Gerald, who will be more inclined to attack the player.

### Stretch Goals (20%)
* 10%: Lootable Weapons (provide new abilities and cause enemies to react in different ways)
  * 5%: The Big One - has a further range and a stronger beam of light than the default gun. Will consume more light from the player.
  * 5%: The Little One - decreased damage and range, but will not deplete the player's light level.
* 10%: Optics System
  * Dynamically light the entire game, with light sources casting shadows. Make the light level more strongly affect enemies (i.e., enemies could be damaged by ambient lighting).
