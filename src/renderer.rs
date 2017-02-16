use scene::Scene;

pub trait Renderer<'a, Input, Output> {
  fn render(&'a self, Input) -> Output;
}
