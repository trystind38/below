use bevy::prelude::*;
use std::collections::HashMap;

// TODO: make one single node/point type for procgen and a-star to share

pub struct Node {
  pub id: IVec2, // IVec2 for a 2-d integer coordinates representing tile locations
}

impl Node {
  pub fn new(x: i32, y: i32) -> Self {
    Self(IVec2::new(x,y))
  }
}

