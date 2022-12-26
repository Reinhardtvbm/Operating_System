use core::ops::{Deref, DerefMut};

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug, Eq)]
#[repr(u8)]
enum VGAColour {
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

/// A wrapper over a single byte to represent a colour code in the vga
/// where the first nibble is the background colour, and the second is
/// the foreground colour:
///  0 1 2 3 4 5 6 7
/// [ back  | fore  ]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // ensure that ColourCode has the same layout as `u8`
struct ColourCode(u8);

impl ColourCode {
    fn new(background_col: VGAColour, foreground_col: VGAColour) -> Self {
        Self((background_col as u8) << 4 | foreground_col as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // ensures the struct fields are laid out in memory in the same order as below
struct ScreenChar {
    ascii_char: u8,
    colour_byte: ColourCode,
}

impl Deref for ScreenChar {
    type Target = ScreenChar;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const LAST_LINE: usize = 24;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.chars[LAST_LINE][self.column_position].write(ScreenChar {
                    ascii_char: byte,
                    colour_byte: self.colour_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        s.bytes().for_each(|byte| match byte {
            // if the byte is in ASCII
            0x20..=0x7E | b'\n' => self.write_byte(byte),
            _ => self.write_byte(0xFE),
        });
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank_char = ScreenChar {
            ascii_char: b' ',
            colour_byte: self.colour_code,
        };

        for char_index in 0..BUFFER_WIDTH {
            self.buffer.chars[row][char_index].write(blank_char);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn print_something() {
    use core::fmt::Write;

    let mut writer = Writer {
        column_position: 0,
        colour_code: ColourCode::new(VGAColour::Black, VGAColour::White),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    };

    writeln!(
        writer,
        "The answer to life, the universe, and everything is {}",
        42
    )
    .unwrap();
}

lazy_static! {
    /// instead of declaring a writer for the vga buffer at 0xB8000, we
    /// want a static/constant global writer for the vga buffer. A spinlock
    /// Mutex ensures readwrite safety
    pub static ref VGA_WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        colour_code: ColourCode::new(VGAColour::Black, VGAColour::White),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}

/// Copy paste print! and prinln! from `libstd` to get out own println!() for the
/// VGA buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    VGA_WRITER.lock().write_fmt(args).unwrap();
}
