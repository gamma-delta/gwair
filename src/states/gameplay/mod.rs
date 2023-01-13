mod update;

use aglet::CoordVec;
use broccoli::{aabb::pin::AabbPin, Tree};
use itertools::Itertools;
use macroquad::prelude::Color;
use palkia::prelude::*;

use crate::{
    ecm::{
        self,
        component::{
            Collider, ColoredHitbox, HasDims, Mover, Positioned, ZLevel,
        },
        message::{MsgDraw, MsgPhysicsTick, MsgTick},
        resource::{Camera, FabCtxHolder, HitboxTracker},
    },
    fabctx::FabCtx,
    geom::{EntityAABB, Hitbox},
    gfx::{GAME_HEIGHT, GAME_WIDTH},
    resources::Resources,
};

pub struct StateGameplay {
    world: World,

    // TODO: make dt really work
    dt: f32,
}

impl StateGameplay {
    pub fn new() -> StateGameplay {
        let mut world = World::new();
        ecm::setup_world(&mut world);

        let resources = Resources::get();
        let fabber = resources.fabber();

        let ctx = FabCtx {};

        fabber
            .instantiate(
                "player",
                world.spawn().with(Positioned::new(CoordVec::new(0, 0))),
                &ctx,
            )
            .unwrap();

        world.insert_resource(FabCtxHolder(ctx));
        setup_temp_level(&mut world);

        StateGameplay {
            world,
            dt: 1.0 / 60.0,
        }
    }

    pub fn on_update(&mut self) {
        self.world.dispatch_to_all(MsgPhysicsTick::new(self.dt));
        update::do_collision(self);

        self.world.dispatch_to_all(MsgTick);
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

fn setup_temp_level(world: &mut World) {
    const TEMP_LEVEL: &str = r"
XX
XX
    
    
   XX           XXX
   XX           XXX
                           XXXX
                           
       XXXX    
       XXXX          
                                  XXX
XXX                               XXX
 

XXXXXXXXXXXXX              XXXXX
XXXXXXXXXXXXX              XXXXX
        
        
                      XXX
        
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
        ";
    for (y, line) in TEMP_LEVEL.lines().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch != ' ' {
                let wx = (x as i32) * 8 - 140;
                let wy = (y as i32) * 8 - 100;
                world
                    .spawn()
                    .with(Positioned::new(CoordVec::new(wx, wy)))
                    .with(Mover::new())
                    .with(HasDims::new(8, 8))
                    .with(ColoredHitbox::new(Color::new(
                        (x as f32 + y as f32).sin() * 0.5 + 0.5,
                        0.0,
                        1.0,
                        1.0,
                    )))
                    .with(Collider)
                    .build();
            }
        }
    }
}
