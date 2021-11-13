use std::{ffi::CString, fs::File, io::Read, ptr};

pub struct Shader {
    pub id: u32
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Shader {
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

        let id = unsafe {
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &v_src.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &f_src.as_ptr(), ptr::null());
            gl::CompileShader(fragment);

            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            
            id
        };

        Shader { id }
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

    unsafe fn str2cstr(&self, v: &str) -> CString {
        CString::new(v).unwrap()
    }
}