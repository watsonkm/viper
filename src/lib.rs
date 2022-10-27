mod utils;

use crate::utils::set_panic_hook;

use rand::prelude::*;
use bitvec::prelude::*;
use wasm_bindgen::prelude::*;

const MEM_SIZE: usize = 4096;       // bytes
const DISP_WIDTH: u16 = 64;         // pixels
const DISP_HEIGHT: u16 = 32;        // pixels
const VAR_REG_COUNT: usize = 16;    // registers
const START_ADDR: u16 = 0x200;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


type Display = BitArr!(for (DISP_WIDTH * DISP_HEIGHT) as usize, in u8, Msb0);

#[allow(unused)]
#[wasm_bindgen]
pub struct CPU {
    memory: [u8; MEM_SIZE],
    display: Display,
    pc: u16,
    index_reg: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    var_regs: [u8; VAR_REG_COUNT],
}

#[wasm_bindgen]
impl CPU {
    pub fn new() -> CPU {
        set_panic_hook();
        CPU {
            memory: [0; MEM_SIZE],
            display: BitArray::ZERO,
            pc: START_ADDR,
            index_reg: 0,
            stack: Vec::new(), 
            delay_timer: 0,
            sound_timer: 0,
            var_regs: [0; VAR_REG_COUNT],
        }
    }

    pub fn load(&mut self, img: &[u8]) {
        let end_addr: usize = START_ADDR as usize + img.len();
        self.memory[START_ADDR as usize..end_addr].copy_from_slice(img);
    }


    pub fn step(&mut self) {
        let upper = self.memory[self.pc as usize];
        let lower = self.memory[(self.pc + 1) as usize];

        self.pc += 2;

        let opcode = upper >> 4;
        let reg_idx_a: usize = (upper & 0xf) as usize;
        let reg_idx_b: usize = (lower >> 4) as usize;
        let addr: u16 = ((upper as u16) << 8 | (lower as u16)) & 0xfff;
        let opt: u8 = lower & 0xf;

        match opcode {
            0x0 => self.handle_misc(upper, lower),
            0x1 => self.pc = addr,
            0x2 => self.call(addr),
            0x3 => if self.var_regs[reg_idx_a] == lower { self.pc += 2 },
            0x4 => if self.var_regs[reg_idx_a] != lower { self.pc += 2 },
            0x5 => if self.var_regs[reg_idx_a] == self.var_regs[reg_idx_b] { self.pc += 2 },
            0x6 => self.var_regs[reg_idx_a] = lower,
            0x7 => self.var_regs[reg_idx_a] += lower,
            0x8 => self.handle_assign(reg_idx_a, reg_idx_b, opt),
            0x9 => if self.var_regs[reg_idx_a] != self.var_regs[reg_idx_b] { self.pc += 2 },
            0xA => self.index_reg = addr,
            0xB => self.pc = self.var_regs[0x0] as u16 + addr,
            0xC => self.var_regs[reg_idx_a] = rand::thread_rng().gen::<u8>() & lower,
            0xD => self.handle_draw(reg_idx_a, reg_idx_b, opt),
            _ => panic!("invalid opcode {}", opcode),
        };
    }

    pub fn display_width(&self) -> u16{
        DISP_WIDTH
    }

    pub fn display_height(&self) -> u16 {
        DISP_HEIGHT
    }

    pub fn pixels(&self) -> *const u8 {
        self.display.as_raw_slice().as_ptr()
    }

    pub fn memory(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    fn call(&mut self, addr: u16) {
        self.stack.push(self.pc); 
        self.pc = addr;
    }

    fn handle_misc(&mut self, upper: u8, lower: u8) {
        match (upper, lower) {
            (0x00, 0xE0) => self.display = BitArray::ZERO,
            (0x00, 0xEE) => self.pc = self.stack.pop().expect("invalid return"),
            _ => panic!("invalid opcode"),
        };
    }

    fn handle_assign(&mut self, reg_idx_a: usize, reg_idx_b: usize, opt: u8) {
        let reg_val_a = self.var_regs[reg_idx_a];
        let reg_val_b = self.var_regs[reg_idx_b];

        match opt {
            0x0 => self.var_regs[reg_idx_a] = reg_val_b,
            0x1 => self.var_regs[reg_idx_a] |= reg_val_b,
            0x2 => self.var_regs[reg_idx_a] &= reg_val_b,
            0x3 => self.var_regs[reg_idx_a] ^= reg_val_b,
            0x4 => {
                self.var_regs[0xF] = if reg_val_a > u8::MAX - reg_val_b { 1 } else { 0 };
                self.var_regs[reg_idx_a] = reg_val_a.wrapping_add(reg_val_b);
            },
            0x5 => {
                self.var_regs[0xF] = if reg_val_a < reg_val_b { 1 } else { 0 };
                self.var_regs[reg_idx_a] = reg_val_a.wrapping_sub(reg_val_b);
            },
            0x6 => {
                self.var_regs[0xF] = reg_val_a & 0x1;
                self.var_regs[reg_idx_a] >>= 1;
            },
            0x7 => {
                self.var_regs[0xF] = if reg_val_b < reg_val_a { 1 } else { 0 };
                self.var_regs[reg_idx_a] = reg_val_b.wrapping_sub(reg_val_a);
            },
            0xE => {
                self.var_regs[0xF] = reg_val_a >> 7;
                self.var_regs[reg_idx_a] <<= 1;
            },
            _ => panic!("invalid opcode"),
        }
    }

    fn handle_draw(&mut self, reg_idx_a: usize, reg_idx_b: usize, n: u8) {
        let x = self.var_regs[reg_idx_a];
        let y = self.var_regs[reg_idx_b];

        let mut addr: usize = self.index_reg.into();

        for i in 0..n {
            let row: u8 = self.memory[addr];
            let start: usize = (y + i) as usize * DISP_WIDTH as usize + x as usize;
            self.display[start..(start + 8)] ^= row.view_bits::<Msb0>();
            addr += 1;
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pc_init() {
        let cpu = CPU::new();
        assert_eq!(cpu.pc, START_ADDR);
    }

    #[test]
    fn test_load() {
        let mut cpu = CPU::new();
        let img = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        cpu.load(&img);

        assert_eq!(cpu.memory[START_ADDR as usize..(START_ADDR + 8) as usize], img);
    }

    #[test]
    fn test_clear_screen() { 
        let mut cpu = CPU::new();
        let img = [0x00, 0xE0];

        cpu.load(&img);
        cpu.display[0..8].store(0xff);
        cpu.step();

        assert_eq!(cpu.display[0..8].load::<u8>(), 0x00);
    }

    #[test]
    fn test_jump() {
        let mut cpu = CPU::new();
        let img = [0x12, 0x34];

        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, 0x234);
    }

    #[test]
    fn test_set() {
        let mut cpu = CPU::new();
        let img = [0x6A, 0x23];

        cpu.load(&img);
        cpu.step();
        
        assert_eq!(cpu.var_regs[0xA], 0x23);
    }

    #[test]
    fn test_add() {
        let mut cpu = CPU::new();
        let img = [0x7A, 0x23];
        cpu.var_regs[0xA] = 0x45;

        cpu.load(&img);
        cpu.step();
        
        assert_eq!(cpu.var_regs[0xA], 0x68);
    }

    #[test]
    fn test_set_index() {
        let mut cpu = CPU::new();
        let img = [0xA1, 0x23];

        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.index_reg, 0x123);
    }

    #[test]
    fn test_draw() {
        let mut cpu = CPU::new();
        let img = [0xDA, 0xB3, 0x55, 0xaa, 0x55];
        let x = 3;
        let y = 4;

        cpu.index_reg = START_ADDR + 0x2;
        cpu.var_regs[0xA] = x; 
        cpu.var_regs[0xB] = y; 

        cpu.load(&img);
        cpu.step();

        let mut start: usize = y as usize * DISP_WIDTH as usize + x as usize;
        assert_eq!(cpu.display[start..(start + 8)].load_be::<u8>(), 0x55);

        start += DISP_WIDTH as usize;
        assert_eq!(cpu.display[start..(start + 8)].load_be::<u8>(), 0xaa);

        start += DISP_WIDTH as usize;
        assert_eq!(cpu.display[start..(start + 8)].load_be::<u8>(), 0x55);
    }

    #[test]
    fn test_skip_if_equal() {
        let mut cpu = CPU::new();
        let img = [0x35, 0xC8];

        cpu.var_regs[0x5] = 0xC7;        
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x2);

        cpu = CPU::new();
        cpu.var_regs[0x5] = 0xC8;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x4);
    }

    #[test]
    fn test_skip_if_not_equal() {
        let mut cpu = CPU::new();
        let img = [0x45, 0xC8];

        cpu.var_regs[0x5] = 0xC7;        
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x4);

        cpu = CPU::new();
        cpu.var_regs[0x5] = 0xC8;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x2);
    }

    #[test]
    fn test_skip_if_regs_equal() {
        let mut cpu = CPU::new();
        let img = [0x5A, 0x60];

        cpu.var_regs[0xA] = 0xB2;
        cpu.var_regs[0x6] = 0xB2;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x4);

        cpu = CPU::new();
        cpu.var_regs[0xA] = 0xB2;
        cpu.var_regs[0x6] = 0xB3;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x2);
    }

    #[test]
    fn test_skip_if_regs_not_equal() {
        let mut cpu = CPU::new();
        let img = [0x9A, 0x60];

        cpu.var_regs[0xA] = 0xB2;
        cpu.var_regs[0x6] = 0xB2;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x2);

        cpu = CPU::new();
        cpu.var_regs[0xA] = 0xB2;
        cpu.var_regs[0x6] = 0xB3;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, START_ADDR + 0x4);
    }

    #[test]
    fn test_assign() {
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC0];

        cpu.var_regs[0xC] = 0x2C;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0x2C);
    }

    #[test]
    fn test_or_assign() {
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC1];

        cpu.var_regs[0xB] = 0xAA;
        cpu.var_regs[0xC] = 0x55;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0xFF);
    }

    #[test]
    fn test_and_assign() {
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC2];

        cpu.var_regs[0xB] = 0xAA;
        cpu.var_regs[0xC] = 0xA5;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0xA0);
    }

    #[test]
    fn test_xor_assign() {
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC3];

        cpu.var_regs[0xB] = 0xAA;
        cpu.var_regs[0xC] = 0xA5;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0x0F);
    }

    #[test]
    fn test_add_assign() { 
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC4];

        cpu.var_regs[0xB] = 0xF3;
        cpu.var_regs[0xC] = 0x45;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0x38);
        assert_eq!(cpu.var_regs[0xF], 1);
    }

    #[test]
    fn test_sub_assign() { 
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC5];

        cpu.var_regs[0xB] = 0x45;
        cpu.var_regs[0xC] = 0xF3;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0x52);
        assert_eq!(cpu.var_regs[0xF], 1);
    }

    #[test]
    fn test_right_shift_assign() {
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC6];

        cpu.var_regs[0xB] = 0x45;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0x22);
        assert_eq!(cpu.var_regs[0xF], 1);       
    }

    #[test]
    fn test_reverse_sub_assign() { 
        let mut cpu = CPU::new();
        let img = [0x8B, 0xC7];

        cpu.var_regs[0xB] = 0xF3;
        cpu.var_regs[0xC] = 0x45;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0x52);
        assert_eq!(cpu.var_regs[0xF], 1);
    }

    #[test]
    fn test_left_shift_assign() {
        let mut cpu = CPU::new();
        let img = [0x8B, 0xCE];

        cpu.var_regs[0xB] = 0xFF;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.var_regs[0xB], 0xFE);
        assert_eq!(cpu.var_regs[0xF], 1);       
    }

    #[test]
    fn test_reg_offset_jump() {
        let mut cpu = CPU::new();
        let img = [0xB2, 0x02];

        cpu.var_regs[0x0] = 0x23;
        cpu.load(&img);
        cpu.step();
        assert_eq!(cpu.pc, 0x225);
    }

    #[test]
    fn test_random_assign() {
        let mut cpu = CPU::new();
        let img = [0xC8, 0xF0];

        cpu.var_regs[0x8] = 0xFF;
        cpu.load(&img);
        cpu.step();

        assert_eq!(cpu.var_regs[0x8] & 0xF, 0);
    }   

    #[test]
    fn test_call() {
        let mut cpu = CPU::new();
        let img = [0x23, 0x45];

        cpu.load(&img);
        cpu.step();

        assert_eq!(cpu.stack[0], START_ADDR + 0x2);
        assert_eq!(cpu.pc, 0x345);
    }   

    #[test]
    fn test_return() {
        let mut cpu = CPU::new();
        let img = [0x00, 0xEE];

        cpu.stack.push(0x369);
        cpu.load(&img);
        cpu.step();

        assert_eq!(cpu.pc, 0x369);
        assert_eq!(cpu.stack.len(), 0);
    }   
}