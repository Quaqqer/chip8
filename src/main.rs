mod chip8;

use chip8::Chip8;
use game_loop::{
    game_loop,
    winit::{
        dpi::{LogicalSize, PhysicalSize, Size},
        event::{ElementState, Event, VirtualKeyCode, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
    },
};
use pixels::{Pixels, SurfaceTexture};

/// The struct used for the game loop, needs data for the emulator and display
struct Game {
    chip8: Chip8,
    pixels: Pixels,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let rom = std::fs::read(
        args.get(1)
            .expect("No rom passed, needs a chip-8 rom path as a second argument"),
    )
    .expect("Could not read the rom passed, maybe you passed a wrong path?");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("CHIP8 Emulator")
        .with_min_inner_size(Size::Logical(LogicalSize {
            width: 64.,
            height: 32.,
        }))
        .with_inner_size(Size::Logical(LogicalSize {
            width: 640.,
            height: 320.,
        }))
        .build(&event_loop)
        .expect("Could not create a window");

    let chip8 = Chip8::new(rom);

    let surface_texture = SurfaceTexture::new(640, 320, &window);
    let pixels =
        Pixels::new(64, 32, surface_texture).expect("Could not instantiate Pixels library");

    let game = Game { chip8, pixels };

    game_loop(
        event_loop,
        window,
        game,
        500,
        0.1,
        |g| {
            if g.number_of_updates() % 8 == 0 {
                g.game.chip8.decrease_timers();
            };

            g.game.chip8.cycle();
        },
        |g| {
            let frame = g.game.pixels.frame_mut();

            for x in 0..64 {
                for y in 0..32 {
                    let set = g.game.chip8.display()[y * 64 + x];

                    let color_on: [u8; 4] = [0x89, 0xB4, 0xFA, 0xFF];
                    let color_off: [u8; 4] = [0x1E, 0x1E, 0x2E, 0xFF];

                    let pixel_i = (y * 64 + x) * 4;
                    frame[pixel_i..pixel_i + 4].copy_from_slice(if set {
                        &color_on
                    } else {
                        &color_off
                    });
                }
            }

            if let Err(err) = g.game.pixels.render() {
                eprintln!("Render error: {}", err);
                panic!();
            }
        },
        |g, e| match e {
            Event::WindowEvent {
                window_id,
                event: window_event,
            } if *window_id == g.window.id() => match window_event {
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    g.game
                        .pixels
                        .resize_surface(*width, *height)
                        .expect("Could not resize surface");
                }
                WindowEvent::CloseRequested => g.exit(),
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(k) = input.virtual_keycode {
                        if let Some(hex) = key_to_hex(k) {
                            match input.state {
                                ElementState::Pressed => g.game.chip8.down(hex),
                                ElementState::Released => g.game.chip8.up(hex),
                            }
                        }
                    }
                }
                _ => (),
            },

            _ => (),
        },
    );
}

/// Map keys to numbers, uses the same layout as chip8 but around rows 1234, qwer, asdf, and zxcv.
fn key_to_hex(virtual_key_code: VirtualKeyCode) -> Option<u8> {
    match virtual_key_code {
        VirtualKeyCode::Key1 => Some(0x1),
        VirtualKeyCode::Key2 => Some(0x2),
        VirtualKeyCode::Key3 => Some(0x3),
        VirtualKeyCode::Key4 => Some(0xc),
        VirtualKeyCode::Q => Some(0x4),
        VirtualKeyCode::W => Some(0x5),
        VirtualKeyCode::E => Some(0x6),
        VirtualKeyCode::R => Some(0xd),
        VirtualKeyCode::A => Some(0x7),
        VirtualKeyCode::S => Some(0x8),
        VirtualKeyCode::D => Some(0x9),
        VirtualKeyCode::F => Some(0xe),
        VirtualKeyCode::Z => Some(0xa),
        VirtualKeyCode::X => Some(0x0),
        VirtualKeyCode::C => Some(0xb),
        VirtualKeyCode::V => Some(0xf),
        _ => None,
    }
}
