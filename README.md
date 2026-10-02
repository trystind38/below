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

We will generate a dozen large rooms between 20 and 200 tiles long/tall, and use Delaunay Triangulation to find the center of each room. Then, after finding the minumum spanning tree of each graph, turn each edge into a hallway, connecting all of the rooms. From here, we will generate an additonal 20 rooms, between 5-50 tiles in dimention, and decorate them with loot and enemies. certain weapons and power-ups will only generate in rooms of certain sizes
    
### Advanced AI

Each of our enemies will have an advanced, unique AI that will determine how they interact with the player. Each enemy will have a unique patrol, attack, and chase pathfinding abilities

Three of our enemies-- Amoog, Gerald, and Hive Mind-- will have unique decision trees, and unique methods for pathfinding and attacking the player. A full description of each can be found in the final rubric section.

## Midterm Goals

* Develop the player's character to be moving and attacking with comfortable controls. The player should be able to navigate the static environment at a fast pace, without being able to completely avoid enemies.
* Complete one enemy, with a simple state condition, along with the outlines for the rest of the enemies and how they will work.
	* fully design decision trees for all three enemies, prepare to implement
* Set up checkpoints and statically-generated terrain. Begin developing infastructure for procedural generation.

## Final Goals

### Main Game (80%):
* 30%: Procedural generation using Delaunay triangulation
	* 15%: Hallways generated, using the graph edges gained from Delaunay Triagnulation. 
		* Generate 12 rooms between 20 and 200 tiles in both directions
		* Compute Delaunay triangulation to find the center of each room, then find the minimum spanning tree for the result. Connect all nodes of the new graph
		* Turn the edges of the graph into hallways, connecting them if they overlap in the x or y directions.
	* 10%: 20 rooms generated per level, each connected to the main hallways. 
	* 5%: Each room randomly generated with lootables and enemies.
		* Lootables include light essence (to replenish health), and weapons/power-ups described in the stretch goals
		* Different protocols for generating loot in larger rooms vs. smaller rooms.
* 26%: Advanced AI implementation
	* 9%: Gerald
		* Decision tree changes based on sound propagation. Has a short line of sight but strong hearing. Will chase the player if it hears it, will avoid attacking if light level is too high (either with the player or another light source). It will be more inclined to attack the player when it perceives that their health is low (i.e.: they have a low light level)
	* 8%: Amoog
		* Decision tree changes based on sight. Has a strong, far sight. Will try to keep its distance from the player, aiming its projectiles where it estimates the player would be (using player’s movement)
	* 9%: Hive Mind
		* Generates dozens of enemies at once, all have the same decision tree. Short line of sight, and poor hearing, but once one is in sight of the player, all others in the room are notified. If many are dying, will summon more from the central hive. Central hive has no decision tree but can be destroyed like a normal enemy, stopping the spawning of all enemies.
* 5%: Boss & intro level completed
* 5%: Player movement and animations 
* 5%: Health and damage system
* 4%: Enemies
	* 2%: Fly Trap
	* %: Boss
* 5%: Power-ups
	* 2.5%: The unmatched power of the sun. Will expel a beam of light that kills every enemy on the screen (won’t kill the boss, but will deplete at least a quarter of its health). Extremely rare, only found in small rooms.
	* 2.5%: The unmatched power of darkness. Will cast a shadow around the player, decreasing their visibility to enemies. Effective against every enemy except Gerald, who will be more inclined to attack the player.

### Stretch Goals (20%):
* 10%: Lootable Weapons
	* 5%: The Big One - has a further range and a stronger beam of light than the default gun. Will consume more light from the player.
	* 5%: The Little One - decreased damage and range, but will not deplete player’s light level.
* 10%: Optics System
	* Dynamically light the entire game. make the light level more strongly effect enemies (i.e. enemies could be damaged by ambient lighting)
