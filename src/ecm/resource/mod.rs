mod phys;
pub use phys::*;

use aglet::CoordVec;
use palkia::prelude::*;

use crate::fabctx::FabCtx;

/// Where the world is viewed from
#[derive(Debug)]
pub struct Camera {
    pub center: CoordVec,
}
impl Resource for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: CoordVec::new(0, 0),
        }
    }
}

pub struct FabCtxHolder(pub FabCtx);
impl Resource for FabCtxHolder {}
