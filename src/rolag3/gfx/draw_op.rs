use crate::gfx::renderer::{Renderer, ColorRGBA32f, ViewSpaceCoordinate};

pub trait DrawOp {
    fn draw(&self, renderer: &mut dyn Renderer);
}

pub struct DrawOpWithMetadata {
   op: Box<dyn DrawOp>,
   z: f64, // ops with lower z are drawn first. Lower z is background, and higher z is foreground.
}

impl DrawOpWithMetadata {
    pub fn new(z: f64, op: Box<dyn DrawOp>) -> Self {
        Self {
            op,
            z,
        }
    }
}

pub struct DrawOpTriFan {
    color: ColorRGBA32f,
    vertexes: Vec<ViewSpaceCoordinate>,
}

impl DrawOp for DrawOpTriFan {
    fn draw(&self, renderer: &mut dyn Renderer) {
        renderer.draw_tri_fan(self.color, &self.vertexes);
    }
}

impl DrawOpTriFan {
    pub fn new(color: ColorRGBA32f, vertexes: Vec<ViewSpaceCoordinate>) -> Self {
        Self {
            color,
            vertexes
        }
    }
}

// performs multiple DrawOps that are guaranteed to happen consecutively in the same order they appear in the ops vec
pub struct DrawOpMulti {
    ops: Vec<Box<dyn DrawOp>>,
}

impl DrawOp for DrawOpMulti {
    fn draw(&self, renderer: &mut dyn Renderer) {
        for op in self.ops.iter() {
            op.draw(renderer);
        }
    }
}

impl DrawOpMulti {
    #[allow(dead_code)] // this function is not used yet
    pub fn new(ops: Vec<Box<dyn DrawOp>>) -> Self {
        Self { ops }
    }
}

pub fn process_draw_ops(renderer: &mut dyn Renderer,  mut ops_with_md: Vec<DrawOpWithMetadata>) {
    ops_with_md.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
    for op_with_md in ops_with_md.iter() {
        op_with_md.op.draw(renderer);
    }
}