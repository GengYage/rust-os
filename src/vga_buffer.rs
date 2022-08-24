use core::fmt;
use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER_ADDR: usize = 0xb8000;

// 一屏
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_WIDTH],
}

pub struct Writer {
    char_numbers: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // 计算当前位置
                let mut row = self.char_numbers / BUFFER_WIDTH;
                let col = self.char_numbers % BUFFER_WIDTH;

                // 行数溢出,置为最后一行
                if row >= (BUFFER_HEIGHT - 1) {
                    row = BUFFER_HEIGHT - 1;
                }

                // 写入字符串
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                // 位置+1
                self.char_numbers += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // 若到达最大高度,所有字符串前移一行
        if self.char_numbers / BUFFER_WIDTH >= BUFFER_HEIGHT {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    //上移一行
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
        }
        // 清空最后一行
        self.clear_row(BUFFER_HEIGHT - 1);

        // 先防止整数溢出
        if self.char_numbers >= (usize::MAX - BUFFER_WIDTH * BUFFER_HEIGHT) {
            // 与一页字符数取模
            self.char_numbers = self.char_numbers % (BUFFER_WIDTH * BUFFER_HEIGHT);
        }

        self.char_numbers = (self.char_numbers / BUFFER_WIDTH + 1) * BUFFER_WIDTH;
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // ascii byte
                0x20..=0x7e | b'\n' => self.write_byte(byte),

                // not part of ascii
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        char_numbers: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! println {
    () => {print!("\n")};
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}