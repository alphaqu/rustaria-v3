use std::os::raw::{c_int, c_void};
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use eyre::{ContextCompat, Result, WrapErr};
use glfw::{
    Context, Glfw, OpenGlProfileHint, SwapInterval, WindowEvent, WindowHint, WindowMode, with_c_str,
};
use glium::{Frame, Program, SwapBuffersError, Vertex};
use glium::backend::Backend;
use glium::debug::{DebugCallbackBehavior, Severity};
use glium::program::SourceCode;
use tracing::{event, Level};
use crate::renderer::buffer::MeshDrawer;

pub struct Frontend {
    glfw: Glfw,
    window: Rc<Window>,
    events: Receiver<(f64, WindowEvent)>,
    pub ctx: Rc<glium::backend::Context>,

    pub dimensions: (u32, u32),
    pub screen_ratio: f32,
}

impl Frontend {
    pub fn new() -> Result<Frontend> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::ContextVersion(4, 6));

        let (mut window, events) = glfw
            .create_window(900, 600, "your mom", WindowMode::Windowed)
            .wrap_err("Failed to create window")?;

        window.make_current();
        glfw.set_swap_interval(SwapInterval::Sync(1));

        window.set_key_polling(true);
        window.set_size_polling(true);
        window.set_scroll_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);

        let window = Rc::new(Window(window));
        let mut frontend = Frontend {
            glfw,
            ctx: unsafe {
                glium::backend::Context::new(
                    window.clone(),
                    false,
                    DebugCallbackBehavior::Custom {
                        synchronous: false,
                        callback: Box::new(|src, kind, severity, _, _, msg| {
                            match severity {
                                Severity::Notification => {
                                    event!(target: "opengl", Level::DEBUG, ?src, ?kind, "{}", msg);
                                }
                                Severity::Low => {
                                    event!(target: "opengl", Level::INFO, ?src, ?kind, "{}", msg);
                                }
                                Severity::Medium => {
                                    event!(target: "opengl", Level::WARN, ?src, ?kind, "{}", msg);
                                }
                                Severity::High => {
                                    event!(target: "opengl", Level::ERROR, ?src, ?kind, "{}", msg);
                                }
                            }
                        }),
                    },
                )
            }?,
            window,
            events,
            dimensions: (0, 0),
            screen_ratio: 0.0
        };

        frontend.resize(900, 600);
        Ok(frontend)
    }

    pub fn create_drawer<T: Copy + Vertex>(&self) -> Result<MeshDrawer<T>> {
        MeshDrawer::new(self)
    }

    pub fn create_program(&self, code: SourceCode) -> Result<Program> {
        Program::new(&self.ctx, code).wrap_err("Failed to create program")
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.dimensions = (width, height);
        self.screen_ratio = height as f32 / width as f32;
    }

    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        let mut out = Vec::new();
        self.glfw.poll_events();
        while let Ok((_, event)) = self.events.try_recv() {
            match event {
                WindowEvent::FramebufferSize(width, height) => {
                    self.resize(width as u32, height as u32);
                }
                WindowEvent::Close => {
                    unsafe { glfw::ffi::glfwSetWindowShouldClose( self.window.0.window_ptr(), true as c_int) }
                }
                _ => {}
            }
            out.push(event);
        }

        out
    }

    pub fn running(&self) -> bool {
        !self.window.0.should_close()
    }

    pub fn start_draw(&mut self) -> Frame {
        Frame::new(self.ctx.clone(), self.dimensions)
    }
}

struct Window(glfw::Window);

unsafe impl Backend for Window {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        unsafe {
            glfw::ffi::glfwSwapBuffers(self.0.window_ptr());
        }
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        debug_assert!(!glfw::ffi::glfwGetCurrentContext().is_null());
        with_c_str(symbol, |procname| {
            glfw::ffi::glfwGetProcAddress(procname)
        })
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let size = self.0.get_framebuffer_size();
        (size.0 as u32, size.1 as u32)
    }

    fn is_current(&self) -> bool {
        self.0.is_current()
    }

    unsafe fn make_current(&self) {
            glfw::ffi::glfwMakeContextCurrent(self.0.window_ptr());
    }
}
