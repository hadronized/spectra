use scene::Scene;

pub trait Renderer<'a, 'b, 'c, Input, Output> {
  fn render(&'a self, &'b mut Scene<'c>, Input) -> Output;
}
