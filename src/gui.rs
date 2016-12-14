use std::marker::PhantomData;
use std::rc::Rc;

use color::Color;
use linear::Vector2;

pub trait Widget {
  fn get_rect(&self) -> Rect;
}

pub trait WidgetContainer: Widget {
  fn get_widgets(&self) -> &[Rc<Widget>];
  fn add_widget(&mut self, widget: Rc<Widget>);
}

type Px = u32;
type Pos = Vector2<Px>;

pub mod layout {
  pub struct Horizontal;
  pub struct Vertical;
  pub struct Floating;
}

// Upper-left is origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rect {
  lower: Pos,
  upper: Pos
}

impl Rect {
  pub fn new(upper_left: Pos, w: Px, h: Px) -> Self {
    Rect {
      lower: Pos::new(upper_left.x, upper_left.y + h),
      upper: Pos::new(upper_left.x + w, upper_left.y)
    }
  }
}

pub struct FillRectWidget<L> {
  color: Color,
  rect: Rect,
  widgets: Vec<Rc<Widget>>,
  _l: PhantomData<L>
}

impl<L> FillRectWidget<L> {
  pub fn new(rect: Rect, color: Color) -> Self {
    FillRectWidget {
      color: color,
      rect: rect,
      widgets: Vec::new(),
      _l: PhantomData
    }
  }
}

impl<L> Widget for FillRectWidget<L> {
  fn get_rect(&self) -> Rect {
    self.rect.clone()
  }
}

impl<L> WidgetContainer for FillRectWidget<L> {
  fn get_widgets(&self) -> &[Rc<Widget>] {
    &self.widgets
  }

  fn add_widget(&mut self, widget: Rc<Widget>) {
    self.widgets.push(widget);
  }
}

pub struct TopWidget<L> {
  rect: Rect,
  widgets: Vec<Rc<Widget>>,
  _l: PhantomData<L>
}

impl<L> TopWidget<L> {
  pub fn new(w: Px, h: Px) -> Self {
    TopWidget {
      rect: Rect::new(Pos::new(0, 0), w, h),
      widgets: Vec::new(),
      _l: PhantomData
    }
  }

}

impl<L> Widget for TopWidget<L> {
  fn get_rect(&self) -> Rect {
    self.rect.clone()
  }
}

impl<L> WidgetContainer for TopWidget<L> {
  fn get_widgets(&self) -> &[Rc<Widget>] {
    &self.widgets
  }

  fn add_widget(&mut self, widget: Rc<Widget>) {
    self.widgets.push(widget);
  }
}

fn test() {
  let mut top = TopWidget::<layout::Vertical>::new(100, 100);
  let fill = FillRectWidget::<layout::Floating>::new(Rect::new(Pos::new(0, 0), 10, 10), Color::new(1., 0., 0.));

  top.add_widget(Rc::new(fill));
}
