use palkia::prelude::*;

use super::component::Collider;

pub fn collides_with<A: AccessQuery>(
    access: &A,
    src: Entity,
    dst: Entity,
) -> bool {
    if src == dst {
        return false;
    }

    access.query::<&Collider>(dst).is_some()
}
