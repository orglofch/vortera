extern crate gl;

#[cfg(debug_assertions)]
extern crate notify;

use gl::types::*;
use self::notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;
use std::str;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

static VERTEX_DEFINE: &'static str = "#define VERTEX \n";
static FRAGMENT_DEFINE: &'static str = "#define FRAGMENT \n";

pub struct Shader {
    pub id: u32,
}

impl Shader {
    /**
     * Creates a shader from a vertex and fragment shader stored in separate files.
     *
     * @param vs_path The vertex shader file path.
     * @param fs_path The fragment shader file path.
     */
    pub unsafe fn create(vs_path: &str, fs_path: &str) -> Shader {
        let mut vs_file = File::open(vs_path).expect(&format!("Failed to open {}", vs_path));
        let mut fs_file = File::open(fs_path).expect(&format!("Failed to open {}", fs_path));

        let mut vs_src = String::new();
        let mut fs_src = String::new();

        vs_file
            .read_to_string(&mut vs_src)
            .expect(&format!("Failed to read {}", vs_path));
        fs_file
            .read_to_string(&mut fs_src)
            .expect(&format!("Failed to read {}", fs_path));

        let program = Shader::compile_and_link(&vs_src, &fs_src);

        let shader = Shader { id: program };

        if cfg!(debug_assertions) {
            // Initialize shader hot-loading.
            Shader::debug_watch_thread(&shader, vs_path.to_owned());
            Shader::debug_watch_thread(&shader, fs_path.to_owned());
        }

        shader
    }

    /**
     * Creates a shader from a vertex and fragment shader stored in the same file.
     *
     * Requires the input file to have the vertex shader inside an "#ifdef VERTEX" and
     * the fragment shader inside an "#ifdef FRAGMENT"".
     *
     * @param vs_fs_path Combined vertex & fragment shader file path.
     */
    pub unsafe fn create_joined(vs_fs_path: &str) -> Shader {
        let mut file = File::open(vs_fs_path).expect(&format!("Failed to open {}", vs_fs_path));

        let mut vs_fs_src = String::new();

        file.read_to_string(&mut vs_fs_src)
            .expect(&format!("Failed to read {}", vs_fs_path));

        // Concatenate the #define headers with the source file.
        let vs_src = VERTEX_DEFINE.to_owned() + &vs_fs_src;
        let fs_src = FRAGMENT_DEFINE.to_owned() + &vs_fs_src;

        let program = Shader::compile_and_link(&vs_src, &fs_src);

        let shader = Shader { id: program };

        if cfg!(debug_assertions) {
            // Initialize shader hot-loading.
            Shader::debug_watch_thread(&shader, vs_fs_path.to_owned());
        }

        shader
    }

    /** Compile and link a vertex and fragment shader into a single program. */
    unsafe fn compile_and_link(vs_src: &String, fs_src: &String) -> u32 {
        let vs = Shader::compile(vs_src, gl::VERTEX_SHADER);
        let fs = Shader::compile(fs_src, gl::FRAGMENT_SHADER);

        // Link.
        let program = gl::CreateProgram();

        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);

        gl::LinkProgram(program);

        // Check Status.
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            panic!("{}", Shader::get_program_info(program));
        }

        program
    }

    /** Compile a single shader. */
    unsafe fn compile(src: &String, shader_type: GLenum) -> GLuint {
        let src_c_str = CString::new(src.as_bytes()).unwrap();

        let shader = gl::CreateShader(shader_type);

        // Compile.
        gl::ShaderSource(shader, 1, &src_c_str.as_ptr(), ptr::null());

        gl::CompileShader(shader);

        // Check Status.
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            panic!("{}", Shader::get_shader_info(shader));
        }

        shader
    }

    unsafe fn get_program_info(program: GLuint) -> String {
        let mut info_len = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut info_len);

        // Skip trailing null character.
        info_len -= 1;

        let mut buffer = Vec::with_capacity(info_len as usize);
        buffer.set_len(info_len as usize);

        gl::GetProgramInfoLog(program,
                              info_len,
                              ptr::null_mut(),
                              buffer.as_mut_ptr() as *mut GLchar);

        return String::from_utf8(buffer).expect("Program info-log couldn't be parsed as utf8");
    }

    unsafe fn get_shader_info(shader: GLuint) -> String {
        let mut info_len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_len);

        // Skip trailing null character.
        info_len -= 1;

        let mut buffer = Vec::with_capacity(info_len as usize);
        buffer.set_len(info_len as usize);

        gl::GetShaderInfoLog(shader,
                             info_len,
                             ptr::null_mut(),
                             buffer.as_mut_ptr() as *mut GLchar);

        return String::from_utf8(buffer).expect("Shader info-log couldn't be parsed as utf8");
    }

    fn debug_watch_thread(shader: &Shader, path: String) {
        /*thread::spawn(move || {
            if let Err(e) = Shader::debug_watch(shader, &path.clone()) {
                panic!("Failed to watch {}, error: {:?}", path, e);
            }
        });*/
    }

    fn debug_watch(shader: &Shader, path: &str) -> notify::Result<()> {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher = try!(Watcher::new_raw(tx));

        try!(watcher.watch(path, RecursiveMode::NonRecursive));

        loop {
            match rx.recv() {
                Ok(event) => println!("{:?}", event),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }
}
