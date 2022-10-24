mod utils;

use crate::utils::set_panic_hook;

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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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

        let long_val = ((upper as u16) << 8 | (lower as u16)) & 0xfff;
        log(&format!("{:02x} {:02x} {:03x} {:03x}", upper, lower, self.pc, long_val));

        match upper >> 4 {
            0x0 => self.handle_misc(upper, lower),
            0x1 => self.pc = long_val,
            0x6 => self.var_regs[(upper & 0xf) as usize] = lower,
            0x7 => self.var_regs[(upper & 0xf) as usize] += lower,
            0xA => self.index_reg = long_val,
            0xD => self.handle_draw(upper, lower),
            _ => (),
        };
    }

    pub fn render(&self) -> String {
        let mut disp_text: String = String::new();

        for line in self.display.as_bitslice().chunks(DISP_WIDTH as usize) {
            for bit in line {
                disp_text.push(if *bit { '⬜' } else { '⬛' });
            }
            disp_text.push('\n');
        }

        disp_text
    }

    pub fn display_width(&self) -> u16{
        DISP_WIDTH
    }

    pub fn display_height(&self) -> u16 {
        DISP_HEIGHT
    }

    pub fn display_pixels(&self) -> *const u8 {
        self.display.as_raw_slice().as_ptr()
    }

    fn handle_misc(&mut self, upper: u8, lower: u8) {
        match (upper, lower) {
            (0x00, 0xE0) => self.display = BitArray::ZERO,
            _ => panic!("invalid opcode"),
        };
    }

    fn handle_draw(&mut self, upper: u8, lower: u8) {
        let x = self.var_regs[(upper & 0xf) as usize];
        let y = self.var_regs[(lower >> 4) as usize];
        let n = lower & 0xf;

        log(&format!("{} {} {}", x, y, n));

        let mut addr: usize = self.index_reg.into();

        for i in 0..n {
            let row: u8 = self.memory[addr];
            log(&format!("{}", row));
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

        println!("{}", cpu.render());
    }
}