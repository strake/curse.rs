#![no_std]

#[macro_use]
extern crate bitflags;
extern crate cursebox;
extern crate libc;

use core::char;
use core::marker::PhantomData;
use core::mem;
use cursebox::*;
use libc::c_int;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Key {
    Tab,
    Enter,
    Esc,
    Backspace,
    Right,
    Left,
    Up,
    Down,
    Delete,
    Insert,
    Home,
    End,
    PgUp,
    PgDn,
    Char(char),
    Ctrl(char),
    F(u16),
}

impl Key {
    pub fn from_code(code: u16) -> Option<Key> {
        use Key::*;
        match code {
              8 => Some(Backspace),
              9 => Some(Tab),
             13 => Some(Enter),
            1...26 => Some(Ctrl(unsafe { char::from_u32_unchecked('`' as u32 + code as u32) })),
             27 => Some(Esc),
             28 => Some(Ctrl('\\')),
             29 => Some(Ctrl(']')),
             30 => Some(Ctrl('6')),
             31 => Some(Ctrl('/')),
             32 => Some(Char(' ')),
            127 => Some(Backspace),
            TB_KEY_ARROW_LEFT  => Some(Left),
            TB_KEY_ARROW_RIGHT => Some(Right),
            TB_KEY_ARROW_UP    => Some(Up),
            TB_KEY_ARROW_DOWN  => Some(Down),
            TB_KEY_INSERT => Some(Insert),
            TB_KEY_DELETE => Some(Delete),
            TB_KEY_HOME => Some(Home),
            TB_KEY_END  => Some(End),
            TB_KEY_PGUP => Some(PgUp),
            TB_KEY_PGDN => Some(PgDn),
            TB_KEY_F1  => Some(F( 1)),
            TB_KEY_F2  => Some(F( 2)),
            TB_KEY_F3  => Some(F( 3)),
            TB_KEY_F4  => Some(F( 4)),
            TB_KEY_F5  => Some(F( 5)),
            TB_KEY_F6  => Some(F( 6)),
            TB_KEY_F7  => Some(F( 7)),
            TB_KEY_F8  => Some(F( 8)),
            TB_KEY_F9  => Some(F( 9)),
            TB_KEY_F10 => Some(F(10)),
            TB_KEY_F11 => Some(F(11)),
            TB_KEY_F12 => Some(F(12)),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C, u16)]
pub enum Color {
    Black   = 0x00,
    Red     = 0x01,
    Green   = 0x02,
    Yellow  = 0x03,
    Blue    = 0x04,
    Magenta = 0x05,
    Cyan    = 0x06,
    White   = 0x07,
    Default = 0x0F,
}

bitflags! {
    #[repr(C)]
    pub struct Face: u8 {
        const BOLD      = 0x10;
        const UNDERLINE = 0x20;
        const REVERSE   = 0x40;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Event {
    Key(Key),
    Resize(u32, u32),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Failure {
    Unknown,
    UnsupportedTerminal,
    FailedToOpenTty,
    PipeTrapError,
}

#[derive(Debug)]
pub struct Term(PhantomData<*mut ()>);

impl Term {
    pub fn init() -> Result<Self, Failure> {
        let c = unsafe { tb_init() };
        if c >= 0 { Ok(Term(PhantomData)) } else {
            use Failure::*;
            Err(match c {
                    TB_EUNSUPPORTED_TERMINAL => UnsupportedTerminal,
                    TB_EFAILED_TO_OPEN_TTY => FailedToOpenTty,
                    TB_EPIPE_TRAP_ERROR => PipeTrapError,
                    _ => Unknown,
                })
        }
    }


    pub fn width(&self) -> usize { unsafe { tb_width() as usize } }
    pub fn height(&self) -> usize { unsafe { tb_height() as usize } }

    pub fn clear(&mut self) { unsafe { tb_clear() } }
    pub fn freshen(&mut self) { unsafe { tb_present() } }

    pub fn print_char(&mut self, x_pos: usize, y_pos: usize,
                      face: Face, fg: Color, bg: Color, x: char) {
        unsafe { tb_change_cell(x_pos as c_int, y_pos as c_int, x as u32,
                                fg as u16 | ((face.bits as u16) << 8), bg as u16) };
    }

    pub fn next_event(&mut self, timeout: Option<u32>) -> Result<Option<Event>, ()> { unsafe {
        let mut ev: RawEvent = mem::uninitialized();
        let c = match timeout {
            Some(t_msec) => tb_peek_event(&mut ev, t_msec as c_int),
            None => tb_poll_event(&mut ev),
        };
        if c < 0 { return Err(()) };
        match c as u8 {
            0 => Ok(None),
            TB_EVENT_KEY => Ok(if ev.key == 0 { char::from_u32(ev.ch).map(Key::Char) }
                               else { Key::from_code(ev.key) }.map(Event::Key)),
            TB_EVENT_RESIZE => Ok(Some(Event::Resize(ev.w as u32, ev.h as u32))),
            _ => Err(()),
        }
    } }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        unsafe { tb_set_cursor(x as c_int, y as c_int) }
    }
}

impl Drop for Term {
    #[inline] fn drop(&mut self) {
        unsafe { tb_shutdown() };
    }
}
