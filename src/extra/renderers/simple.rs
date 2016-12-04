use luminance::{Depth32F, Dim2, Flat, RGBA32F};
use luminance_gl::gl33::{Framebuffer, Pipe, Pipeline, RenderCommand, ShadingCommand, Texture};

use camera::{Camera, Freefly};
use extra::shaders::default::{DEFAULT_3D_INST, DEFAULT_3D_PROJ, DEFAULT_3D_VIEW, DefaultProgram3D};
use object::Object;
use projection::Projectable;
use scene::Scene;
use transform::Transformable;

pub type Texture2D<A> = Texture<Flat, Dim2, A>;

/// Simple renderer that takes a camera, a set of model and applies a shader on them. This renderer
/// outputs a single color map along with the depth map.
pub struct SimpleRenderer<'a> {
  program: DefaultProgram3D<'a>,
  framebuffer: Framebuffer<Flat, Dim2, Texture2D<RGBA32F>, Texture2D<Depth32F>>
}

impl<'a> SimpleRenderer<'a> {
  pub fn new_from(w: u32, h: u32, scene: &mut Scene<'a>) -> Self {
    SimpleRenderer {
      program: DefaultProgram3D::new_from(scene).unwrap(),
      framebuffer: Framebuffer::new((w, h), 0).unwrap()
    }
  }

  pub fn render(&mut self, scene: &mut Scene<'a>, camera: &Camera<Freefly>, objects: &[&Object<'a>]) -> (&Texture2D<RGBA32F>, &Texture2D<Depth32F>) {
    let program = scene.get_by_id(&self.program).unwrap();

    // reify objects
    let objects: Vec<_> = objects.iter().map(|object| {
      (object, scene.get_by_id(&object.model).unwrap())
    }).collect();

    let tessellations = objects.iter().flat_map(|&(object, ref model)| {
      model.parts.iter().map(move |part| {
        Pipe::new(move |program| {
                    program.update(&DEFAULT_3D_INST, *object.transform().as_ref());
                  },
                  &part.tess)
      })
    }).collect();

    Pipeline::new(&self.framebuffer, [0., 0., 0., 0.], &[], &[], vec![
      Pipe::new(|program| {
                  // update the camera
                  program.update(&DEFAULT_3D_PROJ, *camera.projection().as_ref());
                  program.update(&DEFAULT_3D_VIEW, *camera.transform().as_ref());
                },
                ShadingCommand::new(&program, vec![
                  Pipe::new(|_| {}, RenderCommand::new(None, true, tessellations, 1, None))
                ]))
    ]).run();

    (&self.framebuffer.color_slot, &self.framebuffer.depth_slot)
  }
}
