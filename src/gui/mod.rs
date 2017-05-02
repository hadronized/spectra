use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bootstrap::{Action, EventHandler, EventSig, MouseButton};
use color::RGBA;
use compositing::Layer;
use overlay::{Disc, Overlay, Quad, RenderInput, Text, Triangle, Vert};
use resource::ResCache;

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
  overlay: Overlay,
  h: f32,
  tris: Vec<Triangle>,
  quads: Vec<Quad>,
  discs: Vec<Disc>,
  texts: Vec<Text<'a>>,

  widgets: HashMap<String, Box<Widget<'a> + 'a>>,

  // event stuff
  last_cursor: Option<[f32; 2]>,
  last_mouse_left_down: Option<[f32; 2]>,
  last_mouse_left_up: Option<[f32; 2]>,
  focused_widgets: HashMap<Focus, String>
}

impl<'a> GUI<'a> {
  pub fn new(viewport: Viewport, cache: &mut ResCache) -> Self {
    GUI {
      overlay: Overlay::new(viewport.w.ceil() as u32, viewport.h.ceil() as u32, 1024, 1024, 1024, cache),
      h: viewport.h,
      tris: Vec::new(),
      quads: Vec::new(),
      discs: Vec::new(),
      texts: Vec::new(),
      widgets: HashMap::new(),
      last_cursor: None,
      last_mouse_left_down: None,
      last_mouse_left_up: None,
      focused_widgets: HashMap::new(),
    }
  }

  pub fn add_widget<W>(&mut self, id: &str, widget: W) where W: 'a + Widget<'a> {
    self.widgets.insert(id.to_owned(), Box::new(widget));
  }

  pub fn remove_widget(&mut self, id: &str) {
    self.widgets.remove(id);
  }

  pub fn render_layer(&mut self) -> Layer {
    self.tris.clear();
    self.quads.clear();
    self.discs.clear();
    self.texts.clear();

    for widget in self.widgets.values() {
      for prim in widget.widget_prims() {
        match prim {
          WidgetPrim::Triangle(ref tri) => self.tris.push(*tri),
          WidgetPrim::Quad(ref quad) => self.quads.push(*quad),
          WidgetPrim::Disc(ref disc) => self.discs.push(*disc),
          WidgetPrim::Text(ref text) => self.texts.push(*text)
        }
      }
    }

    let render_input = RenderInput::new()
      .triangles(&self.tris)
      .quads(&self.quads)
      .discs(&self.discs)
      .texts(&self.texts, 1.);

    let overlay = &self.overlay;
    Layer::new(move |framebuffer| overlay.render(framebuffer, &render_input))
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Focus {
  MouseButton(MouseButton, Action),
  Drag
}

impl<'a> EventHandler for GUI<'a> {
  fn on_mouse_button(&mut self, button: MouseButton, action: Action) -> EventSig {
    if let MouseButton::Button1 = button {
      let last_cursor = self.last_cursor.unwrap();

      match action {
        Action::Press => {
          self.last_mouse_left_down = Some(last_cursor);
          self.last_mouse_left_up = None;

          for (key, widget) in &mut self.widgets {
            if widget.bounding_box().is_point_in(last_cursor) && widget.on_mouse_button(button, action) == EventSig::Focused {
              self.focused_widgets.insert(Focus::MouseButton(button, action), key.clone());
              break;
            }
          }
        },
        Action::Release => {
          // check whether it’s a click
          if px_dist(self.last_mouse_left_down.unwrap(), last_cursor) <= 5. {
            // it’s a click
            for widget in self.widgets.values() {
              if widget.bounding_box().is_point_in(last_cursor) {
                widget.on_click(last_cursor);
              }
            }
          }

          self.last_mouse_left_down = None;
          self.last_mouse_left_up = Some(last_cursor);

          self.focused_widgets.remove(&Focus::MouseButton(button, Action::Press));
          self.focused_widgets.remove(&Focus::Drag);
        },
        _ => ()
      }
    }

    EventSig::Handled
  }

  // TODO: change the implementation to take into account widget focus
  fn on_cursor_move(&mut self, cursor: [f32; 2]) -> EventSig {
    self.last_cursor = Some([cursor[0], self.h - cursor[1]]);

    if let Some(key) = self.focused_widgets.get(&Focus::Drag).cloned() {
      let focused = self.widgets.get(&key).unwrap();
      let down_cursor = self.last_mouse_left_down.unwrap();
      focused.on_drag(cursor, down_cursor);

      return EventSig::Handled;
    } else if let Some(key) = self.focused_widgets.get(&Focus::MouseButton(MouseButton::Button1, Action::Press)).cloned() {
      let focused = self.widgets.get(&key).unwrap();
      let down_cursor = self.last_mouse_left_down.unwrap();

      if px_dist(down_cursor, cursor) > 5. {
        self.focused_widgets.insert(Focus::Drag, key);
        focused.on_drag(cursor, down_cursor);

        return EventSig::Focused;
      }
    }

    EventSig::Ignored
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
  lower: [f32; 2],
  upper: [f32; 2]
}

impl BoundingBox {
  pub fn new(lower: [f32; 2], upper: [f32; 2]) -> Self {
    BoundingBox {
      lower: lower,
      upper: upper
    }
  }

  pub fn is_point_in(&self, p: [f32; 2]) -> bool {
    self.lower[0] <= p[0] && self.upper[0] >= p[0] && self.lower[1] <= p[1] && self.upper[1] >= p[1]
  }
}

pub trait Widget<'a>: EventHandler {
  fn widget_prims(&self) -> Vec<WidgetPrim<'a>>;
  fn bounding_box(&self) -> BoundingBox;
  fn on_click(&self, _: [f32; 2]) {}
  fn on_drag(&self, _: [f32; 2], _: [f32; 2]) {}
}

pub struct ProgressBar {
  x: f32,
  w: f32,
  progress_quad: Quad,
  inactive_quad: Quad,
  recip_dur_sec: f32,
  listeners: HashMap<String, Rc<RefCell<ProgressBarListener>>>
}

pub trait ProgressBarListener {
  /// Called whenever the value of the progress bar changes.
  fn on_set(&mut self, _: Time) {}
  /// Called whenever the progress bar is clicked.
  fn on_click(&mut self, _: [f32; 2]) {}
  /// Called whenever the progress bar is dragged.
  fn on_drag(&mut self, _: [f32; 2], _: [f32; 2]) {}
}

impl ProgressBar {
  pub fn new<PC, IC>(x: f32, y: f32, w: f32, h: f32, progress_color: PC, inactive_color: IC, dur_sec: Time) -> Rc<RefCell<Self>> where PC: Into<RGBA>, IC: Into<RGBA> {
    let pcol = progress_color.into().into();
    let icol = inactive_color.into().into();

    let progress_quad = Quad(
      Vert::new([x, y, 0.], pcol),
      Vert::new([x, y + h, 0.], pcol),
      Vert::new([x, y, 0.], pcol),
      Vert::new([x, y + h, 0.], pcol)
    );

    let inactive_quad = Quad(
      Vert::new([x + w, y, 0.], icol),
      Vert::new([x + w, y + h, 0.], icol),
      Vert::new([x, y, 0.], icol),
      Vert::new([x, y + h, 0.], icol)
    );

    Rc::new(RefCell::new(ProgressBar {
      x: x,
      w: w,
      progress_quad: progress_quad,
      inactive_quad: inactive_quad,
      recip_dur_sec: 1. / dur_sec,
      listeners: HashMap::new()
    }))
  }

  /// Add a listener.
  pub fn add_listener(&mut self, key: &str, listener: Rc<RefCell<ProgressBarListener>>) {
    self.listeners.insert(key.to_owned(), listener);
  }

  /// Remove a listener.
  pub fn remove_listener(&mut self, key: &str) {
    self.listeners.remove(key);
  }

  /// Set the cursor (seconds).
  pub fn set(&mut self, cursor: Time) {
    let cursor = cursor.max(0.).min(1. / self.recip_dur_sec); // clamp
    let c = self.x + cursor * self.recip_dur_sec * self.w;

    // update the quads
    self.progress_quad.0.pos[0] = c;
    self.progress_quad.1.pos[0] = c;
    self.inactive_quad.2.pos[0] = c;
    self.inactive_quad.3.pos[0] = c;

    for l in self.listeners.values() {
      l.borrow_mut().on_set(cursor);
    }
  }

  /// Get the cursor (seconds).
  pub fn get(&self) -> Time {
    (self.progress_quad.0.pos[0] - self.x) / self.w
  }

  fn on_cursor_change(&mut self, cursor: [f32; 2]) {
    let c = (cursor[0] as f32).min(self.x + self.w).max(self.x); // clamp

    // update the quads
    self.progress_quad.0.pos[0] = c;
    self.progress_quad.1.pos[0] = c;
    self.inactive_quad.2.pos[0] = c;
    self.inactive_quad.3.pos[0] = c;

    let time = (c - self.x) / (self.recip_dur_sec * self.w);
    for l in self.listeners.values() {
      l.borrow_mut().on_set(time);
    }
  }
}

impl EventHandler for Rc<RefCell<ProgressBar>> {
  fn on_mouse_button(&mut self, _: MouseButton, action: Action) -> EventSig {
    if action == Action::Press {
      EventSig::Focused
    } else {
      EventSig::Ignored
    }
  }
}

impl<'a> Widget<'a> for Rc<RefCell<ProgressBar>> {
  fn widget_prims(&self) -> Vec<WidgetPrim<'a>> {
    let bar = &self.borrow();
    vec![WidgetPrim::Quad(bar.progress_quad), WidgetPrim::Quad(bar.inactive_quad)]
  }

  fn bounding_box(&self) -> BoundingBox {
    let bar = &self.borrow();
    let lower = bar.progress_quad.2.pos;
    let upper = bar.inactive_quad.1.pos;

    BoundingBox::new([lower[0], lower[1]], [upper[0], upper[1]])
  }

  fn on_click(&self, cursor: [f32; 2]) {
    self.borrow_mut().on_cursor_change(cursor);

    for l in self.borrow().listeners.values() {
      l.borrow_mut().on_click(cursor);
    }
  }

  fn on_drag(&self, cursor: [f32; 2], down_cursor: [f32; 2]) {
    self.borrow_mut().on_cursor_change([cursor[0], down_cursor[1]]);

    for l in self.borrow().listeners.values() {
      l.borrow_mut().on_drag(cursor, down_cursor);
    }
  }
}

/// Pixel distance between two points.
fn px_dist(a: [f32; 2], b: [f32; 2]) -> f32 {
  let x = b[0] - a[0];
  let y = b[1] - a[1];

  f32::sqrt(x*x + y*y)
}
