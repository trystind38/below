# Below

by The Knights of the Round Table

## Team Members
* Advanced Topic Subteam 1: Procedural Generation
	* jwb141@pitt.edu: Jake Biondolillo
	* fis14@pitt.edu: Finn Snyder
	* agm121@pitt.edu: Aiden McCoy
	* ndc45@pitt.edu: Nick Cheddar

* Advanced Topic Subteam 2: Advanced AI
	* cts58@pitt.edu: Caleb Sarmiento
	* tcd27@pitt.edu: Trystin DeRemer
	* nrm97@pitt.edu : Nicholas Myers

## Game Description

Below is a 2d, run-and-gun, Metroidvania game for PC where the player character has to descend deeper into the Below to reach an elevator to get back to the surface.

## Advanced Topic Description

### Procedural Generation

Segments of levels between checkpoints will be procedurally generated. This includes the terrain, enemy locations, and lootable items. The terrain will be appropriate to navigate given our character's physics.

Implementation details:
* We will use randomized depth-first search with backtracking to generate paths between rooms in each section between checkpoints.
* Platforms and gaps within each room will also be procedurally generated, using player movement limits to constrain placement.
* When a layout cannot connect the required entrances and exits, the generator will backtrack and try another arrangement.
* Once a valid main route exists, we will add optional branches, enemies, and resources for variety.
* Each completed section will be checked for traversability to ensure the player can reach the next checkpoint.
    
### Advanced AI

- **Behavior states:** A unique finite state machine will be used for each enemy type, defining how it transitions between states. Enemies will generally have four states: Idle, Patrol, Chase, and Attack.
- **Sound detection:** Enemies are blind and rely on sound events created when the player walks, jumps, or shoots. Hearing a sound creates a “last heard location” at the source of that sound.
  - **Proposed sound falloff:** Dijkstra’s algorithm will propagate sound through traversable space. Sound intensity will decrease with distance traveled, and enemies will react if the sound is loud enough for them to hear.
- **Chase state:** Triggered by hearing a sound. Enemies will pathfind to the last heard location whenever it updates. A* pathfinding will be adapted for a platforming environment where jumping or falling alters the cost.
- **Patrol state:** Triggered when an enemy reaches the last heard location. The enemy will wander randomly around that spot.
- **Attack state:** Melee enemies will attack when they collide with the player or are within close range. For ranged enemies, we propose triggering attacks within firing range and aiming toward the last heard location.
- **Enemy types:**
  - **Gerald:** Walks around and goes after the player to bite them within close range. The player can stomp on or shoot Gerald.
  - **Amoog:** Loops between trying to shoot at the last heard location and chasing it.
  - **Fly Trap:** Does not move and shoots fireballs in a direction determined by its placement. It alternates between Idle and Attack at regular intervals, with no pathfinding. The player cannot kill it.
  - **Hive Mind:** A swarm attacks the player within a certain range, dealing small amounts of damage at regular intervals through contact. If the stationary hive is destroyed, the swarm disappears.
  - **Boss:** Walks around near the player before striking with a melee attack that has a wind-up. The player can shoot the Boss.

## Midterm Goals

* Develop the player's character to be moving and attacking with comfortable controls.
* Complete one enemy, with a simple state condition, along with the outlines for the rest of the enemies and how they will work.
* Set up checkpoints and statically-generated terrain. Begin developing infastructure for procedural generation.

## Final Goals

* 33%: Have four completed level (tutorial, two middle levels, final boss)
* 33%: Segments of all levels between checkpoints are able to be properly procedurally generated
* 33%: Have three unique enemies, with unique and advanced AI that provide a challenge

## Stretch Goals

* 1-2 different lootable weapons, which would provide new ablilites and cause the enemies to react in different ways.
* Optics system (dynamic lighting that casts shadows).
