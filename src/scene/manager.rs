use std::{boxed, collections::HashMap};

use crate::{
    core::error::{EngineError, EngineResult},
    input::InputState,
    scene::{Scene, SceneId},
};

pub struct SceneManager {
    scenes: HashMap<SceneId, Box<dyn Scene>>,
    current_scene_id: Option<SceneId>,
}

impl SceneManager {
    pub fn new() -> Self {
        SceneManager {
            scenes: HashMap::new(),
            current_scene_id: None,
        }
    }

    pub fn register_scene(&mut self, id: SceneId, scene: Box<dyn Scene>) {
        self.scenes.insert(id, scene);
    }

    pub fn set_current_scene(&mut self, id: SceneId) -> EngineResult<()> {
        if self.scenes.contains_key(&id) {
            self.current_scene_id = Some(id);
            Ok(())
        } else {
            Err(EngineError::SceneNotFound(format!(
                "SceneId: {:?} is not found",
                id
            )))
        }
    }

    pub fn get_current_scene(&self) -> Option<&Box<dyn Scene>> {
        if let Some(id) = self.current_scene_id {
            self.scenes.get(&id)
        } else {
            None
        }
    }

    pub fn get_current_scene_mut(&mut self) -> Option<&mut Box<dyn Scene>> {
        if let Some(id) = self.current_scene_id {
            self.scenes.get_mut(&id)
        } else {
            None
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        if let Some(scene) = self.get_current_scene_mut() {
            scene.update(dt, input);
        }
    }
}
