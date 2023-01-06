mod update;

use aglet::CoordVec;
use broccoli::{aabb::pin::AabbPin, Tree};
use itertools::Itertools;
use palkia::prelude::*;

use crate::{
    ecm::{
        self,
        component::{
            Collider, ColoredHitbox, HasDims, Mover, Positioned, Velocitized,
            ZLevel,
        },
        message::{MsgDraw, MsgPhysicsTick, MsgTick},
        resource::{Camera, HitboxTracker, ThePlayerEntity},
    },
    fabctx::FabCtx,
    geom::{EntityAABB, Hitbox},
    gfx::{GAME_HEIGHT, GAME_WIDTH},
    resources::Resources,
};

const PLAYER_WALK_SPEED: f32 = 128.0;
const PLAYER_ACC: f32 = 32.0;

pub struct StateGameplay {
    world: World,

    // TODO: make dt really work
    dt: f32,
    fab_ctx: FabCtx,
}

impl StateGameplay {
    pub fn new() -> StateGameplay {
        let mut world = World::new();
        ecm::setup_world(&mut world);

        let resources = Resources::get();
        let fabber = resources.fabber();

        let ctx = FabCtx {};

        let player = fabber
            .instantiate(
                "player",
                world.spawn().with(Positioned::new(CoordVec::new(0, 0))),
                &ctx,
            )
            .unwrap();
        world.insert_resource(ThePlayerEntity(player));

        world
            .spawn()
            .with(Positioned::new(CoordVec::new(0, 32)))
            .with(Mover::new())
            .with(HasDims::new(8 * 8, 4))
            .with(Velocitized::still())
            .with(ColoredHitbox::new(macroquad::prelude::PINK))
            .with(Collider)
            .build();
        world
            .spawn()
            .with(Positioned::new(CoordVec::new(32, 16)))
            .with(Mover::new())
            .with(HasDims::new(8 * 8, 4))
            .with(Velocitized::still())
            .with(ColoredHitbox::new(macroquad::prelude::PINK))
            .with(Collider)
            .build();

        StateGameplay {
            world,
            dt: 1.0 / 60.0,
            fab_ctx: ctx,
        }
    }

    pub fn on_update(&mut self) {
        update::move_player(self);

        update::do_collision(self);

        self.world.dispatch_to_all(MsgTick);
        self.world.dispatch_to_all(MsgPhysicsTick::new(self.dt));
        self.world.finalize();
    }

    pub fn on_draw(&self) {
        let mut hitboxeds = {
            let tracker = self.world.read_resource::<HitboxTracker>().unwrap();
            tracker
                .iter()
                .map(|e| {
                    let (pos, dims) =
                        self.world.query::<(&Positioned, &HasDims)>(e).unwrap();
                    EntityAABB::new(e, pos.make_hitbox(*dims))
                })
                .collect_vec()
        };
        let mut tree = Tree::new(hitboxeds.as_mut_slice());

        let camera_center =
            self.world.read_resource::<Camera>().unwrap().center;
        let view_rect = Hitbox::new(
            camera_center.x,
            camera_center.y,
            GAME_WIDTH as i32 + 32,
            GAME_HEIGHT as i32 + 32,
        );

        let mut es = Vec::new();
        // We don't care about the "entity" "doing" the rect scanning... but we need to give broccoli one, wah
        tree.find_all_intersect_rect(
            AabbPin::new(&mut EntityAABB::new(
                Entity::recompose(0, 0),
                view_rect,
            )),
            |_, profile| {
                let zlevel = self.world.query::<&ZLevel>(profile.e);
                es.push((profile.e, profile.rect, zlevel.map(|rqr| rqr.level)))
            },
        );
        es.sort_unstable_by(|a, b| {
            ZLevel::sort(a.2, b.2)
                // we want things "above" => less Y to be rendered first, so a cmp b
                .then(a.1.y.start.total_cmp(&b.1.y.start))
        });

        for (e, _, _) in es.iter() {
            self.world.dispatch(*e, MsgDraw::default());
        }
    }
}
