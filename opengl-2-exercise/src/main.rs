#![allow(non_upper_case_globals)]
use gl::{self, types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr}};
use glfw::{self, Context, Key, Action};
use std::{ffi::{CString, c_void}, mem, ptr, sync::mpsc::Receiver, str};

const SCREEN_WIDTH:     u32 = 800;
const SCREEN_HEIGHT:    u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;

void main() {
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}
"#;
const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}
"#;
const FRAGMENT2_SHADER_SOURCE: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);
}
"#;

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

    let triangles: ([f32; 9], [f32; 9]) = (
        [
            0.2,  0.0,  0.0,
            0.5,  0.8,  0.0,
            0.8,  0.0,  0.0,
        ],
        [            
            -0.2,  0.0,  0.0,
            -0.5,  0.8,  0.0,
            -0.8,  0.0,  0.0
        ]
    );

    // let vertices: [f32; 18] = [
    //      0.2,  0.0,  0.0,
    //      0.5,  0.8,  0.0,
    //      0.8,  0.0,  0.0,
    //     -0.2,  0.0,  0.0,
    //     -0.5,  0.8,  0.0,
    //     -0.8,  0.0,  0.0
    // ];

    let (shader_program, vaos) = {
        let shader_program = build_shaders();

        let vaos = build_vaos(&triangles);

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        (shader_program, vaos)
    };

    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            gl::UseProgram(shader_program.0);
            gl::BindVertexArray(vaos[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::UseProgram(shader_program.1);
            gl::BindVertexArray(vaos[1]);
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

fn build_shaders() -> (u32, u32) {
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);
    
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        
        let fragment_shader2 = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag2 = CString::new(FRAGMENT2_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader2, 1, &c_str_frag2.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader2);
    
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        let shader_program2 = gl::CreateProgram();
        gl::AttachShader(shader_program2, vertex_shader);
        gl::AttachShader(shader_program2, fragment_shader2);
        gl::LinkProgram(shader_program2);
    
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(fragment_shader2);

        (shader_program, shader_program2)
    }
}

fn build_vaos(triangles: &([f32; 9], [f32; 9])) -> [u32; 2] {
    unsafe {
        let mut vbos: [u32; 2] = [0; 2];
        let mut vaos: [u32; 2] = [0; 2];
        gl::GenVertexArrays(2, vaos.as_mut_ptr());
        gl::GenBuffers(2, vbos.as_mut_ptr());

        // first triangle
        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            mem::size_of_val(&triangles.0) as GLsizeiptr, 
            &triangles.0[0] as *const f32 as *const c_void, 
            gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        // first triangle
        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            mem::size_of_val(&triangles.1) as GLsizeiptr, 
            &triangles.1[0] as *const f32 as *const c_void, 
            gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        vaos
    }
}