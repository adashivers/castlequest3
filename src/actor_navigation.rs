use core::f32;
use std::{sync::Arc};

use crate::{debug::{DebugFlags}, loading_system::AssetsLoading};
use bevy::{prelude::*};
use bevy_landmass::{
  prelude::*,
  nav_mesh::bevy_mesh_to_landmass_nav_mesh,
  NavMeshHandle
};
