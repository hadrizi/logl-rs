use std::{ffi::CString, fs::File, io::Read, ptr};

use cgmath::{Array, Matrix, Matrix4};
use gl::types::{GLchar, GLint};

pub struct Shader {
    pub id: u32
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Shader {
        let mut shader = Shader { id: 0 };

        let (v_src, f_src) = {
            let mut v_shader_file = File::open(vertex_path)
                .unwrap_or_else(|_| panic!("Failed to open {}", vertex_path));
            let mut f_shader_file = File::open(fragment_path)
                .unwrap_or_else(|_| panic!("Failed to open {}", fragment_path));

            let (mut vertex_code, mut fragment_code) = (String::new(), String::new());

            v_shader_file
                .read_to_string(&mut vertex_code)
                .expect("Failed to read vertex shader");
            f_shader_file
                .read_to_string(&mut &mut fragment_code)
                .expect("failed to read fragment shader");
            
            (
                CString::new(vertex_code.as_bytes()).unwrap(),
                CString::new(fragment_code.as_bytes()).unwrap()
            )
        };

        unsafe {
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &v_src.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            shader.check_compile_errors(vertex, "VERTEX");
            
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &f_src.as_ptr(), ptr::null());
            gl::CompileShader(fragment);
            shader.check_compile_errors(vertex, "FRAGMENT");

            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            shader.check_compile_errors(id, "PROGRAM");

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            
            shader.id = id;
        };
        
        shader
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn set_bool(&self, name: &str, val: bool) {
        gl::Uniform1i(
            gl::GetUniformLocation(self.id, self.str2cstr(name).as_ptr()),
            val as i32
        );
    }

    pub unsafe fn set_int(&self, name: &str, val: i32) {
        gl::Uniform1i(
            gl::GetUniformLocation(self.id, self.str2cstr(name).as_ptr()),
            val
        );
    }

    pub unsafe fn set_float(&self, name: &str, val: f32) {
        gl::Uniform1f(
            gl::GetUniformLocation(self.id, self.str2cstr(name).as_ptr()),
            val
        );
    }

    pub unsafe fn set_mat4f(&self, name: &str, val: &Matrix4<f32>){
        gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, self.str2cstr(name).as_ptr()),
        1, 
        gl::FALSE,
        val.as_ptr()
        );
    }
    
    unsafe fn str2cstr(&self, v: &str) -> CString {
        CString::new(v).unwrap()
    }

    unsafe fn check_compile_errors(&self, shader: u32, type_: &str) {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(1024);
        info_log.set_len(1024 - 1);
        if type_ != "PROGRAM" {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == gl::FALSE as GLint {
                gl::GetShaderInfoLog(
                    shader, 
                    1024, 
                    ptr::null_mut(), 
                    info_log.as_mut_ptr() as *mut GLchar
                );
                println!("ERROR::SHADER_COMPILATION_ERROR of type: {}\n{}\n",
                    type_,
                    String::from_utf8_lossy(&info_log)
                );
            }
        } else {
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
            if success == gl::FALSE as GLint {
                gl::GetProgramInfoLog(
                    shader, 
                    1024, 
                    ptr::null_mut(), 
                    info_log.as_mut_ptr() as *mut GLchar
                );
                println!("ERROR::PROGRAM_LINKING_ERROR of type: {}\n{}\n",
                    type_,
                    String::from_utf8_lossy(&info_log)
                );
            }
        }
    }
}