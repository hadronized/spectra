use std::cell::RefCell;
use std::rc::Rc;

use color::ColorAlpha;
use linear::Vector2;
use overlay::{Quad, Vert};

type Time = f32;

struct Viewport {
  x: f32,
  y: f32,
  w: f32,
  h: f32
}

struct GUI {
  viewport: Viewport
}

impl GUI {
  fn new(viewport: Viewport) -> Self {
    GUI {
      viewport: viewport
    }
  }

  //fn new_timeline(h: u32, progress_color: Color, inactive_color: Color, dur_sec: f32) -> Timeline {
}

mod timeline {
  use super::*;

  struct Timeline {
    viewport: Viewport,
    h: f32,
    progress_color: ColorAlpha,
    inactive_color: ColorAlpha,
    //hints_color: Color,
    progress_quad: Quad,
    inactive_quad: Quad,
    recip_dur_sec: f32,
    listeners: Vec<Rc<RefCell<Listener>>>
  }
  
  enum Event {
    Set(Time),
    Toggle
  }
  
  trait Listener {
    fn on(&mut self, e: Event) -> bool;
  }

  impl Timeline {
    fn new(viewport: Viewport, h: f32, progress_color: ColorAlpha, inactive_color: ColorAlpha, dur_sec: f32) -> Self {
      let w = viewport.w;
      let pcol = *progress_color.as_ref();
      let icol = *inactive_color.as_ref();

      let progress_quad = Quad(
        Vert::new([0., 0., 0.], pcol),
        Vert::new([0., 0., 0.], pcol),
        Vert::new([0., h, 0.], pcol),
        Vert::new([0., h, 0.], pcol)
      );

      let inactive_quad = Quad(
        Vert::new([0., 0., 0.], icol),
        Vert::new([w, 0., 0.], icol),
        Vert::new([w, h, 0.], icol),
        Vert::new([0., h, 0.], icol)
      );

      Timeline {
        viewport: viewport,
        h: h,
        progress_color: progress_color,
        inactive_color: inactive_color,
        progress_quad: progress_quad,
        inactive_quad: inactive_quad,
        recip_dur_sec: 1. / dur_sec,
        listeners: Vec::new()
      }
    }

    /// Set the cursor (seconds).
    fn set(&mut self, cursor: f32) {
      let c = cursor * self.recip_dur_sec;

      // update the quads
      self.progress_quad.1.pos[0] = c;
      self.progress_quad.2.pos[0] = c;
      self.inactive_quad.0.pos[0] = c;
      self.inactive_quad.3.pos[0] = c;

      for l in &self.listeners {
        l.borrow_mut().on(Event::Set(cursor));
      }
    }

    fn toggle(&mut self) {
      for l in &self.listeners {
        l.borrow_mut().on(Event::Toggle);
      }
    }
  }
}
