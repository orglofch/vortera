extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate libc;

mod shader;

use cgmath::{Deg, Matrix, Matrix4, One, vec3, perspective};
use glutin::GlContext;
use std::ffi::CStr;

macro_rules! c_str {
    ($literal:expr) => {
        CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    }
}

struct State<'a> {
    running: bool,

    //world: world::World,
    gl_window: &'a glutin::GlWindow,
}

impl<'a> State<'a> {
    fn new(gl_window: &'a glutin::GlWindow) -> State<'a> {
        State {
            running: true,
            gl_window: gl_window,
        }
    }
}

fn handle_event(event: glutin::Event, state: &mut State) {
    match event {
        glutin::Event::WindowEvent { event, .. } => {
            match event {
                glutin::WindowEvent::Closed => state.running = false,
                glutin::WindowEvent::Resized(w, h) => state.gl_window.resize(w, h),
                _ => (),
            }
        }
        _ => (),
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ECS")
        .with_dimensions(1024, 780);
    let context = glutin::ContextBuilder::new().with_vsync(true);

    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

    // Set up GL Context.
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 0.0);

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }

    let mut shader: Shader = Shader { id: 0 };
    unsafe {
        shader = Shader::create("data/shader/vs.vert", "data/shader/fs.frag");

        gl::UseProgram(shader.id);

        // TODO(orglofch): Set aspect ratio appropriately.
        let projection: Matrix4<f32> = perspective(Deg(80.0), 1.0, 0.1, 1000.0);
        gl::UniformMatrix4fv(gl::GetUniformLocation(shader.id, c_str!("projection").as_ptr()),
                             1,
                             gl::FALSE,
                             projection.as_ptr());
        let view = Matrix4::<f32>::from_translation(vec3(0.0, 0.0, -5.0));
        gl::UniformMatrix4fv(gl::GetUniformLocation(shader.id, c_str!("view").as_ptr()),
                             1,
                             gl::FALSE,
                             view.as_ptr());
    }

    // TODO(orglofch): Use quaternions and add track-ball rotation.
    let mut model_mat = Matrix4::<f32>::one();
    let mut angle = 0.0;

    let mut state = State::new(&gl_window);
    while state.running {
        events_loop.poll_events(|event| handle_event(event, &mut state));

        // TODO(orglofch): Why doesn't multi assign work?
        model_mat = Matrix4::<f32>::from_angle_y(Deg(angle));

        angle += 0.4;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::UniformMatrix4fv(gl::GetUniformLocation(shader.id, c_str!("model").as_ptr()),
                                 1,
                                 gl::FALSE,
                                 model_mat.as_ptr());
        }

        state.gl_window.swap_buffers().unwrap();
    }
}
