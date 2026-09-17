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
* First, we will design and create handcrafted room templates. 
* Then, we will use a randomized backtracking algorithm to generate levels between checkpoints
* The algorithm will attempt to build a main route, backtracking and pruning branches when a room placement prevents a valid path. 
* Eventually, we will add optional branches, enemies, and resources for variety. 
* Finally, each section will be checked to ensure that the path is actually traversable and that players can reach the next checkpoint.
    
### Advanced AI

Each of our enemies will have an advanced, unique AI that will determine how they interact with the player. Each enemy will have a unique patrol, attack, and chase pathfinding abilities, as well as being able to make descisions to change their state based on their environemt. they will also be able to pathfind around obstacles.

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
