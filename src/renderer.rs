use scene::Scene;

pub trait Renderer<'a> {
  type Output;

  fn render(&'a mut self, scene: &'a mut Scene) -> Self::Output;
}
