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

    let vertices: [f32; 18] = [
         0.2,  0.0,  0.0,
         0.5,  0.8,  0.0,
         0.8,  0.0,  0.0,
        -0.2,  0.0,  0.0,
        -0.5,  0.8,  0.0,
        -0.8,  0.0,  0.0
    ];
    let indicies = [
        0, 1, 3,
        1, 2, 3
    ];

    let (shader_program, vao) = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512-1);
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                vertex_shader, 
                512, 
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar 
            );
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED {}", str::from_utf8(&info_log).unwrap());
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar
            );
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED {}", str::from_utf8(&info_log).unwrap());
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program, 
                512, 
                ptr::null_mut(), 
                info_log.as_mut_ptr() as *mut GLchar
            );
            println!("ERROR::SHADER::PROGRAM::LINK_FAILED {}", str::from_utf8(&info_log).unwrap());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            mem::size_of_val(&vertices) as GLsizeiptr, 
            &vertices[0] as *const f32 as *const c_void, 
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER, 
            mem::size_of_val(&indicies) as GLsizeiptr, 
            &indicies[0] as *const i32 as *const c_void, 
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

        // unbind VBO and VAO
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        (shader_program, vao)
    };

    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
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
