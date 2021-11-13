#![allow(non_upper_case_globals)]
use gl::{self, types::{GLfloat, GLsizei, GLsizeiptr}};
use glfw::{self, Context, Key, Action};
use image::GenericImage;
use std::{ffi::{CString, c_void}, mem, path::Path, ptr, sync::mpsc::Receiver};

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

    let vertices = vec![
        // coords          // colors         // tex coords 
         0.5,  0.5,  0.0,  1.0,  0.0,  0.0,  1.0,  1.0,     // top right    
         0.5, -0.5,  0.0,  0.0,  1.0,  0.0,  1.0,  0.0,     // bottom right
        -0.5, -0.5,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,     // bottom left
        -0.5,  0.5,  0.0,  1.0,  1.0,  0.0,  0.0,  1.0,     // top left
    ];

    let indices = vec![
        0, 1, 3,
        1, 2, 3,
    ];

    let (shader, vao, texture) = {
        let shader = Shader::new(
            "shaders/vertex.glsl",
            "shaders/fragment.glsl"
        );

        let vao = build_vaos(&vertices, &indices);

        let texture = unsafe {
            let (mut tex1, mut tex2) = (0, 0);

            // first texture
            gl::GenTextures(1, &mut tex1);
            gl::BindTexture(gl::TEXTURE_2D, tex1);
            // wrapping
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // filtering
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            let img = image::open(&Path::new("assets/textures/container.jpg"))
                .expect("failed to load texture");
            let data = img.raw_pixels();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            
            // second texture
            gl::GenTextures(1, &mut tex2);
            gl::BindTexture(gl::TEXTURE_2D, tex2);
            // wrapping
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // filtering
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            let img = image::open(&Path::new("assets/textures/awesomeface.png"))
                .expect("failed to load texture");
            let img = img.flipv();
            let data = img.raw_pixels();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            
            shader.use_program();
            shader.set_int("texture1", 0);
            shader.set_int("texture2", 1);

            (tex1, tex2)
        };

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        (shader, vao, texture)
    };

    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.0);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture.1);
            
            shader.use_program();
            gl::BindVertexArray(vao); 
            // gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());      
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

fn build_vaos(vertices: &Vec<f32>, indices: &Vec<i32>) -> u32 {
    unsafe {
        let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        // first triangle
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            &vertices[0] as *const f32 as *const c_void, 
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER, 
            (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            &indices[0] as *const i32 as *const c_void, 
            gl::STATIC_DRAW
        );

        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        // aPos
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        
        // aCol
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void
        );
        gl::EnableVertexAttribArray(1);

        // aTexCoord
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<GLfloat>()) as *const c_void
        );
        gl::EnableVertexAttribArray(2);
        
        vao
    }
}