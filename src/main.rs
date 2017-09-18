#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Device;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];
const BLACK: [f32; 3] = [0.0, 0.0, 0.0];
const WHITE: [f32; 3] = [0.1, 0.2, 0.3];

struct Glyph {
    raw: Vec<u8>,
} impl Glyph {
    pub fn new_empty() -> Glyph {
        Glyph {raw: vec![]}
    }
    pub fn new(code: Vec<u8>) -> Glyph {
        Glyph {raw: code}
    }
    pub fn unpacked(&self) -> Vec<u8> {
        Glyph::unpack(&self.raw)
    }
    pub fn vertices_indices(&self) -> (Vec<Vertex>, Vec<u16>) {
        let bits = self.unpacked();
        let mut ret = (vec![], vec![]);
        for (i, bit) in bits.iter().enumerate() {
            let (vx, ix) = Glyph::square_at(i as i32, if bit == &1u8 {BLACK} else {WHITE});
            ret.0.extend(vx);
            ret.1.extend(ix);
        }
        ret
    }
    fn unpack(bytes: &Vec<u8>) -> Vec<u8> {
        bytes.iter().flat_map(|byte| {
            Glyph::as_bits(&byte)
        }).collect::<Vec<u8>>()
    }
    fn as_bits(byte: &u8) -> Vec<u8> {
        let mut b = byte.clone();
        let mut ret = vec![];
        let mut times = 8;
        while times > 0 {
            if b >= 128u8 {
                ret.push(1u8)
            } else {
                ret.push(0u8)
            }
            b = b << 1;
            times -= 1;
        }
        ret
    }
    fn square_at(i: i32, c: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
        let i = i as u16;
        let x = ((i % 7) as f32) - 3.5;
        let y = ((7 - (i / 7)) as f32) - 3.5;
        let xd = x + 1.0;
        let yd = y - 1.0;
        let f = 3.5;
        (vec![
            Vertex{pos: [ x / f,  y / f], color: c},
            Vertex{pos: [xd / f,  y / f], color: c},
            Vertex{pos: [xd / f, yd / f], color: c},
            Vertex{pos: [ x / f, yd / f], color: c},
        ],vec![4*i, 4*i + 1, 4*i + 2, 4*i + 2, 4*i + 3, 4*i])
    }
}

pub fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("Triangle example".to_string())
        .with_dimensions(1024, 768);
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        include_bytes!("shader/triangle_150.glslv"),
        include_bytes!("shader/triangle_150.glslf"),
        pipe::new()
    ).unwrap();

    let glyph = Glyph {raw: vec![0b10000010u8]};

    let (vertices, indices) = glyph.vertices_indices();
    println!("{:?}, {:?}", &vertices, &indices);
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertices, &*indices);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event: glutin::Event| {
            let glutin::Event::WindowEvent { event, .. } = event;
            match event {
                glutin::WindowEvent::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _)
                | glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(width, height) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                },
                _ => (),
            }
        });

        // draw a frame
        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}