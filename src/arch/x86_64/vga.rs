use core::ptr::Unique;
use core::fmt;
use volatile::Volatile;
use spin::Mutex;

/// The global writer for the x86 VGA buffer.
pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    col: 0,
    color: ColorCode::new(Color::White, Color::Black),
    buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
});

#[macro_export]
macro_rules! vgaprint {
    ($($args:tt)*) => { $crate::arch::x86_64::vga::vgaprint(format_args!($($args)*)); };
}

macro_rules! vgaprintln {
    ($fmt:expr, $($args:tt)*) => { vgaprint!(concat!($fmt, "\n"), $($args)*); };
    ($fmt:expr) => { vgaprint!(concat!($fmt, "\n")); };
    () => { vgaprint!("\n"); };
}

pub fn vgaprint(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    writer.write_fmt(args)
        .expect("Error in arch::x86_64::vga::vgaprint! macro");
}

/// VGA character buffer color.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

/// A color composed of a foreground and background color.
#[derive(Debug, Copy, Clone)]
struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | fg as u8)
    }
}

/// A single character on a VGA screen.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Char {
    ascii: u8,
    color: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

pub struct Buffer {
    chars: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    col: usize,
    color: ColorCode,
    buffer: Unique<Buffer>,
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.col;

                let color = self.color;
                self.buffer().chars[row][col].write(Char {
                    ascii: byte,
                    color,
                });
                self.col += 1;
            }
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) {
        // shift up
        for row in 1 .. BUFFER_HEIGHT {
            for col in 0 .. BUFFER_WIDTH {
                let c = self.buffer().chars[row][col].clone();
                self.buffer().chars[row - 1][col] = c;
            }
        }
        // clear bottom line
        let color = self.color;
        let blank = Char { ascii: b' ', color };
        for col in 0 .. BUFFER_WIDTH {
            self.buffer().chars[BUFFER_HEIGHT - 1][col] = Volatile::new(blank);
        }
        self.col = 0;
    }

    pub fn clear(&mut self) {
        let color = self.color;
        let blank = Char { ascii: b' ', color };
        for row in 0 .. BUFFER_HEIGHT {
            for col in 0 .. BUFFER_WIDTH {
                self.buffer().chars[row][col] = Volatile::new(blank);
            }
        }
        self.col = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}
