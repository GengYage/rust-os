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
            // 换行符直接调用new_line方法“新开一行”
            b'\n' => self.new_line(),

            byte => {
                // 计算当前位置
                let mut row = self.char_numbers / BUFFER_WIDTH;
                let col = self.char_numbers % BUFFER_WIDTH;

                // 行数溢出,置为最后一行
                if row >= (BUFFER_HEIGHT - 1) {
                    // 最后一行一行的最后一个字符,换行
                    if col >= BUFFER_WIDTH - 1 {
                        self.new_line();
                        // 由于换行将最后一行上移,因此要打印到倒数第二行
                        row = BUFFER_HEIGHT - 2;
                        // 抵消换行时将char_numbers置为下一行行首
                        self.char_numbers -= 1;
                    } else {
                        row = BUFFER_HEIGHT - 1;
                    }
                }

                // 写入字符串
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                // 字符数+1
                self.char_numbers += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // 先防止整数溢出 字符数到达最大值-一页的范围内时，就对一页字符数取模，防止整数溢出
        if self.char_numbers >= (usize::MAX - BUFFER_WIDTH * BUFFER_HEIGHT) {
            // 与一页字符数取模
            self.char_numbers = self.char_numbers % (BUFFER_WIDTH * BUFFER_HEIGHT);
        }

        // 换行 字符数 + 80 减去本行的字符数(self.char_numbers % BUFFER_WIDTH 计算出新行有多少个字符)
        self.char_numbers = self.char_numbers + BUFFER_WIDTH - self.char_numbers % BUFFER_WIDTH;

        // 若到达最大高度,所有字符上移一行,并清空最后一行
        if self.char_numbers / BUFFER_WIDTH >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    //上移一行
                    self.buffer.chars[row - 1][col].write(character);
                }
            }

            // 清空最后一行
            self.clear_row(BUFFER_HEIGHT - 1);
        }
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
        // 定于空白字符
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        // 将最后一行都设置为空白字符
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

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // 在闭包执行时禁用中断
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => {print!("\n")};
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[test_case]
fn test_print_many_characters() {
    for i in 0..1024 {
        println!("print test:{}", i);
    }
}