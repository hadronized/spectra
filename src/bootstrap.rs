use gl;
use std::default::Default;
use std::os::raw::c_void;
use std::sync::mpsc;
use std::thread;

pub use glfw::{self, Action, Context, CursorMode, Key, MouseButton, Window};

#[derive(Clone, Copy, Debug)]
pub enum LuminanceBackend {
  GL33
}

impl Default for LuminanceBackend {
  fn default() -> Self {
    LuminanceBackend::GL33
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowDim {
  Windowed(u32, u32),
  FullScreen,
  FullScreenRestricted(u32, u32)
}

pub type Keyboard = mpsc::Receiver<(Key, Action)>;
pub type Mouse = mpsc::Receiver<(MouseButton, Action)>;
pub type MouseMove = mpsc::Receiver<[f64; 2]>;
pub type Scroll = mpsc::Receiver<[f64; 2]>;

pub fn bootstrap<App: FnOnce(u32, u32, Keyboard, Mouse, MouseMove, Scroll, Window)>(dim: WindowDim, title: &'static str, backend: LuminanceBackend, app: App) {
  info!("{} starting", title);
  info!("window mode: {:?}", dim);
  info!("luminance backend: {:?}", backend);

  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  match backend {
    LuminanceBackend::GL33 => {
      // OpenGL hints; implements luminance-gl’s core contexts creation
      glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
      glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
      glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    }
  }

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

  if cfg!(feature = "release") {
    deb!("hiding cursor");
    window.set_cursor_mode(CursorMode::Disabled);
  }

  window.set_key_polling(true);
  window.set_cursor_pos_polling(true);
  window.set_mouse_button_polling(true);
  window.set_scroll_polling(true);

  deb!("initializing OpenGL pointers");

  // init OpenGL
  gl::load_with(|s| window.get_proc_address(s) as *const c_void);

  // create channels to stream keyboard and mouse events
  let (kbd_snd, kbd_rcv) = mpsc::channel();
  let (mouse_snd, mouse_rcv) = mpsc::channel();
  let (mouse_move_snd, mouse_move_rcv) = mpsc::channel();
  let (scroll_snd, scroll_rcv) = mpsc::channel();

  deb!("spawning the event thread");
  let _ = thread::spawn(move || {
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
              let _ = mouse_move_snd.send([x, y]);
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
  app(w, h, kbd_rcv, mouse_rcv, mouse_move_rcv, scroll_rcv, window);
}
