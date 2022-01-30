use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

mod fifo_mode;
mod lcd_control;
mod lcd_status;

use fifo_mode::FifoMode;
use lcd_control::LcdControl;
use lcd_status::LcdStatus;

pub const FRAME_WIDTH: usize = 160;
pub const FRAME_HEIGHT: usize = 144;

pub type Frame = Box<[u8; FRAME_WIDTH * FRAME_HEIGHT]>;

pub struct Ppu {
    x: u16,
    y: u8,
    y_compare: u8,

    window_x: u8,
    window_y: u8,

    scroll_x: u8,
    scroll_y: u8,

    vram: [u8; 0x3FFF],
    vram_bank_register: bool,
    oam: [u8; 0x9f],
    lcd_control_reg: LcdControl,
    lcd_status_reg: LcdStatus,

    greyscale_bg_palette: u8,
    greyscale_obj_palette: [u8; 2],

    background_pixel_pipeline: u128,
    sprite_pixel_pipeline: u128,

    cycle: u16,
    fifo_mode: FifoMode,
    frame: Option<Frame>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            y_compare: 0,

            window_x: 0,
            window_y: 0,

            scroll_x: 0,
            scroll_y: 0,

            vram: [0u8; 0x3FFF],
            vram_bank_register: false,
            oam: [0u8; 0x9f],
            lcd_control_reg: Default::default(),
            lcd_status_reg: Default::default(),

            greyscale_bg_palette: 0,
            greyscale_obj_palette: [0; 2],

            background_pixel_pipeline: 0,
            sprite_pixel_pipeline: 0,

            cycle: 0,
            fifo_mode: Default::default(),
            frame: None,
        }
    }
}

impl Ppu {
    pub fn new() -> Self {
        let mut ppu: Self = Default::default();

        ppu.allocate_new_frame();

        ppu
    }

    pub fn clock(&mut self) {
        // TODO: Actual rendering
        self.cycle += 1;

        if self.cycle == 80 {
            self.fifo_mode = FifoMode::Drawing;
        } else if self.cycle == 456 {
            self.cycle = 0;
            self.x = 0;
            self.y += 1;

            match self.y {
                143..=153 => {
                    // We are in VBLANK
                    self.fifo_mode = FifoMode::VBlank;
                }
                154 => {
                    // End of the frame
                    self.y = 0;
                    self.fifo_mode = FifoMode::OamScan;
                }
                _ => {
                    self.fifo_mode = FifoMode::OamScan;
                }
            };
        };
    }

    pub fn ready_frame(&mut self) -> Option<Frame> {
        if self.y == 0 && self.cycle == 0 {
            // Returns the current frame buffer
            let frame = self
                .frame
                .take()
                .expect("the frame buffer should never be unallocated");

            // Allocate a new frame buffer
            self.allocate_new_frame();

            Some(frame)
        } else {
            None
        }
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) {
        match self.fifo_mode {
            FifoMode::Drawing => {
                // Calls are blocked during this mode
                // Do nothing
            }
            _ => {
                let addr = addr & 0x1FFF | if self.vram_bank_register { 0x2000 } else { 0 };
                self.vram[addr as usize] = data;
            }
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        match self.fifo_mode {
            FifoMode::Drawing => {
                // Calls are blocked during this mode
                // Do nothing and return trash
                0xFF
            }
            _ => {
                let addr = addr & 0x1FFF | if self.vram_bank_register { 0x2000 } else { 0 };
                self.vram[addr as usize]
            }
        }
    }

    pub fn write_oam(&mut self, addr: u16, data: u8) {
        match self.fifo_mode {
            FifoMode::OamScan | FifoMode::Drawing => {
                // Calls are blocked during this mode
                // Do nothing
            }
            _ => {
                let addr = addr & 0x7F;
                self.oam[addr as usize] = data;
            }
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        match self.fifo_mode {
            FifoMode::OamScan | FifoMode::Drawing => {
                // Calls are blocked during this mode
                // Do nothing and return trash
                0xFF
            }
            _ => {
                let addr = addr & 0x7F;
                self.oam[addr as usize]
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF40 => self.write_lcd_control(data),
            0xFF41 => self.write_lcd_status(data),
            0xFF42 => self.scroll_y = data,
            0xFF43 => self.scroll_x = data,
            0xFF45 => self.y_compare = data,
            0xFF47 => self.greyscale_bg_palette = data,
            0xFF48 | 0xFF49 => self.greyscale_obj_palette[(addr & 1) as usize] = data,
            0xFF4A => self.window_y = data,
            0xFF4B => self.window_x = data,
            _ => {
                // Address not recognised, do nothing
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.read_lcd_control(),
            0xFF41 => self.read_lcd_status(),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.y,
            0xFF45 => self.y_compare,
            0xFF47 => self.greyscale_bg_palette,
            0xFF48 | 0xFF49 => self.greyscale_obj_palette[(addr & 1) as usize],
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => {
                // Address not recognised, do nothing
                0
            }
        }
    }

    fn write_lcd_control(&mut self, data: u8) {
        self.lcd_control_reg =
            LcdControl::from_bits(data).expect("any data should be valid for LCDC bitflags")
    }

    fn read_lcd_control(&self) -> u8 {
        self.lcd_control_reg.bits()
    }

    fn write_lcd_status(&mut self, data: u8) {
        // Only those bits are writeable.
        let mask = 0b01111000;
        let status_reg = self.lcd_status_reg.bits() & !mask;
        let status_reg = status_reg | (data & mask);

        self.lcd_status_reg = LcdStatus::from_bits(status_reg)
            .expect("the reg can take 8 bits, so no value shoul fail");
    }

    fn read_lcd_status(&self) -> u8 {
        let mut status_reg = self.lcd_status_reg.clone();

        // Those bits are constantly changed, so might as well update them only when needed
        status_reg.set(LcdStatus::LYC_EQ_LC, self.y == self.y_compare);
        status_reg.set_mode(self.fifo_mode);

        status_reg.bits()
    }

    fn allocate_new_frame(&mut self) {
        //   Hackish way to create fixed size boxed array.
        // I don't know of any way to do it without
        // having the data allocated on the stack at some point or using unsafe
        let v: Vec<u8> = vec![0u8; FRAME_WIDTH * FRAME_HEIGHT];
        let b = v.into_boxed_slice();

        // Safety: This only uses constants and the fonction doesn't have arguments
        self.frame = unsafe {
            Some(Box::from_raw(
                Box::into_raw(b) as *mut [u8; FRAME_WIDTH * FRAME_HEIGHT]
            ))
        }
    }
}
