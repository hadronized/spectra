use gl;
use std::os::raw::c_void;
use std::process::exit;
use std::sync::mpsc;

pub use glfw::{self, Action, Context, CursorMode, Key, MouseButton};

pub type Keyboard = mpsc::Receiver<(Key, Action)>;
pub type Mouse = mpsc::Receiver<(MouseButton, Action)>;
pub type MouseMove = mpsc::Receiver<[f64; 2]>;
pub type Scroll = mpsc::Receiver<[f64; 2]>;

pub fn with_window<Init: Fn(u32, u32, Keyboard, Mouse, MouseMove, Scroll) -> Result<Box<FnMut() -> bool>, String>>(dim: Option<(u32, u32)>, title: &'static str, init: Init) {
  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  // OpenGL hint
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
  glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));

  let (mut window, events, w, h) = if let Some((w, h)) = dim {
    let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::Windowed)
      .expect("Failed to create GLFW window.");

    (window, events, w, h)
  } else {
    glfw.with_primary_monitor(|glfw, monitor| {
      let monitor = monitor.unwrap();
      let vmode = monitor.get_video_mode().expect("primary monitor’s video mode");
      let (w, h) = (vmode.width, vmode.height);
      let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::FullScreen(monitor)).expect("Failed to create GLFW window.");

      (window, events, w, h)
    })
  };

  window.make_current();
  if cfg!(feature = "release") {
    window.set_cursor_mode(CursorMode::Disabled);
    window.set_key_polling(true);
  } else {
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);
  }

  // init OpenGL
  gl::load_with(|s| window.get_proc_address(s) as *const c_void);

  info!("using resolution {}x{}", w, h);

  // create channels to stream keyboard and mouse events
  let (kbd_snd, kbd_rcv) = mpsc::channel();
  let (mouse_snd, mouse_rcv) = mpsc::channel();
  let (mouse_move_snd, mouse_move_rcv) = mpsc::channel();
  let (scroll_snd, scroll_rcv) = mpsc::channel();

  match init(w, h, kbd_rcv, mouse_rcv, mouse_move_rcv, scroll_rcv) {
    Ok(mut run) => {
      while !window.should_close() {
        glfw.poll_events();

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
  
        let alive = run();
        window.swap_buffers();

        if !alive {
          window.set_should_close(true);
          break;
        }
      }
    },
    Err(e) => {
      err!("unable to initialize application");
      err!("{}", e);
      exit(1);
    }
  }
}
