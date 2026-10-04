use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: f32,// health right now, goes down when hit
    pub max: f32,// maximum health, used to reset health when player respawns
}

impl Health {
    pub fn new(max: f32) -> Self {
        // Shortcut for creating Health at full.
        // Health::new(100.0) gives current = 100, max = 100.
        Self { current: max, max }
    }
}

#[derive(Component)]
pub struct Damage(pub f32);// How much damage this entity deals when it hits something.

#[derive(Component)]
pub struct MoveSpeed(pub f32);

// Starting values for each character, kept in one place so
// nobody hardcodes numbers elsewhere. These are placeholders,
// so change them freely during playtesting.
pub const PLAYER_HEALTH: f32 = 100.0;
pub const PLAYER_DAMAGE: f32 = 10.0;
pub const PLAYER_SPEED: f32 = 300.0;

pub const GERALD_HEALTH: f32 = 10.0;
pub const GERALD_DAMAGE: f32 = 5.0;
pub const GERALD_SPEED: f32 = 100.0;