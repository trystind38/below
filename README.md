# Below

by The Knights of the Round Table

## Game Description

Below is a 2d, run-and-gun, Metroidvania game for PC where the player character must descend deeper into the Below, fighting enemies along the way, to reach an elevator to get back to the surface.

## Team Members

**Advanced Topic Subteam 1: Procedural Generation**
* Jake Biondolillo: jwb141@pitt.edu
* Finn Snyder:      fis14@pitt.edu
* Aiden McCoy:      agm121@pitt.edu
* Nick Cheddar:     ndc45@pitt.edu

**Advanced Topic Subteam 2: Advanced AI**
* Caleb Sarmiento: cts58@pitt.edu
* Trystin DeRemer: tcd27@pitt.edu
* Nicholas Myers:  nrm97@pitt.edu

## Advanced Procedural Generation

Segments of levels between checkpoints will be procedurally generated. This includes platforms, enemy locations, and collectable items.

Implementation details:
  * Generate 12 "rooms" of random size (between 20 and 200 tiles in both dimensions) placed randomly in a 2D plane such that they do not overlap
  * Compute a [Delaunay triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation) of the centers of all of those rooms
  * Find a minimum spanning tree from the resulting graph
  * From the set of edges from the triangulation that were not in the MST, add 20% back to the MST to form the basic connection of the dungeon
  * Turn each edge of the resulting graph into a "hallway" (if the rooms overlap in x, draw a horizontal line between them, if they overlap in y, draw a vertical line, if they overlap in neither, create an L connector to join them)
  * Randomly generate 20 additional, smaller rooms (5-50 tiles per dimension) that overlap with the hallways to add additional exploration options
  * Randomly populate all rooms.
    * Platforms, obstacles, enemies, etc.
    * Different approaches will be used for bigger rooms and smaller hallway rooms.

## Advanced AI

Each enemy will have their own behavior according to what unique abilities (attacks) they have.
* Decision trees will model behavior state transitions.
* Gerald, Amoog, and Hive Mind enemies will have unique behavior tree models using decision trees.

Enemies are blind and rely on sound events created when the player walks, jumps, or shoots. Hearing a sound creates a “last heard location” at the source of that sound. Each enemy will have their own unique response, but for a general example:
* Enemies that are in a certain range will automatically know the location of the "last heard location" object and pursue it, given their decision tree allows it.
* If they reach the "last heard location" object and don't collide with the player, they will patrol in that area randomly for some interval.
  * If they collide with the player while traversing to the "last heard location" object, then they attack.

This would be what the Gerald enemy does, but for an enemy like the Amoog, they will not pursue the "last heard location" object and instead shoot in the direction of it. See the table below for futher details on enemies and their general behavior.

## Enemy Overview

| Name      | Health  | Behavior                                                                                                                                     | Ability                                                                                                           | Player Interaction      |
|-----------|---------|----------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|-------------------------|
| Gerald    | Low     | Gerald patrols and chases the player to attack them.                                                                                         | Deal damage by biting the player.                                                                                 | Stomp or shoot to kill. |
| Amoog     | Medium  | Amoog patrols and attacks by shooting at the player when in range. It will chase if the player exits the range while it is shooting at them. | Deal damage by shooting fireballs at the player.                                                                  | Shoot to kill.          |
| Fly Trap  | High    | Fly Trap is immobile. It shoots fireballs in the direction it's facing.                                                                      | Deal damage by shooting fireballs in one direction at a regular interval.                                         | Cannot kill.            |
| Hive Mind | Medium  | A swarm patrols around the hive mind. When the player is in range, the swarm chases and attacks the player.                                  | Deal accumulative damage when swarming the player. If the player kills the stationary hive, the swarm disappears. | Shoot to kill.          |
| Boss      | Highest | **_TBD_**                                                                                                                                    | **_TBD_**                                                                                                         |                         |

## Midterm Goals

* Develop the player's character to be moving and attacking with controls that are intuitive to learn and aren't difficult to remember.
* Complete one enemy, with a simple state condition, along with the outlines for the rest of the enemies and how they will work.
* Set up checkpoints and statically-generated terrain. Begin developing infastructure for procedural generation.

## Final Goals

* 33%: Have four completed level (tutorial, two middle levels, final boss)
* 33%: Segments of all levels between checkpoints are able to be properly procedurally generated
* 33%: Have three unique enemies, with unique and advanced AI that provide a challenge

## Stretch Goals

* 1-2 different lootable weapons, which would provide new ablilites and cause the enemies to react in different ways.
* Optics system (dynamic lighting that casts shadows).
