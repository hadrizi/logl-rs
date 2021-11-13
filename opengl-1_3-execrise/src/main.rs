#![allow(non_upper_case_globals)]
use gl::{self, types::{GLfloat, GLsizei, GLsizeiptr}};
use glfw::{self, Context, Key, Action};
use std::{ffi::{CString, c_void}, mem, ptr, sync::mpsc::Receiver};

mod shader;
use shader::Shader;

const SCREEN_WIDTH:     u32 = 800;
const SCREEN_HEIGHT:    u32 = 600;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(
        SCREEN_WIDTH,
        SCREEN_HEIGHT, 
        "dev", 
        glfw::WindowMode::Windowed
    ).expect("Failed to crate GLFW window");
    
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let triangle: [f32; 18] = [            
         0.5,  -0.5,  0.0,  1.0, 0.0, 0.0,
        -0.5,  -0.5,  0.0,  0.0, 1.0, 0.0,
         0.0,   0.5,  0.0,  0.0, 0.0, 1.0
    ];

    // let vertices: [f32; 18] = [
    //      0.2,  0.0,  0.0,
    //      0.5,  0.8,  0.0,
    //      0.8,  0.0,  0.0,
    //     -0.2,  0.0,  0.0,
    //     -0.5,  0.8,  0.0,
    //     -0.8,  0.0,  0.0
    // ];

    let (shader, vao) = {
        let shader = Shader::new(
            "shaders/vertex.glsl",
            "shaders/fragment.glsl"
        );

        let vao = build_vaos(&triangle);

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        (shader, vao)
    };

    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            shader.use_program();
            
            let time_value = glfw.get_time() as f32;
            let green_value = time_value.sin() / 2.0 + 0.5;
            let var_name = CString::new("ourColor").unwrap();
            let vertex_color_location = gl::GetUniformLocation(
                shader.id,
                var_name.as_ptr()
            );
            
            gl::Uniform4f(vertex_color_location, 0.0, green_value, 0.0, 1.0);
            gl::BindVertexArray(vao);

            shader.set_float("offset", 0.5);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            
            // gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(w, h) => {
                unsafe {
                    gl::Viewport(0, 0, w, h);
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}

fn build_vaos(triangle: &[f32; 18]) -> u32 {
    unsafe {
        let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        // first triangle
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            mem::size_of_val(triangle) as GLsizeiptr, 
            &triangle[0] as *const f32 as *const c_void, 
            gl::STATIC_DRAW
        );

        // aPos
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        
        // aCol
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            (3 * mem::size_of::<GLfloat>()) as *const c_void
        );
        gl::EnableVertexAttribArray(1);
        
        vao
    }
}