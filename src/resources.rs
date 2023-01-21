use std::{
  fs, io,
  ops::Deref,
  path::{Path, PathBuf},
  sync::{Mutex, MutexGuard},
};

use ahash::AHashMap;
use macroquad::prelude as mq;
use smol_str::SmolStr;

use crate::{ecm, EntityFab};

pub struct Resources {
  textures: AHashMap<SmolStr, mq::Texture2D>,
  fallback_tex: mq::Texture2D,
  fabber: EntityFab,
}

#[cfg(debug_assertions)]
pub const RESOURCES_ROOT: &str =
  concat!(env!("CARGO_MANIFEST_DIR"), "/resources");
#[cfg(not(debug_assertions))]
pub const RESOURCES_ROOT: &str = "./resources";

impl Resources {
  pub fn load() -> eyre::Result<Resources> {
    let tex_root = [RESOURCES_ROOT, "textures"]
      .into_iter()
      .collect::<PathBuf>();
    let mut textures = AHashMap::new();
    for path in all_subpaths(&tex_root, "png")? {
      let abs_path = tex_root.join(&path);
      let file = fs::read(&abs_path)?;
      // MQ just panics here, but like ... yeah fine
      let tex =
        mq::Texture2D::from_file_with_format(&file, Some(mq::ImageFormat::Png));

      let stem: SmolStr = path
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/")
        .into();
      textures.insert(stem, tex);
    }
    let fallback_tex = {
      let img = mq::Image::gen_image_color(
        16,
        16,
        mq::Color::from_rgba(255, 0, 255, 255),
      );
      mq::Texture2D::from_image(&img)
    };

    let bp_root = [RESOURCES_ROOT, "blueprints"]
      .into_iter()
      .collect::<PathBuf>();
    let mut fabber = EntityFab::new();
    ecm::setup_fabber(&mut fabber);
    for path in all_subpaths(&bp_root, "kdl")? {
      let abs_path = bp_root.join(&path);
      // println!("{:?}", &abs_path);
      let file = fs::read_to_string(&abs_path)?;
      fabber.load_str(&file, &path.display().to_string())?;
    }

    Ok(Resources {
      textures,
      fallback_tex,
      fabber,
    })
  }

  pub fn get() -> ResourcesRef {
    let lock = THE_RESOURCES
      .try_lock()
      .expect("assets were mutably borrowed somehow");
    ResourcesRef(lock)
  }

  pub fn swap(new: Resources) {
    let mut lock = THE_RESOURCES
      .try_lock()
      .expect("assets were mutably borrowed somehow");
    *lock = Some(new);
  }

  pub fn fabber(&self) -> &EntityFab {
    &self.fabber
  }

  pub fn get_texture(&self, path: &str) -> mq::Texture2D {
    self
      .textures
      .get(path)
      .copied()
      .unwrap_or(self.fallback_tex)
  }
}

impl Drop for Resources {
  fn drop(&mut self) {
    for tex in self.textures.values() {
      tex.delete();
    }
    self.fallback_tex.delete();
  }
}

static THE_RESOURCES: Mutex<Option<Resources>> = Mutex::new(None);

pub struct ResourcesRef(MutexGuard<'static, Option<Resources>>);

impl Deref for ResourcesRef {
  type Target = Resources;

  fn deref(&self) -> &Self::Target {
    self.0.as_ref().expect("assets must be filled")
  }
}

/// Return all files with the given extension under the given path, as relative paths.
fn all_subpaths(root: impl AsRef<Path>, ext: &str) -> io::Result<Vec<PathBuf>> {
  if !root.as_ref().is_dir() {
    return Err(io::Error::new(
      io::ErrorKind::InvalidInput,
      format!("{} is not a directory", root.as_ref().display()),
    ));
  }
  let root = root.as_ref().canonicalize().unwrap();
  let mut todo = vec![root.clone()];
  let mut out = Vec::new();

  while let Some(path) = todo.pop() {
    if path.is_dir() {
      for entry in fs::read_dir(&path)? {
        let entry = entry?;
        todo.push(path.join(entry.path()));
      }
    } else {
      let ext_matches = match path.extension() {
        None => false,
        Some(it) => it == ext,
      };
      if ext_matches {
        out.push(path.strip_prefix(&root).unwrap().to_path_buf());
      }
    }
  }

  Ok(out)
}
