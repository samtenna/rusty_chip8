use rand::random;

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
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; NUM_V_REGISTERS],
    index_register: u16,
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
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
        self.index_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
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
            // NOP - no operation
            (0, 0, 0, 0) => return,
            // CLS - clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // RET - return from subroutine
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop();
                self.pc = return_address;
            }
            // JMP nnn - jump
            (1, _, _, _) => {
                let address = op & 0x0FFF;
                self.pc = address;
            }
            // CALL nnn - call subroutine
            (2, _, _, _) => {
                let address = op & 0x0FFF;
                self.push(self.pc);
                self.pc = address;
            }
            // SKIP VX == NN - skip next if VX == VN
            (3, _, _, _) => {
                let vx = digit_two as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_registers[vx] == nn {
                    // instruction length is 2 bytes
                    self.pc += 2;
                }
            }
            // SKIP VX != NN - skip next if VX != VN
            (4, _, _, _) => {
                let vx = digit_two as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_registers[vx] != nn {
                    self.pc += 2;
                }
            }
            // SKIP VX == VY - skip next if VX == VY
            (5, _, _, 0) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                if self.v_registers[vx] == self.v_registers[vy] {
                    self.pc += 2;
                }
            }
            // VX = VN - set VX -> NN
            (6, _, _, _) => {
                let vx = digit_two as usize;
                let nn = (op & 0x00FF) as u8;

                self.v_registers[vx] = nn;
            }
            // VX += NN - set VX -> VX + NN
            (7, _, _, _) => {
                let vx = digit_two as usize;
                let nn = (op & 0x00FF) as u8;

                self.v_registers[vx] = self.v_registers[vx].wrapping_add(nn);
            }
            // VX = VY - set VX -> VY
            (8, _, _, 0) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                self.v_registers[vx] = self.v_registers[vy];
            }
            // VX |= VY
            (8, _, _, 1) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                self.v_registers[vx] |= self.v_registers[vy];
            }
            // VX &= VY
            (8, _, _, 2) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                self.v_registers[vx] &= self.v_registers[vy];
            }
            // VX ^= VY
            (8, _, _, 3) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                self.v_registers[vx] ^= self.v_registers[vy];
            }
            // VX += VY - VX -> VX + VY
            (8, _, _, 4) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                let (result, overflow) = self.v_registers[vx].overflowing_add(self.v_registers[vy]);

                // set carry flag
                self.v_registers[0xF] = if overflow { 1 } else { 0 };
                self.v_registers[vx] = result;
            }
            // VX -= VY - VX -> VX - VY
            (8, _, _, 5) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                let (result, underflow) =
                    self.v_registers[vx].overflowing_sub(self.v_registers[vy]);

                self.v_registers[0xF] = if underflow { 0 } else { 1 };
                self.v_registers[vx] = result;
            }
            // VX >> 1
            (8, _, _, 6) => {
                let vx = digit_two as usize;
                // the flag register is set to the LSB
                let rightmost_bit = self.v_registers[vx] & 1;

                self.v_registers[vx] >>= 1;
                self.v_registers[0xF] = rightmost_bit;
            }
            // VX = VY - VX
            (8, _, _, 7) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                let (result, underflow) =
                    self.v_registers[vy].overflowing_sub(self.v_registers[vx]);

                self.v_registers[0xF] = if underflow { 0 } else { 1 };
                self.v_registers[vx] = result;
            }
            // VX << 1
            (8, _, _, 0xE) => {
                let vx = digit_two as usize;
                // NOTE: may need & 1
                let leftmost_bit = self.v_registers[vx] >> 7;

                self.v_registers[vx] <<= 1;
                self.v_registers[0xF] = leftmost_bit;
            }
            // SKIP VX != VY
            (9, _, _, 0) => {
                let vx = digit_two as usize;
                let vy = digit_three as usize;

                if self.v_registers[vx] != self.v_registers[vy] {
                    self.pc += 2;
                }
            }
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;

                self.index_register = nnn;
            }
            // JUMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0x0FFF;

                self.pc = self.v_registers[0] as u16 + nnn;
            }
            // VX = RAND() & NN
            (0xC, _, _, _) => {
                let vx = digit_two as usize;
                let nn = (op & 0x00FF) as u8;
                let rng: u8 = random();

                self.v_registers[vx] = rng & nn;
            }
            // DRAW
            (0xD, _, _, _) => {
                let draw_x = self.v_registers[digit_two as usize] as u16;
                let draw_y = self.v_registers[digit_three as usize] as u16;
                let height = digit_four;

                let mut pixels_flipped = false;

                for current_y in 0..height {
                    let address = self.index_register + current_y as u16;
                    let row_pixels = self.memory[address as usize];

                    for current_x in 0..8 {
                        if (row_pixels & (0b1000_0000 >> current_x)) != 0 {
                            let x = (draw_x + current_x) as usize % SCREEN_WIDTH;
                            let y = (draw_y + current_y) as usize % SCREEN_HEIGHT;

                            let index = x + SCREEN_WIDTH * y;

                            pixels_flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                self.v_registers[0xF] = if pixels_flipped { 1 } else { 0 };
            }
            // SKIP IF KEY PRESSED
            (0xE, _, 9, 0xE) => {
                let vx = digit_two as usize;
                let key_pressed = self.keys[self.v_registers[vx] as usize];

                if key_pressed {
                    self.pc += 2;
                }
            }
            // SKIP IF KEY NOT PRESSED
            (0xE, _, 0xA, 1) => {
                let vx = digit_two as usize;
                let key_pressed = self.keys[self.v_registers[vx] as usize];

                if !key_pressed {
                    self.pc += 2;
                }
            }
            // VX = DT
            (0xF, _, 0, 7) => {
                let vx = digit_two as usize;

                self.v_registers[vx] = self.delay_timer;
            }
            // WAIT FOR KEY PRESS
            (0xF, _, 0, 0xA) => {
                let vx = digit_two as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_registers[vx] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            // DT = VX
            (0xF, _, 1, 5) => {
                let vx = digit_two as usize;

                self.delay_timer = self.v_registers[vx];
            }
            // ST = VX
            (0xF, _, 1, 8) => {
                let vx = digit_two as usize;

                self.sound_timer = self.v_registers[vx];
            }
            // I += VX
            (0xF, _, 1, 0xE) => {
                let vx = digit_two as usize;

                self.index_register = self
                    .index_register
                    .wrapping_add(self.v_registers[vx] as u16);
            }
            // I = FONT
            (0xF, _, 2, 9) => {
                let vx = digit_two as usize;
                let char = self.v_registers[vx] as u16;

                self.index_register = char * 5;
            }
            // BCD
            (0xF, _, 3, 3) => {
                let vx = digit_two as usize;
                let mut vx_value = self.v_registers[vx] as f32;

                let hundreds = (vx_value / 100.0).floor() as u8;
                vx_value %= 100.0;
                let tens = (vx_value / 10.0).floor() as u8;
                vx_value %= 10.0;
                let ones = vx_value.floor() as u8;

                self.memory[self.index_register as usize] = hundreds;
                self.memory[(self.index_register + 1) as usize] = tens;
                self.memory[(self.index_register + 2) as usize] = ones;
            }
            // STORE V0 - VX
            (0xF, _, 5, 5) => {
                let vx = digit_two as usize;
                let memory_start = self.index_register as usize;

                for i in 0..=vx as usize {
                    self.memory[memory_start + i] = self.v_registers[i];
                }
            }
            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                let vx = digit_two as usize;
                let memory_start = self.index_register as usize;

                for i in 0..=vx as usize {
                    self.v_registers[i] = self.memory[memory_start + i];
                }
            }
            (_, _, _, _) => panic!("unknown opcode: {:#x}", op),
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

        if self.stack_pointer > STACK_SIZE as u16 {
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

    // operations

    #[test]
    fn test_cls() {
        let mut cpu = CPU::new();

        cpu.screen = [true; SCREEN_WIDTH * SCREEN_HEIGHT];
        cpu.execute(0x00E0);
        assert_eq!(cpu.screen, [false; SCREEN_WIDTH * SCREEN_HEIGHT]);
    }

    #[test]
    fn test_ret() {
        let mut cpu = CPU::new();

        cpu.push(0x69);
        cpu.execute(0x00EE);
        assert_eq!(cpu.pc, 0x69);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::new();

        cpu.execute(0x1420);
        assert_eq!(cpu.pc, 0x420);
    }

    #[test]
    fn test_call() {
        let mut cpu = CPU::new();

        cpu.pc = 0x69;
        cpu.execute(0x2420);
        assert_eq!(cpu.pop(), 0x69);
        assert_eq!(cpu.pc, 0x420);
    }

    #[test]
    fn test_skip_vx_equal_nn() {
        let mut cpu = CPU::new();

        cpu.v_registers[5] = 0x69;
        cpu.execute(0x3569);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
        cpu.execute(0x3570);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
    }

    #[test]
    fn test_skip_vx_not_equal_nn() {
        let mut cpu = CPU::new();

        cpu.v_registers[5] = 0x69;
        cpu.execute(0x3570);
        assert_eq!(cpu.pc, START_ADDRESS);
        cpu.execute(0x3569);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
    }

    #[test]
    fn test_skip_vx_equal_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 0x69;
        cpu.v_registers[15] = 0x69;
        cpu.execute(0x50F0);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
        cpu.execute(0x5010);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
    }

    #[test]
    fn test_set_vx_to_nn() {
        let mut cpu = CPU::new();

        cpu.execute(0x6769);
        assert_eq!(cpu.v_registers[7], 0x69);
    }

    #[test]
    fn test_add_nn_to_vx() {
        let mut cpu = CPU::new();

        cpu.v_registers[3] = 255;
        cpu.execute(0x7302);
        assert_eq!(cpu.v_registers[3], 1);
    }

    #[test]
    fn test_vx_or_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[5] = 0b1010_1010;
        cpu.v_registers[0xA] = 0b0101_0101;
        cpu.execute(0x85A1);
        assert_eq!(cpu.v_registers[5], 0xFF);
    }

    #[test]
    fn test_vx_and_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[8] = 0b1010_1010;
        cpu.v_registers[2] = 0b0101_0101;
        cpu.execute(0x8822);
        assert_eq!(cpu.v_registers[8], 0x00);
    }

    #[test]
    fn test_vx_xor_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[0xF] = 0b1110_1110;
        cpu.v_registers[0] = 0b0111_0111;
        cpu.execute(0x8F03);
        assert_eq!(cpu.v_registers[0xF], 0b1001_1001);
    }

    #[test]
    fn test_vx_add_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 255;
        cpu.v_registers[1] = 1;
        cpu.execute(0x8014);
        assert_eq!(cpu.v_registers[0], 0);
        assert_eq!(cpu.v_registers[0xF], 1);

        cpu.v_registers[6] = 10;
        cpu.v_registers[0xA] = 10;
        cpu.execute(0x86A4);
        assert_eq!(cpu.v_registers[6], 20);
        assert_eq!(cpu.v_registers[0xF], 0);
    }

    #[test]
    fn test_vx_sub_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 0;
        cpu.v_registers[1] = 1;
        cpu.execute(0x8015);
        assert_eq!(cpu.v_registers[0], 255);
        assert_eq!(cpu.v_registers[0xF], 0);

        cpu.v_registers[6] = 10;
        cpu.v_registers[0xA] = 10;
        cpu.execute(0x86A5);
        assert_eq!(cpu.v_registers[6], 0);
        assert_eq!(cpu.v_registers[0xF], 1);
    }

    #[test]
    fn test_vx_shift_right() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 0b0101_0101;
        cpu.execute(0x8006);
        assert_eq!(cpu.v_registers[0], 0b0010_1010);
        assert_eq!(cpu.v_registers[0xF], 1);

        cpu.v_registers[0xB] = 0b1010_1010;
        cpu.execute(0x8B06);
        assert_eq!(cpu.v_registers[0xB], 0b0101_0101);
        assert_eq!(cpu.v_registers[0xF], 0);
    }

    #[test]
    fn test_vx_to_vy_minus_vx() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 1;
        cpu.execute(0x8017);
        assert_eq!(cpu.v_registers[0], 255);
        assert_eq!(cpu.v_registers[0xF], 0);

        cpu.v_registers[0] = 0;
        cpu.v_registers[1] = 1;
        cpu.execute(0x8017);
        assert_eq!(cpu.v_registers[0], 1);
        assert_eq!(cpu.v_registers[0xF], 1);
    }

    #[test]
    fn test_vx_shift_left() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 0b1010_1010;
        cpu.execute(0x800E);
        assert_eq!(cpu.v_registers[0], 0b0101_0100);
        assert_eq!(cpu.v_registers[0xF], 1);

        cpu.v_registers[0] = 0b0101_0101;
        cpu.execute(0x800E);
        assert_eq!(cpu.v_registers[0], 0b1010_1010);
        assert_eq!(cpu.v_registers[0xF], 0);
    }

    #[test]
    fn test_skip_vx_not_equal_vy() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 1;
        cpu.execute(0x9010);
        assert_eq!(cpu.pc, START_ADDRESS + 2);

        cpu.v_registers[0] = 0;
        cpu.execute(0x9010);
        assert_eq!(cpu.pc, START_ADDRESS + 2)
    }

    #[test]
    fn test_set_i_nnn() {
        let mut cpu = CPU::new();

        cpu.execute(0xA420);
        assert_eq!(cpu.index_register, 0x420);
    }

    #[test]
    fn test_jump_v0_nnn() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 69;
        cpu.execute(0xB420);
        assert_eq!(cpu.pc, 69 + 0x420);
    }

    // NOTE: test for the random opcode is omitted on account of the fact it's random

    #[test]
    fn test_draw() {
        let mut cpu = CPU::new();

        // draws a plus
        // 01000000
        // 11100000
        // 01000000
        let sprite = [0x40, 0xE0, 0x40];
        cpu.memory[(START_ADDRESS + 4) as usize..(START_ADDRESS + 7) as usize]
            .copy_from_slice(&sprite);
        cpu.v_registers[0] = 10;
        cpu.v_registers[1] = 10;
        cpu.index_register = START_ADDRESS + 4;
        cpu.execute(0xD013);

        assert_eq!(cpu.screen[650], false);
        assert_eq!(cpu.screen[651], true);
        assert_eq!(cpu.screen[652], false);

        assert_eq!(cpu.screen[714], true);
        assert_eq!(cpu.screen[715], true);
        assert_eq!(cpu.screen[716], true);

        assert_eq!(cpu.screen[778], false);
        assert_eq!(cpu.screen[779], true);
        assert_eq!(cpu.screen[780], false);
    }

    #[test]
    fn test_skip_key_pressed() {
        let mut cpu = CPU::new();

        cpu.v_registers[0xA] = 2;
        cpu.keys[2] = true;
        cpu.execute(0xEA9E);
        assert_eq!(cpu.pc, START_ADDRESS + 2);

        cpu.keys[2] = false;
        cpu.execute(0xEA9E);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
    }

    #[test]
    fn test_skip_key_not_pressed() {
        let mut cpu = CPU::new();

        cpu.v_registers[0xA] = 2;
        cpu.keys[2] = false;
        cpu.execute(0xEA9E);
        assert_eq!(cpu.pc, START_ADDRESS);

        cpu.keys[2] = true;
        cpu.execute(0xEA9E);
        assert_eq!(cpu.pc, START_ADDRESS + 2);
    }

    #[test]
    fn test_set_vx_dt() {
        let mut cpu = CPU::new();

        cpu.delay_timer = 69;
        cpu.execute(0xF407);
        assert_eq!(cpu.v_registers[4], 69);
    }

    #[test]
    fn test_wait_for_key() {
        let mut cpu = CPU::new();

        cpu.keys[0xD] = true;
        cpu.execute(0xF80A);
        assert_eq!(cpu.v_registers[8], 0xD);

        // TODO: can't test the waiting functionality in this way, requires multiple cycles - change
    }

    #[test]
    fn test_set_dt_vx() {
        let mut cpu = CPU::new();

        cpu.v_registers[0xE] = 42;
        cpu.execute(0xFE15);
        assert_eq!(cpu.delay_timer, 42);
    }

    #[test]
    fn test_set_st_vx() {
        let mut cpu = CPU::new();

        cpu.v_registers[0xE] = 42;
        cpu.execute(0xFE18);
        assert_eq!(cpu.sound_timer, 42);
    }

    #[test]
    fn test_i_add_vx() {
        let mut cpu = CPU::new();

        cpu.v_registers[0xB] = 9;
        cpu.index_register = 10;
        cpu.execute(0xFB1E);
        assert_eq!(cpu.index_register, 19);
    }

    #[test]
    fn test_set_i_font_location() {
        let mut cpu = CPU::new();

        cpu.v_registers[2] = 7;
        cpu.execute(0xF229);
        assert_eq!(cpu.index_register, 7 * 5);
    }

    #[test]
    fn test_bcd() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 123;
        cpu.index_register = 69;
        cpu.execute(0xF033);
        assert_eq!(cpu.memory[69], 1);
        assert_eq!(cpu.memory[70], 2);
        assert_eq!(cpu.memory[71], 3);
    }

    #[test]
    fn test_store_v0_vx() {
        let mut cpu = CPU::new();

        cpu.v_registers[0] = 1;
        cpu.v_registers[1] = 2;
        cpu.v_registers[2] = 3;
        cpu.index_register = START_ADDRESS + 10;
        cpu.execute(0xF255);
        assert_eq!(cpu.memory[(START_ADDRESS + 10) as usize], 1);
        assert_eq!(cpu.memory[(START_ADDRESS + 11) as usize], 2);
        assert_eq!(cpu.memory[(START_ADDRESS + 12) as usize], 3);
    }

    #[test]
    fn test_load_v0_vx() {
        let mut cpu = CPU::new();

        cpu.memory[(START_ADDRESS + 10) as usize] = 1;
        cpu.memory[(START_ADDRESS + 11) as usize] = 2;
        cpu.memory[(START_ADDRESS + 12) as usize] = 3;
        cpu.index_register = START_ADDRESS + 10;
        cpu.execute(0xF265);
        assert_eq!(cpu.v_registers[0], 1);
        assert_eq!(cpu.v_registers[1], 2);
        assert_eq!(cpu.v_registers[2], 3);
    }
}
