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
