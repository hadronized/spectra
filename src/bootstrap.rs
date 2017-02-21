use gl;
use glfw::{self, Action, Context, CursorMode, Key, MouseButton, SwapInterval, Window};
use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use camera::{Camera, Freefly};
use transform::Translation;


/// Dimension of the window to create.
#[derive(Clone, Copy, Debug)]
pub enum WindowDim {
  Windowed(u32, u32),
  FullScreen,
  FullScreenRestricted(u32, u32)
}

type Keyboard = mpsc::Receiver<(Key, Action)>;
type Mouse = mpsc::Receiver<(MouseButton, Action)>;
type MouseMove = mpsc::Receiver<[f64; 2]>;
type Scroll = mpsc::Receiver<[f64; 2]>;

/// Empty handler.
///
/// This handler will just let pass all events without doing anything. It’s only useful for debug
/// purposes when you don’t want to bother with interaction – it doesn’t even let you close the
/// application!
#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub struct Unhandled;

/// Class of event handlers.
pub trait EventHandler {
  /// Implement this function if you want to react to key strokes.
  fn on_key(&mut self, _: Key, _: Action) -> bool { true }
  /// Implement this function if you want to react to mouse button events.
  fn on_mouse_button(&mut self, _: MouseButton, _: Action) -> bool { true }
  /// Implement this function if you want to react to cursor moves.
  fn on_cursor_move(&mut self, _: [f64; 2]) -> bool { true }
  /// Implement this function if you want to react to scroll events.
  fn on_scroll(&mut self, _: [f64; 2]) -> bool { true }
}

impl EventHandler for Unhandled {}

/// Device object.
///
/// Upon bootstrapping, this type is created to add interaction and context handling.
pub struct Device {
  /// Width of the window.
  w: u32,
  /// Height of the window.
  h: u32,
  /// Some kind of epoch start the application started at.
  start_time: Instant,
  /// Keyboard receiver.
  kbd: Keyboard,
  /// Mouse receiver.
  mouse: Mouse,
  /// Cursor receiver.
  cursor: MouseMove,
  /// Scroll receiver.
  scroll: Scroll,
  /// Window.
  window: Window,
  /// Event thread join handle. Unused and keep around until death.
  #[allow(dead_code)]
  event_thread: thread::JoinHandle<()>
}

impl Device {
  /// Entry point.
  ///
  /// This function is the first one you have to call before anything else related to this crate.
  /// 
  /// # Arguments
  ///
  /// - `dim`: dimension of the window to create
  /// - `title`: title to give to the window
  pub fn bootstrap(dim: WindowDim, title: &'static str) -> Self {
    info!("{} starting", title);
    info!("window mode: {:?}", dim);

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // OpenGL hints
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));

    // open a window in windowed or fullscreen mode
    let (mut window, events, w, h) = match dim {
      WindowDim::Windowed(w, h) => {
        let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::Windowed).expect("Failed to create GLFW window.");
        (window, events, w, h)
      },
      WindowDim::FullScreen => {
        glfw.with_primary_monitor(|glfw, monitor| {
          let monitor = monitor.unwrap();
          let vmode = monitor.get_video_mode().expect("primary monitor’s video mode");
          let (w, h) = (vmode.width, vmode.height);

          let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::FullScreen(monitor)).expect("Failed to create GLFW window.");
          (window, events, w, h)
        })
      },
      WindowDim::FullScreenRestricted(w, h) => {
        glfw.with_primary_monitor(|glfw, monitor| {
          let monitor = monitor.unwrap();

          let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::FullScreen(monitor)).expect("Failed to create GLFW window.");
          (window, events, w, h)
        })
      }
    };

    deb!("opened window");

    window.make_current();

    // FIXME: use a target instead
    if cfg!(feature = "release") {
      deb!("hiding cursor");
      window.set_cursor_mode(CursorMode::Disabled);
    }

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);
    glfw.set_swap_interval(SwapInterval::Sync(1));

    deb!("initializing OpenGL pointers");

    // init OpenGL
    gl::load_with(|s| window.get_proc_address(s) as *const c_void);

    // create channels to stream keyboard and mouse events
    let (kbd_snd, kbd_rcv) = mpsc::channel();
    let (mouse_snd, mouse_rcv) = mpsc::channel();
    let (cursor_snd, cursor_rcv) = mpsc::channel();
    let (scroll_snd, scroll_rcv) = mpsc::channel();

    deb!("spawning the event thread");
    let event_thread = thread::spawn(move || {
      loop {
        glfw.wait_events();

        for (_, event) in glfw::flush_messages(&events) {
          match event {
            glfw::WindowEvent::Key(key, _, action, _) => {
              let _ = kbd_snd.send((key, action));
            },
            glfw::WindowEvent::MouseButton(button, action, _) => {
              let _ = mouse_snd.send((button, action));
            },
            glfw::WindowEvent::CursorPos(x, y) => {
              let _ = cursor_snd.send([x, y]);
            },
            glfw::WindowEvent::Scroll(x, y) => {
              let _ = scroll_snd.send([x, y]);
            },
            _ => {},
          }
        }
      }
    });

    deb!("bootstrapping finished");

    Device {
      w: w,
      h: h,
      start_time: Instant::now(),
      kbd: kbd_rcv,
      mouse: mouse_rcv,
      cursor: cursor_rcv,
      scroll: scroll_rcv,
      window: window,
      event_thread: event_thread
    }
  }

  /// Width of the attached window.
  #[inline]
  pub fn width(&self) -> u32 {
    self.w
  }

  /// Height of the attached window.
  #[inline]
  pub fn height(&self) -> u32 {
    self.h
  }

  /// Current time, starting from the beginning of the creation of that object.
  pub fn time(&self) -> f64 {
    let elapsed = Instant::now() - self.start_time;
    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
  }

  /// Dispatch events to a handler.
  pub fn dispatch_events<H>(&self, handler: &mut H) -> bool where H: EventHandler {
    while let Ok((key, action)) = self.kbd.try_recv() {
      if !handler.on_key(key, action) {
        return false;
      }
    }

    while let Ok((button, action)) = self.mouse.try_recv() {
      if !handler.on_mouse_button(button, action) {
        return false;
      }
    }

    while let Ok(xy) = self.cursor.try_recv() {
      if !handler.on_cursor_move(xy) {
        return false;
      }
    }

    while let Ok(xy) = self.scroll.try_recv() {
      if !handler.on_scroll(xy) {
        return false;
      }
    }

    true
  }

  /// Step function.
  ///
  /// This function provides two features:
  ///
  /// - it runs a *drawer* function, responsible of rendering a single frame, by passing it the
  ///   current time
  /// - it performs process idleing if you have requested a certain *framerate* – frame per second.
  ///
  /// The second feature is very neat because it lets you handle the scheduler off your application
  /// and then contribute to better CPU usage.
  ///
  /// > Note: if you pass `None`, no idleing will take place. However, you might be blocked by the
  /// *VSync* if enabled in your driver.
  pub fn step<FPS, R>(&mut self, fps: FPS, mut draw_frame: R) -> bool where FPS: Into<Option<u32>>, R: FnMut(f64) {
    if self.window.should_close() {
      return false;
    }

    let t = self.time();

    draw_frame(t);
    self.window.swap_buffers();

    // wait for next frame according to the wished FPS
    if let Some(fps) = fps.into() {
      let fps = fps as f64;
      let max_time = 1. / fps;
      let elapsed_time = self.time() - t;

      if elapsed_time < max_time {
        let sleep_time = max_time - elapsed_time;
        thread::sleep(Duration::from_millis((sleep_time * 1e3) as u64));
      }
    }

    true
  }
}

/// Freefly handler.
///
/// This handler is very neat as it provides freefly interaction.
pub struct FreeflyHandler {
  camera: Rc<RefCell<Camera<Freefly>>>,
  left_down: bool,
  right_down: bool,
  last_cursor: [f64; 2]
}

impl FreeflyHandler {
  pub fn new(camera: Rc<RefCell<Camera<Freefly>>>) -> Self {
    FreeflyHandler {
      camera: camera,
      left_down: false,
      right_down: false,
      last_cursor: [0., 0.]
    }
  }

  fn move_camera_on_event(&mut self, key: Key) {
    let camera = &mut self.camera.borrow_mut();

    match key {
      Key::W => camera.mv(Translation::new(0., 0., 1.)),
      Key::S => camera.mv(Translation::new(0., 0., -1.)),
      Key::A => camera.mv(Translation::new(1., 0., 0.)),
      Key::D => camera.mv(Translation::new(-1., 0., 0.)),
      Key::Q => camera.mv(Translation::new(0., -1., 0.)),
      Key::E => camera.mv(Translation::new(0., 1., 0.)),
      _ => ()
    }
  }

  fn orient_camera_on_event(&mut self, cursor: [f64; 2]) {
    let camera = &mut self.camera.borrow_mut();

    if self.left_down {
      let (dx, dy) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      camera.look_around(Translation::new(dy as f32, dx as f32, 0.));
    } else if self.right_down {
      let (dx, _) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      camera.look_around(Translation::new(0., 0., dx as f32));
    }

    self.last_cursor = cursor;
  }
}

impl EventHandler for FreeflyHandler {
  fn on_key(&mut self, key: Key, action: Action) -> bool {
    match action {
      Action::Press | Action::Repeat => self.move_camera_on_event(key),
      Action::Release => if key == Key::Escape { return false }
    }

    true
  }

  fn on_mouse_button(&mut self, button: MouseButton, action: Action) -> bool {
    match (button, action) {
      (MouseButton::Button1, Action::Press) => {
        self.left_down = true;
      },
      (MouseButton::Button1, Action::Release) => {
        self.left_down = false;
      },
      (MouseButton::Button2, Action::Press) => {
        self.right_down = true;
      },
      (MouseButton::Button2, Action::Release) => {
        self.right_down = false;
      },
      _ => ()
    }

    true
  }

  fn on_cursor_move(&mut self, dir: [f64; 2]) -> bool {
    self.orient_camera_on_event(dir);
    true
  }
}
