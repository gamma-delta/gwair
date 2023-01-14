use palkia::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for things that you can swing on.
#[derive(Debug, Serialize, Deserialize)]
pub struct SwingableOn;
impl Component for SwingableOn {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
    }
}

/// Marker component for things that are the rod and can be picked up.
#[derive(Debug, Serialize, Deserialize)]
pub struct PickuppableRod;
impl Component for PickuppableRod {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
    }
}
