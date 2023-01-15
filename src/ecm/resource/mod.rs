mod camera;
mod phys;

pub use camera::*;
pub use phys::*;

use palkia::prelude::*;

use crate::fabctx::FabCtx;

pub struct FabCtxHolder(pub FabCtx);
impl Resource for FabCtxHolder {}

/// Holder for the player!
pub struct ThePlayerEntity(pub Entity);
impl Resource for ThePlayerEntity {}
