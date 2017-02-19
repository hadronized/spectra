use std::cell::RefCell;
use std::rc::Rc;

use color::ColorAlpha;
use overlay::{Quad, Renderer, RenderInput, Texture2D, Vert};
use scene::Scene;

type Time = f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32
}

pub struct GUI {
  // common
  viewport: Viewport,
  renderer: Renderer,

  // widgets
  timeline: Option<Rc<RefCell<Timeline>>>,
}

impl GUI {
  pub fn new(viewport: Viewport, scene: &mut Scene) -> Self {
    GUI {
      viewport: viewport,
      renderer: Renderer::new(viewport.w.ceil() as u32, viewport.h.ceil() as u32, 1024, 1024, 1024, scene),
      timeline: None
    }
  }

  pub fn new_timeline(&mut self, h: f32, progress_color: ColorAlpha, inactive_color: ColorAlpha, dur_sec: f32) -> Rc<RefCell<Timeline>> {
    let timeline = Rc::new(RefCell::new(Timeline::new(self.viewport, h, progress_color, inactive_color, dur_sec)));

    self.timeline = Some(timeline.clone());
    timeline
  }

  pub fn render(&self) -> &Texture2D {
    let mut quads = Vec::new();

    if let Some(ref timeline) = self.timeline {
      let timeline = &timeline.borrow();
      quads.push(timeline.progress_quad);
      quads.push(timeline.inactive_quad);
    }

    let render_input = RenderInput::new()
      .quads(&quads);

    self.renderer.render(render_input)
  }
}

pub struct Timeline {
  viewport: Viewport,
  progress_color: ColorAlpha,
  inactive_color: ColorAlpha,
  //hints_color: Color,
  progress_quad: Quad,
  inactive_quad: Quad,
  recip_dur_sec: f32,
  listeners: Vec<Rc<RefCell<TimelineListener>>>
}

pub enum TimelineEvent {
  Set(Time),
  Toggle
}

pub trait TimelineListener {
  fn on(&mut self, e: TimelineEvent) -> bool;
}

impl Timeline {
  pub fn new(viewport: Viewport, h: f32, progress_color: ColorAlpha, inactive_color: ColorAlpha, dur_sec: f32) -> Self {
    let w = viewport.w;
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

    Timeline {
      viewport: viewport,
      progress_color: progress_color,
      inactive_color: inactive_color,
      progress_quad: progress_quad,
      inactive_quad: inactive_quad,
      recip_dur_sec: 1. / dur_sec,
      listeners: Vec::new()
    }
  }

  /// Set the cursor (seconds).
  pub fn set(&mut self, cursor: f32) {
    let c = cursor * self.recip_dur_sec * self.viewport.w;

    // update the quads
    self.progress_quad.0.pos[0] = c;
    self.progress_quad.1.pos[0] = c;
    self.inactive_quad.2.pos[0] = c;
    self.inactive_quad.3.pos[0] = c;

    for l in &self.listeners {
      l.borrow_mut().on(TimelineEvent::Set(cursor));
    }
  }

  pub fn toggle(&mut self) {
    for l in &self.listeners {
      l.borrow_mut().on(TimelineEvent::Toggle);
    }
  }
}

