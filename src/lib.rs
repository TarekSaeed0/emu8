mod processor;
mod display;
mod timers;
mod input;

use std::sync::{Mutex, Arc};
use display::Display;
use processor::Processor;
use winit::{window::WindowBuilder, dpi::{Size, LogicalSize}, event::*, event_loop::{EventLoop, ControlFlow}};

pub async fn run(program: &[u8]) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Emu8")
        .with_inner_size(Size::Logical(LogicalSize { width: 640.0, height: 320.0 }))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let display = Arc::new(Mutex::new(Display::new(&window).await));

    let (input_tx, input_rx) = input::input();
    let input_rx = Arc::new(Mutex::new(input_rx));

    let _processor = Processor::new(input_rx.clone(), display.clone(), program);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit
                },
                WindowEvent::Resized(physical_size) => {
                    display.lock().unwrap().resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    display.lock().unwrap().resize(**new_inner_size);
                },
                WindowEvent::KeyboardInput {
                    input: key_event,
                    ..
                } => {
                    input_tx.send_key_event(*key_event).unwrap();
                    if let Ok(mut input_rx) = input_rx.try_lock() {
                        input_rx.process_key_events();
                    }
                }
                _ => {},
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            display.lock().unwrap().update();
            match display.lock().unwrap().render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => display.lock().unwrap().resize(display.lock().unwrap().size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(err) => eprintln!("{:?}", err),
            }
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        _ => {},
    });
}