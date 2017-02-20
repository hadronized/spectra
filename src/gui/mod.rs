use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use color::ColorAlpha;
use overlay::{Disc, Quad, Renderer, RenderInput, Text, Texture2D, Triangle, Vert};
use scene::Scene;

type Time = f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32
}

pub struct GUI<'a> {
  // common
  renderer: Renderer,

  widgets: HashMap<String, Rc<RefCell<Widget<'a>>>>,
}

impl<'a> GUI<'a> {
  pub fn new(viewport: Viewport, scene: &mut Scene) -> Self {
    GUI {
      renderer: Renderer::new(viewport.w.ceil() as u32, viewport.h.ceil() as u32, 1024, 1024, 1024, scene),
      widgets: HashMap::new()
    }
  }

  pub fn add_widget(&mut self, id: &str, widget: &Rc<RefCell<Widget<'a>>>) {
    self.widgets.insert(id.to_owned(), widget.clone());
  }

  pub fn remove_widget(&mut self, id: &str) -> Option<Rc<RefCell<Widget<'a>>>> {
    self.widgets.remove(id)
  }

  //pub fn new_timeline(&mut self, h: f32, progress_color: ColorAlpha, inactive_color: ColorAlpha, dur_sec: f32) -> Rc<RefCell<Timeline>> {
  //  let timeline = Rc::new(RefCell::new(Timeline::new(self.viewport, h, progress_color, inactive_color, dur_sec)));

  //  self.timeline = Some(timeline.clone());
  //  timeline
  //}

  pub fn render(&self) -> &Texture2D {
    let mut tris = Vec::new();
    let mut quads = Vec::new();
    let mut discs = Vec::new();
    let mut texts = Vec::new();

    for widget in self.widgets.values() {
      for prim in &widget.borrow().unwidget() {
        match prim {
          &WidgetPrim::Triangle(ref tri) => tris.push(*tri),
          &WidgetPrim::Quad(ref quad) => quads.push(*quad),
          &WidgetPrim::Disc(ref disc) => discs.push(*disc),
          &WidgetPrim::Text(ref text) => texts.push(*text)
        }
      }
    }

    let render_input = RenderInput::new()
      .triangles(&tris)
      .quads(&quads)
      .discs(&discs)
      .texts(&texts, 1.);

    self.renderer.render(render_input)
  }
}

/// Widget primitives.
///
/// A widget primitive is used as primary a tool to build up widgets. A widget is basically just a
/// sum of widget primitives used to represent it.
pub enum WidgetPrim<'a> {
  Triangle(Triangle),
  Quad(Quad),
  Disc(Disc),
  Text(Text<'a>)
}

pub trait Widget<'a> {
  fn unwidget(&self) -> Vec<WidgetPrim<'a>>;
}

pub struct ProgressBar {
  w: f32,
  progress_quad: Quad,
  inactive_quad: Quad,
  recip_dur_sec: f32,
  listeners: Vec<Rc<RefCell<ProgressBarListener>>>
}

pub enum ProgressBarEvent {
  Set(Time),
  Toggle
}

pub trait ProgressBarListener {
  fn on(&mut self, e: ProgressBarEvent) -> bool;
}

impl ProgressBar {
  pub fn new(w: f32, h: f32, progress_color: ColorAlpha, inactive_color: ColorAlpha, dur_sec: f32) -> Rc<RefCell<Self>> {
    let pcol = *progress_color.as_ref();
    let icol = *inactive_color.as_ref();

    let progress_quad = Quad(
      Vert::new([0., 0., 0.], pcol),
      Vert::new([0., h, 0.], pcol),
      Vert::new([0., 0., 0.], pcol),
      Vert::new([0., h, 0.], pcol)
    );

    let inactive_quad = Quad(
      Vert::new([w, 0., 0.], icol),
      Vert::new([w, h, 0.], icol),
      Vert::new([0., 0., 0.], icol),
      Vert::new([0., h, 0.], icol)
    );

    Rc::new(RefCell::new(ProgressBar {
      w: w,
      progress_quad: progress_quad,
      inactive_quad: inactive_quad,
      recip_dur_sec: 1. / dur_sec,
      listeners: Vec::new()
    }))
  }

  /// Set the cursor (seconds).
  pub fn set(&mut self, cursor: f32) {
    let c = cursor * self.recip_dur_sec * self.w;

    // update the quads
    self.progress_quad.0.pos[0] = c;
    self.progress_quad.1.pos[0] = c;
    self.inactive_quad.2.pos[0] = c;
    self.inactive_quad.3.pos[0] = c;

    for l in &self.listeners {
      l.borrow_mut().on(ProgressBarEvent::Set(cursor));
    }
  }

  pub fn toggle(&mut self) {
    for l in &self.listeners {
      l.borrow_mut().on(ProgressBarEvent::Toggle);
    }
  }
}

impl<'a> Widget<'a> for ProgressBar {
  fn unwidget(&self) -> Vec<WidgetPrim<'a>> {
    vec![WidgetPrim::Quad(self.progress_quad), WidgetPrim::Quad(self.inactive_quad)]
  }
}
