use scene::Scene;

pub trait Renderer<Input, Output> {
  fn render(&self, &mut Scene, Input) -> Output;
}
