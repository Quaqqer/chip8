use chip8::Chip8;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::{LogicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod chip8;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(Size::Logical(LogicalSize {
            width: 640.,
            height: 320.,
        }))
        .build(&event_loop)
        .unwrap();

    let surface_texture = SurfaceTexture::new(640, 320, &window);
    let mut pixels = Pixels::new(64, 32, surface_texture).unwrap();

    let rom = std::fs::read("IBM Logo.ch8").unwrap();

    let mut chip8 = Chip8::new(rom);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();

                for x in 0..64 {
                    for y in 0..32 {
                        let set = chip8.display[y * 64 + x];
                        let color = if set {0xff} else {0x00};
                        frame[(y * 64 + x) * 4 + 0] = color;
                        frame[(y * 64 + x) * 4 + 1] = color;
                        frame[(y * 64 + x) * 4 + 2] = color;
                        frame[(y * 64 + x) * 4 + 3] = 0xff;
                    }
                }

                if let Err(err) = pixels.render() {
                    eprintln!("pixels.render {:?}", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,

            _ => (),
        }

        chip8.cycle();
        window.request_redraw();
    })
}
