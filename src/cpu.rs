pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const MEMORY_SIZE: usize = 4096;
const NUM_V_REGISTERS: usize = 16;
// stack size is not in the Chip8 specification
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
// the first 512 bytes were originally for the interpreter, no program should use them
const START_ADDRESS: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct CPU {
    pc: u16,
    memory: [u8; MEMORY_SIZE],
    // pixels don't have colours, they are either on or off
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; NUM_V_REGISTERS],
    index_register: u16,
    stack: [u16; STACK_SIZE],
    // NOTE: change this if it needs to be a u16 but I don't see why u8 wouldn't work
    stack_pointer: u8,
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            pc: START_ADDRESS,
            memory: [0; MEMORY_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; NUM_V_REGISTERS],
            index_register: 0,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        cpu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        cpu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.memory = [0; MEMORY_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; NUM_V_REGISTERS];
        self.index_register= 0;
        self.stack_pointer= 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer= 0;
        self.sound_timer= 0;
        
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.memory[self.pc as usize] as u16;
        let lower_byte = self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 1;
        (higher_byte << 8) | lower_byte
    }

    fn execute(&mut self, op: u16) {
        let digit_one = (op & 0xF000) >> 12;
        let digit_two = (op & 0x0F00) >> 8;
        let digit_three = (op & 0x00F0) >> 4;
        let digit_four = (op & 0x000F);

        match (digit_one, digit_two, digit_three, digit_four) {
            (_, _, _, _) => todo!()
        }
    }

    fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }
            
            self.sound_timer -= 1;
        }
    }

    // Stack Operations

    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;

        if self.stack_pointer > STACK_SIZE as u8 {
            panic!("stack overflow... HE SAID THE THING");
        }
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut cpu = CPU::new();

        cpu.push(1);
        assert_eq!(cpu.stack[0], 1);
        assert_eq!(cpu.pop(), 1);

        for i in 0..10 {
            cpu.push(i);
        }
        assert_eq!(cpu.stack[5], 5);
        for _ in 0..9 {
            cpu.pop();
        }
        assert_eq!(cpu.stack[0], 0);
    }
}