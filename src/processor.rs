use std::{sync::{Arc, Mutex}, ops::Range, io::{Read, Cursor}};
use crate::{timers::Timers, input::InputReceiver, display::Display};
use rand::Rng;
use timer::{Timer, Guard};
use chrono::Duration;
use byteorder::{ReadBytesExt, BigEndian};

const MEMORY_SIZE: usize = 4096;
const FONT_RANGE: Range<usize> = 0x50..0x9F;
const FONT: [u8; FONT_RANGE.end - FONT_RANGE.start + 1] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const PROGRAM_START: u16 = 0x200;

struct ProcessorState {
    memory: [u8; MEMORY_SIZE],
    program_counter: u16,
    registers: [u8; 16],
    index_register: u16,
    stack: Vec<u16>,
    timers: Timers,
    input: Arc<Mutex<InputReceiver>>,
    display: Arc<Mutex<Display>>,
}
impl ProcessorState {
    fn new(input: Arc<Mutex<InputReceiver>>, display: Arc<Mutex<Display>>, program: &[u8]) -> Self {
        let mut memory = [0; MEMORY_SIZE];
        for (memory_byte, font_byte) in memory[FONT_RANGE].iter_mut().zip(FONT.iter()) {
            *memory_byte = *font_byte;
        }

        Cursor::new(program).read(&mut memory[PROGRAM_START as usize..]).unwrap();

        let program_counter = PROGRAM_START;
     
        let registers = [0; 16];
        let index_register = 0;

        let stack = Vec::new();

        let timers = Timers::new();

        Self {
            memory,
            program_counter,
            registers,
            index_register,
            stack,
            timers,
            input,
            display,
        }
    }
    fn process(&mut self) {
        if let Ok(opcode) = (&self.memory[self.program_counter as usize..]).read_u16::<BigEndian>() {
            self.program_counter += 2;
            match opcode & 0xF000 {
                0x0000 => match opcode {
                    0x00E0 => {
                        println!("display_clear()");
                        let mut display = self.display.lock().unwrap();
                        display.clear();
                    },
                    0x00EE => {
                        println!("return");
                        self.program_counter = self.stack.pop().expect("can't return outside of subroutine");
                    },
                    _ => {
                        println!("call {:#05x}", 0x0FFF & opcode);
                        unimplemented!();
                    },
                }
                0x1000 => {
                    println!("jump {:#05x}", 0x0FFF & opcode);
                    let nnn = 0x0FFF & opcode;
                    self.program_counter = nnn;
                },
                0x2000 => {
                    println!("*({:#05x})()", 0x0FFF & opcode);
                    let nnn = 0x0FFF & opcode;
                    self.stack.push(self.program_counter);
                    self.program_counter = nnn;
                },
                0x3000 => {
                    println!("if (V{:01x} == {:#04x})", (0x0F00 & opcode) >> 8, 0x00FF & opcode);
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let nn = (0x00FF & opcode) as u8;
                    if self.registers[x] == nn {
                        self.program_counter += 2;
                    }
                },
                0x4000 => {
                    println!("if (V{:01x} != {:#04x})", (0x0F00 & opcode) >> 8, 0x00FF & opcode);
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let nn = (0x00FF & opcode) as u8;
                    if self.registers[x] != nn {
                        self.program_counter += 2;
                    }
                },
                0x5000 => match opcode & 0x000F { 
                    0x0000 => {
                        println!("if (V{:01x} == V{:01x})", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        if self.registers[x] == self.registers[y] {
                            self.program_counter += 2;
                        }
                    },
                    _ => panic!("unknown opcode {:#06x}", opcode),
                },
                0x6000 => {
                    println!("V{:01x} = {:#04x}", (0x0F00 & opcode) >> 8, 0x00FF & opcode);
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let nn = (0x00FF & opcode) as u8;
                    self.registers[x] = nn;
                },
                0x7000 => {
                    println!("V{:01x} += {:#04x}", (0x0F00 & opcode) >> 8, 0x00FF & opcode);
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let nn = (0x00FF & opcode) as u8;
                    self.registers[x] = self.registers[x].wrapping_add(nn);
                },
                0x8000 => match opcode & 0x000F { 
                    0x0000 => {
                        println!("V{:01x} = V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        self.registers[x] = self.registers[y];
                    },
                    0x0001 => {
                        println!("V{:01x} |= V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        self.registers[x] |= self.registers[y];
                    },
                    0x0002 => {
                        println!("V{:01x} &= V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        self.registers[x] &= self.registers[y];
                    },
                    0x0003 => {
                        println!("V{:01x} ^= V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        self.registers[x] ^= self.registers[y];
                    },
                    0x0004 => {
                        println!("V{:01x} += V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        self.registers[0xF] = self.registers[x].checked_add(self.registers[y]).is_none() as u8;
                        self.registers[x] = self.registers[x].wrapping_add(self.registers[y]);
                    },
                    0x0005 => {
                        println!("V{:01x} -= V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        self.registers[0xF] = self.registers[x].checked_sub(self.registers[y]).is_some() as u8;
                        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
                    },
                    0x0006 => {
                        println!("V{:01x} >>= 1", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        self.registers[0xF] = self.registers[x] & 0b00000001;
                        self.registers[x] >>= 1;
                    },
                    0x0007 => {
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        println!("V{:01x} = V{:01x} - V{:01x}", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4, (0x0F00 & opcode) >> 8);
                        self.registers[0xF] = self.registers[y].checked_sub(self.registers[x]).is_some() as u8;
                        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
                    },
                    0x000E => {
                        println!("V{:01x} <<= 1", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        self.registers[0xF] = self.registers[x] & 0b10000000;
                        self.registers[x] <<= 1;
                    },
                    _ => panic!("unknown opcode {:#06x}", opcode),
                },
                0x9000 => match opcode & 0x000F {
                    0x0000 => { 
                        println!("if (V{:01x} != V{:01x})", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let y = ((0x00F0 & opcode) >> 4) as usize;
                        if self.registers[x] != self.registers[y] {
                            self.program_counter += 2;
                        }
                    },
                    _ => panic!("unknown opcode {:#06x}", opcode),
                },
                0xA000 => {
                    println!("I = {:#05x}", 0x0FFF & opcode);
                    let nnn = 0x0FFF & opcode;
                    self.index_register = nnn;
                },
                0xB000 => {
                    println!("PC = V0 + {:#05x}", 0x0FFF & opcode);
                    let nnn = 0x0FFF & opcode;
                    self.program_counter = self.registers[0] as u16 + nnn;
                },
                0xC000 => {
                    println!("V{:01x} = rand() & {:#04x}", (0x0F00 & opcode) >> 8, 0x00FF & opcode);
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let nn = (0x00FF & opcode) as u8;
                    self.registers[x] = rand::thread_rng().gen_range(0..=u8::MAX) & nn;
                },
                0xD000 => {
                    println!("draw(V{:01x}, V{:01x}, {:#04x})", (0x0F00 & opcode) >> 8,  (0x00F0 & opcode) >> 4, 0x000F & opcode);
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let y = ((0x00F0 & opcode) >> 4) as usize;
                    let sprite = &self.memory[self.index_register as usize..(self.index_register + (0x000F & opcode)) as usize];
                    self.registers[0xF] = self.display.lock().unwrap().draw(self.registers[x] as usize, self.registers[y] as usize, sprite) as u8;
                },
                0xE000 => match opcode & 0x00FF { 
                    0x009E => {
                        println!("if (key() == V{:01x})", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let input = self.input.lock().unwrap();
                        let key = unsafe { std::mem::transmute(self.registers[x]) };
                        if input.is_key_pressed(key) {
                            self.program_counter += 2;
                        }
                    },
                    0x00A1 => {
                        println!("if (key() != V{:01x})", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let input = self.input.lock().unwrap();
                        let key = unsafe { std::mem::transmute(self.registers[x]) };
                        if !input.is_key_pressed(key) {
                            self.program_counter += 2;
                        }
                    },
                    _ => panic!("unknown opcode {:#06x}", opcode),
                },
                0xF000 => match opcode & 0x00FF { 
                    0x0007 => {
                        println!("V{:01x} = get_delay()", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let delay_timer = *self.timers.delay_timer.lock().unwrap();
                        self.registers[x] = delay_timer;
                    },
                    0x000A => {
                        println!("V{:01x} = get_key()", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let input = self.input.lock().unwrap();
                        let key = input.recv_key_press().unwrap() as u8;
                        self.registers[x] = key;
                    },
                    0x0015 => {
                        println!("delay_timer(V{:01x})", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let mut delay_timer = self.timers.delay_timer.lock().unwrap();
                        *delay_timer = self.registers[x];
                    },
                    0x0018 => {
                        println!("sound_timer(V{:01x})", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let mut sound_timer = self.timers.sound_timer.lock().unwrap();
                        *sound_timer = self.registers[x];
                    },
                    0x001E => {
                        println!("I += V{:01x}", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        self.index_register = self.index_register.wrapping_add(self.registers[x] as u16);
                    },
                    0x0029 => {
                        println!("I = sprite_addr[V{:01x}]", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        self.index_register = FONT_RANGE.start as u16 + 5 * self.registers[x] as u16;
                    },
                    0x0033 => { 
                        println!("set_BCD(V{:01x})", (0x0F00 & opcode) >> 8);
                        println!("*(I+0) = BCD(3)");
                        println!("*(I+1) = BCD(2)");
                        println!("*(I+2) = BCD(1)");
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        self.memory[self.index_register as usize] = self.registers[x] / 100 % 10;
                        self.memory[self.index_register as usize + 1] = self.registers[x] / 10 % 10;
                        self.memory[self.index_register as usize + 2] = self.registers[x] % 10;
                    },
                    0x0055 => { 
                        println!("reg_dump(V{:01x}, &I)", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let memory_iter = (&mut self.memory[self.index_register as usize..=self.index_register as usize + x]).iter_mut();
                        let register_iter = self.registers.iter();
                        for (memory, register) in memory_iter.zip(register_iter) {
                            *memory = *register;
                        }
                    },
                    0x0065 => { 
                        println!("reg_load(V{:01x}, &I)", (0x0F00 & opcode) >> 8);
                        let x = ((0x0F00 & opcode) >> 8) as usize;
                        let register_iter = self.registers.iter_mut();
                        let memory_iter = (&self.memory[self.index_register as usize..=self.index_register as usize + x]).iter();
                        for (register, memory) in register_iter.zip(memory_iter) {
                            *register = *memory;
                        }
                    },
                    _ => panic!("unknown opcode {:#06x}", opcode),
                },
                _ => panic!("unknown opcode {:#06x}", opcode)
            }
        }
    }
}
pub struct Processor {
    _timer: Timer,
    _guard: Guard,
}
impl Processor {
    pub fn new(input: Arc<Mutex<InputReceiver>>, display: Arc<Mutex<Display>>, program: &[u8]) -> Self {
        let mut state = ProcessorState::new(input, display, program);

        let _timer = timer::Timer::new();
        let _guard = _timer.schedule_repeating(Duration::microseconds(1), move || {
            state.process();
        });

        Self {
            _timer,
            _guard,
        }
    }
}