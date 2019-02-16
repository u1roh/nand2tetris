extern crate gl;
extern crate glutin;
use glutin::dpi::*;
use glutin::GlContext;

mod gl_util {
    fn compile_shader(shader_type: u32, source: &str) -> u32 {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            let src = source.as_bytes();
            let ptr = src.as_ptr();
            let len = src.len() as i32;
            gl::ShaderSource(shader, 1, &ptr as *const *const u8 as *const *const i8, &len as *const i32);
            gl::CompileShader(shader);
            let mut compiled: i32 = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled as *mut i32);
            if compiled == gl::FALSE as i32 {
                let mut buf: [u8; 1024] = [0; 1024];
                let mut len: i32 = 0;
                gl::GetShaderInfoLog(shader, buf.len() as i32, &mut len as *mut i32, buf.as_mut_ptr() as *mut u8 as *mut i8);
                let msg = std::ffi::CStr::from_bytes_with_nul_unchecked(&buf).to_str().unwrap();
                panic!("compilation failed: {}", msg);
            }
            shader
        }
    }

    pub fn create_program(shader_sources: &[(u32, &str)]) -> u32 {
        unsafe {
            let program = gl::CreateProgram();
            for &(shader_type, shader_source) in shader_sources {
                let shader = compile_shader(shader_type, shader_source);
                gl::AttachShader(program, shader);
            }
            gl::LinkProgram(program);
            let mut linked: i32 = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut linked as *mut i32);
            if linked == gl::FALSE as i32 {
                panic!("link failed");
            }
            program
        }
    }
}

static SHADER_V: (u32, &str) = (gl::VERTEX_SHADER, "
#version 150
in vec2 pos;
out vec2 texcod;
void main () {
    gl_Position = vec4(pos, 0, 1);
    texcod = 0.5 * (pos + vec2(1, 1));
}
");

static SHADER_F: (u32, &str) = (gl::FRAGMENT_SHADER, "
#version 150
uniform isampler2D texture;
in vec2 texcod;
out vec4 color;
void main ()
{
    int ix = int(round(texcod.x * 512));
    int iy = int(round(texcod.y * 256));
    int ix_lo = ix % 16;
    int ix_hi = ix / 16;
    int a = texelFetch(texture, ivec2(ix_hi, iy), 0).x;
    bool white = (a & (1 << ix_lo)) != 0;
    color = white ? vec4(1, 1, 1, 1) : vec4(0, 0, 0, 1);
    //color = vec4(ix_hi / 32.0, iy / 256.0, 1, 1);
    //color = vec4(texcod, 1, 1);
}
");

static SCREEN_RECT: [(f32, f32); 4] = [
    (-1.0, -1.0), (1.0, -1.0),
    (-1.0,  1.0), (1.0,  1.0)
];

pub struct Window {
    window: glutin::GlWindow
}

impl Window {
    pub fn new (events_loop: &glutin::EventsLoop) -> Self {
        let gl_window = {
            let window = glutin::WindowBuilder::new()
                .with_title("nand2tetris in Rust")
                .with_dimensions(LogicalSize::new(512.0, 256.0));
            let context = glutin::ContextBuilder::new()
                .with_vsync(true);
            glutin::GlWindow::new(window, context, events_loop).unwrap()
        };

        unsafe {
            gl_window.make_current().unwrap();

            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);

            let mut tex: u32 = 0;
            gl::GenTextures(1, &mut tex as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            let program = gl_util::create_program(&[SHADER_V, SHADER_F]);
            gl::UseProgram(program);
            gl::Uniform1i(0, 0);
        }

        Self{ window: gl_window }
    }

    pub fn resize(&self, size: LogicalSize) {
        let size = size.to_physical(self.window.get_hidpi_factor());
        self.window.resize(size);
        unsafe{ gl::Viewport(0, 0, size.width as i32, size.height as i32); }
    }

    pub fn draw (&self, screen_image: &[i16; 32 * 256]) {
        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R16UI as i32, 32, 256, 0, gl::RED_INTEGER, gl::UNSIGNED_SHORT, screen_image.as_ptr() as *const std::ffi::c_void);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, SCREEN_RECT.as_ptr() as *const std::ffi::c_void);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, SCREEN_RECT.len() as i32);
        }
        self.window.swap_buffers().unwrap();
    }
}