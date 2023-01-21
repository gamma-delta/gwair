use serde::{Deserialize, Serialize};

use std::f32::consts::TAU;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStats {
  pub walk_terminal_vel: f32,
  /// Can express these in terms of "seconds reqd to get to/from terminal vel."
  pub walk_accel: f32,
  /// "stop moving in 2 frames"
  pub walk_friction: f32,
  pub walk_turn_accel: f32,

  pub jump_height: f32,
  pub time_to_jump_apex: f32,
  /// Derived from kinematics
  pub jump_impulse_vel: f32,
  /// gravity when rising from a jump
  pub jump_gravity: f32,
  /// gravity when rising from a jump but not holding jump
  pub jump_release_gravity: f32,
  /// normal falling gravity
  pub falling_gravity: f32,
  /// Grace-period gravity when falling off a ledge
  pub coyote_gravity: f32,

  pub fall_terminal_vel: f32,
  pub plummet_terminal_vel: f32,

  pub coyote_time: f32,
  pub jump_buffer_len: f32,

  pub rod_anchor_dist: f32,
  pub vel_to_swing_vel_rate: f32,
  pub swing_gravity: f32,
  pub swing_friction: f32,
  pub swing_too_far_angle: f32,
  pub swing_too_far_gravity: f32,

  /// Amount the player's controls add to the swing
  pub player_swing_acc: f32,

  pub swing_terminal_vel: f32,
  pub swing_vel_to_vel_rate_x: f32,
  pub swing_vel_to_vel_rate_y: f32,
  /// If the player's start swing velocity is between these two, snap it up to
  /// the max one. That way, jumping straight up still has expected behavior,
  /// less feelsbad when you mess up a legitimate grab.
  pub start_grab_speed_cheat_min: f32,
  pub start_grab_speed_cheat_max: f32,

  /// If the angle is near horiz when releasing, cheat in favor of the player
  /// and make it a little smaller to make it easier to launch
  pub angle_to_cheat_launch_vel_at: f32,
  pub angle_launch_vel_cheat_factor: f32,

  /// To try and grab an extant rod, check in a set of ranges....
  pub grab_extant_step_size: f32,
  pub grab_extant_step_count: usize,
  /// The radius from the target swing position you can grab onto a rod.
  pub grab_extant_swingable_radius: i32,
  pub grab_extant_swingable_radius_increment: i32,
  pub grab_extant_start_size: i32,

  pub rod_deployments_from_ground: u32,

  pub debugdraw_grab_hbs: bool,
}

impl Default for PlayerStats {
  fn default() -> Self {
    let walk_terminal_vel = 12.0 * 8.0;
    let walk_accel = walk_terminal_vel / 0.3;
    let walk_friction = walk_terminal_vel * 60.0 / 2.0;
    let walk_turn_accel = walk_terminal_vel * 60.0;

    let jump_height = 40.0;
    let time_to_jump_apex = 0.45;
    let jump_impulse_vel = 2.0 * jump_height / time_to_jump_apex;
    let jump_gravity = jump_impulse_vel / time_to_jump_apex;
    let jump_release_gravity = jump_gravity * 3.0;
    let falling_gravity = jump_gravity * 2.5;
    let coyote_gravity = falling_gravity * 0.5;

    let fall_terminal_vel = 270.0;
    let plummet_terminal_vel = 400.0;

    let coyote_time = 0.05;
    let jump_buffer_len = 0.1;

    let rod_anchor_dist = 12.0;
    let vel_to_swing_vel_rate = 0.05;
    let swing_gravity = 5.0;
    let swing_friction = 0.05;
    let swing_too_far_angle = TAU / 4.0;
    let swing_too_far_gravity = 10.0;

    let player_swing_acc = 4.0;

    let swing_terminal_vel = 13.0;
    let swing_vel_to_vel_rate_y = 2.5;
    let swing_vel_to_vel_rate_x = swing_vel_to_vel_rate_y * 0.9;
    let start_grab_speed_cheat_min = 1.5;
    let start_grab_speed_cheat_max = 9.0;

    let angle_to_cheat_launch_vel_at = TAU * 0.225;
    let angle_launch_vel_cheat_factor = 2.0;

    let grab_extant_step_size = 8.0;
    let grab_extant_step_count = 4;
    let grab_extant_swingable_radius = 6;
    let grab_extant_swingable_radius_increment = 2;
    let grab_extant_start_size = 8;

    let rod_deployments_from_ground = 1;

    let debugdraw_grab_hbs = false;

    Self {
      walk_terminal_vel,
      walk_accel,
      walk_friction,
      walk_turn_accel,
      jump_height,
      time_to_jump_apex,
      jump_impulse_vel,
      jump_gravity,
      jump_release_gravity,
      falling_gravity,
      coyote_gravity,
      fall_terminal_vel,
      plummet_terminal_vel,
      coyote_time,
      jump_buffer_len,
      rod_anchor_dist,
      vel_to_swing_vel_rate,
      swing_gravity,
      swing_friction,
      swing_too_far_angle,
      swing_too_far_gravity,
      player_swing_acc,
      swing_terminal_vel,
      swing_vel_to_vel_rate_x,
      swing_vel_to_vel_rate_y,
      start_grab_speed_cheat_min,
      start_grab_speed_cheat_max,
      angle_to_cheat_launch_vel_at,
      angle_launch_vel_cheat_factor,
      grab_extant_step_size,
      grab_extant_step_count,
      grab_extant_swingable_radius,
      grab_extant_swingable_radius_increment,
      grab_extant_start_size,
      rod_deployments_from_ground,
      debugdraw_grab_hbs,
    }
  }
}
