use crate::secen::render_object::RenderObject;

pub mod render_object;

pub struct Secen {
    objects: Vec<RenderObject>,
}

impl Secen {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[RenderObject] {
        &self.objects
    }
}
