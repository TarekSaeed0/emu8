use std::sync::mpsc::{self, Receiver, SyncSender, Sender, SendError, TrySendError, RecvError};
use winit::event::*;

#[repr(u8)]
#[derive(Debug)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
}

pub struct InputSender {
    key_tx: Sender<KeyboardInput>,
    next_key_tx: SyncSender<KeyboardInput>,
}
impl InputSender {
    pub fn send_key_event(&self, key_event: KeyboardInput) -> Result<(), SendError<KeyboardInput>> {
        self.key_tx.send(key_event)?;
        match self.next_key_tx.try_send(key_event) {
            Err(TrySendError::Disconnected(key_event)) => Err(SendError(key_event)),
            _ => Ok(())
        }
    }
}

pub struct InputReceiver {
    key_states: u16,
    key_rx: Receiver<KeyboardInput>,
    next_key_rx: Receiver<KeyboardInput>,
}
impl InputReceiver {
    pub fn recv_key_press(&self) -> Result<Key, RecvError> {
        for key_event in self.next_key_rx.iter() {
            match key_event {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                    ..
                } => {
                    match keycode {
                        VirtualKeyCode::X => {
                            return Ok(Key::Key0);
                        },
                        VirtualKeyCode::Key1 => {
                            return Ok(Key::Key1);
                        },
                        VirtualKeyCode::Key2 => {
                            return Ok(Key::Key2);
                        },
                        VirtualKeyCode::Key3 => {
                            return Ok(Key::Key3);
                        },
                        VirtualKeyCode::Q => {
                            return Ok(Key::Key4);
                        },
                        VirtualKeyCode::W => {
                            return Ok(Key::Key5);
                        },
                        VirtualKeyCode::E => {
                            return Ok(Key::Key6);
                        },
                        VirtualKeyCode::A => {
                            return Ok(Key::Key7);
                        },
                        VirtualKeyCode::S => {
                            return Ok(Key::Key8);
                        },
                        VirtualKeyCode::D => {
                            return Ok(Key::Key9);
                        },
                        VirtualKeyCode::Z => {
                            return Ok(Key::KeyA);
                        },
                        VirtualKeyCode::C => {
                            return Ok(Key::KeyB);
                        },
                        VirtualKeyCode::Key4 => {
                            return Ok(Key::KeyC);
                        },
                        VirtualKeyCode::R => {
                            return Ok(Key::KeyD);
                        },
                        VirtualKeyCode::F => {
                            return Ok(Key::KeyE);
                        },
                        VirtualKeyCode::V => {
                            return Ok(Key::KeyF);
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }
        Err(RecvError)
    }
    pub fn process_key_events(&mut self) {
        for key_event in self.key_rx.try_iter() {
            match key_event {
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                } => {
                    match keycode {
                        VirtualKeyCode::X => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key0 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key0 as u8),
                            }
                        },
                        VirtualKeyCode::Key1 => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key1 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key1 as u8),
                            }
                        },
                        VirtualKeyCode::Key2 => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key2 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key2 as u8),
                            }
                        },
                        VirtualKeyCode::Key3 => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key3 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key3 as u8),
                            }
                        },
                        VirtualKeyCode::Q => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key4 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key4 as u8),
                            }
                        },
                        VirtualKeyCode::W => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key5 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key5 as u8),
                            }
                        },
                        VirtualKeyCode::E => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key6 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key6 as u8),
                            }
                        },
                        VirtualKeyCode::A => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key7 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key7 as u8),
                            }
                        },
                        VirtualKeyCode::S => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key8 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key8 as u8),
                            }
                        },
                        VirtualKeyCode::D => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::Key9 as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::Key9 as u8),
                            }
                        },
                        VirtualKeyCode::Z => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::KeyA as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::KeyA as u8),
                            }
                        },
                        VirtualKeyCode::C => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::KeyB as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::KeyB as u8),
                            }
                        },
                        VirtualKeyCode::Key4 => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::KeyC as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::KeyC as u8),
                            }
                        },
                        VirtualKeyCode::R => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::KeyD as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::KeyD as u8),
                            }
                        },
                        VirtualKeyCode::F => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::KeyE as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::KeyE as u8),
                            }
                        },
                        VirtualKeyCode::V => {
                            match state {
                                ElementState::Pressed => self.key_states |= 1 << Key::KeyF as u8,
                                ElementState::Released => self.key_states &= !(1 << Key::KeyF as u8),
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }
    }
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.key_states & (1 << key as u8) != 0
    }
}

pub fn input() -> (InputSender, InputReceiver) {
    let key_states = 0;

    let (key_tx, key_rx) = mpsc::channel();
    let (next_key_tx, next_key_rx) = mpsc::sync_channel(0);

    let input_tx = InputSender {
        key_tx,
        next_key_tx,
    };
    let input_rx = InputReceiver {
        key_states,
        key_rx,
        next_key_rx,
    };

    (input_tx, input_rx)
}