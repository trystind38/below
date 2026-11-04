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
pub struct Invulnerable; // damage systems skip entities with this

#[derive(Component)]
pub struct Frozen;

#[derive(Component)]
pub struct MoveSpeed(pub f32);

#[derive(Component)]
pub struct Enemy; // tag: marks an entity as an enemy

#[derive(Component)]
pub struct Hitbox(pub Vec2); // full width/height in world units

#[derive(Component, Default)]
pub struct HitCooldown(pub f32); // seconds of invincibility left

#[derive(Component)]
pub struct Stompable;   // differentiates stompable enemies (Gerald) from the others

pub const HIT_COOLDOWN: f32 = 1.0;//1 second of invincibility after getting hit
pub const ENEMY_HIT_COOLDOWN: f32 = 0.5;

// Starting values for each character, kept in one place so
// nobody hardcodes numbers elsewhere. These are placeholders,
// so change them freely during playtesting.
pub const PLAYER_HEALTH: f32 = 100.0;
pub const PLAYER_DAMAGE: f32 = 10.0;
pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_STOMP_DAMAGE: f32 = 30.0;

pub const GERALD_HEALTH: f32 = 30.0;
pub const GERALD_DAMAGE: f32 = 5.0;
pub const GERALD_SPEED: f32 = 40.0;
pub const GERALD_SIGHT_RANGE: f32 = 200.0; // placeholder

// Stats for enemies, so we can easily tweak them in one place.
pub struct EnemyStats {
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
}
impl EnemyStats {
    pub fn bundle(&self) -> (Enemy, Health, Damage, MoveSpeed) {
        (
            Enemy,
            Health::new(self.health),
            Damage(self.damage),
            MoveSpeed(self.speed),
        )
    }
}

pub const HIVEMIND: EnemyStats = EnemyStats {
    health: 10.0,
    damage: 5.0,
    speed: 60.0,
};
//constructor for gerald stats
pub const GERALD: EnemyStats = EnemyStats {
    health: GERALD_HEALTH,
    damage: GERALD_DAMAGE,
    speed: GERALD_SPEED,
};

pub const STRAIGHT_TEETH: EnemyStats = EnemyStats {
    health: f32::INFINITY,
    damage: 20.0,
    speed: 0.0,
};
//straight teeth is invulnerable but it has a hitbox
//because it can still be hit by the player, but it won't take damage unless hit by a powerup.
//commands.spawn((STRAIGHT_TEETH.bundle(), Invulnerable, Hitbox(Vec2::new(32., 32.))));
