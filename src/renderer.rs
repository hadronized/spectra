use scene::Scene;

pub trait Renderer<Input, Output> {
  fn render(&self, Input) -> Output;
}
