use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bootstrap::{Action, EventHandler, MouseButton};
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

  widgets: HashMap<String, Box<Widget<'a> + 'a>>,

  // event stuff
  last_cursor: Option<[f64; 2]>,
  last_mouse_left_down: Option<[f64; 2]>,
  last_mouse_left_up: Option<[f64; 2]>
}

impl<'a> GUI<'a> {
  pub fn new(viewport: Viewport, scene: &mut Scene) -> Self {
    GUI {
      renderer: Renderer::new(viewport.w.ceil() as u32, viewport.h.ceil() as u32, 1024, 1024, 1024, scene),
      widgets: HashMap::new(),
      last_cursor: None,
      last_mouse_left_down: None,
      last_mouse_left_up: None
    }
  }

  pub fn add_widget<W>(&mut self, id: &str, widget: &W) where W: 'a + Clone + Widget<'a> {
    self.widgets.insert(id.to_owned(), Box::new(widget.clone()));
  }

  pub fn remove_widget(&mut self, id: &str) {
    self.widgets.remove(id);
  }

  pub fn render(&self) -> &Texture2D {
    let mut tris = Vec::new();
    let mut quads = Vec::new();
    let mut discs = Vec::new();
    let mut texts = Vec::new();

    for widget in self.widgets.values() {
      for prim in widget.unwidget() {
        match prim {
          WidgetPrim::Triangle(ref tri) => tris.push(*tri),
          WidgetPrim::Quad(ref quad) => quads.push(*quad),
          WidgetPrim::Disc(ref disc) => discs.push(*disc),
          WidgetPrim::Text(ref text) => texts.push(*text)
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

impl<'a> EventHandler for GUI<'a> {
  fn on_mouse_button(&mut self, button: MouseButton, action: Action) -> bool {
    if let MouseButton::Button1 = button {
      let last_cursor = self.last_cursor.unwrap();

      match action {
        Action::Press => {
          self.last_mouse_left_down = Some(last_cursor);
          self.last_mouse_left_up = None;
        },
        Action::Release => {
          // check whether it’s a click
          if px_dist(self.last_mouse_left_down.unwrap(), last_cursor) <= 5. {
            // it’s a click
            for widget in self.widgets.values() {
              widget.on_click(last_cursor);
            }
          }

          self.last_mouse_left_down = None;
          self.last_mouse_left_up = Some(last_cursor);

        },
        _ => ()
      }
    }

    true
  }

  // TODO: change the implementation to take into account widget focus
  fn on_cursor_move(&mut self, cursor: [f64; 2]) -> bool {
    self.last_cursor = Some(cursor);
    true
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
  fn on_click(&self, cursor: [f64; 2]);
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

  pub fn on_click(&mut self, cursor: [f64; 2]) {
    let c = cursor[0] as f32;

    // update the quads
    self.progress_quad.0.pos[0] = c;
    self.progress_quad.1.pos[0] = c;
    self.inactive_quad.2.pos[0] = c;
    self.inactive_quad.3.pos[0] = c;

    for l in &self.listeners {
      l.borrow_mut().on(ProgressBarEvent::Set(c / (self.recip_dur_sec * self.w)));
    }
  }
}

impl<'a> Widget<'a> for Rc<RefCell<ProgressBar>> {
  fn unwidget(&self) -> Vec<WidgetPrim<'a>> {
    let bar = &self.borrow();
    vec![WidgetPrim::Quad(bar.progress_quad), WidgetPrim::Quad(bar.inactive_quad)]
  }

  fn on_click(&self, cursor: [f64; 2]) {
    self.borrow_mut().on_click(cursor);
  }
}

/// Pixel distance between two points.
fn px_dist(a: [f64; 2], b: [f64; 2]) -> f64 {
  let x = b[0] - a[0];
  let y = b[1] - a[1];

  f64::sqrt(x*x + y*y)
}