// Copyright © 2019 Intel Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Inspired by https://github.com/phil-opp/blog_os/blob/post-03/src/vga_buffer.rs
// from Philipp Oppermann

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use cpuio::Port;
use core::ffi::c_void;

use r_efi::protocols::simple_text_output::Mode as SimpleTextOutputMode;
use r_efi::efi::Status;

use crate::efi::STDOUT_MODE;

const OUTPUT_ESC : bool = false;

const CHAR_BACKSPACE       : u8 = 0x08;
const CHAR_TAB             : u8 = 0x09;
const CHAR_LINEFEED        : u8 = 0x0a;
const CHAR_CARRIAGE_RETURN : u8 = 0x0d;

const EFI_BLACK                 : u8 = 0x00;
const EFI_BLUE                  : u8 = 0x01;
const EFI_GREEN                 : u8 = 0x02;
const EFI_CYAN                  : u8 = (EFI_BLUE | EFI_GREEN);
const EFI_RED                   : u8 = 0x04;
const EFI_MAGENTA               : u8 = (EFI_BLUE | EFI_RED);
const EFI_BROWN                 : u8 = (EFI_GREEN | EFI_RED);
const EFI_LIGHTGRAY             : u8 = (EFI_BLUE | EFI_GREEN | EFI_RED);
const EFI_BRIGHT                : u8 = 0x08;

const ESC                       : u16 = 0x1B;
const BRIGHT_CONTROL_OFFSET     : usize = 2;
const FOREGROUND_CONTROL_OFFSET : usize = 6;
const BACKGROUND_CONTROL_OFFSET : usize = 11;
const ROW_OFFSET                : usize = 2;
const COLUMN_OFFSET             : usize = 5;

const SET_MODE_STRING_SIZE            : usize = 6;
const SET_ATTRIBUTE_STRING_SIZE       : usize = 15;
const CLEAR_SCREEN_STRING_SIZE        : usize = 5;
const SET_CURSOR_POSITION_STRING_SIZE : usize = 9;
const CURSOR_FORWARD_STRING_SIZE      : usize = 6;
const CURSOR_BACKWARD_STRING_SIZE     : usize = 6;


static mut SET_MODE_STRING: [u16; SET_MODE_STRING_SIZE]                       = [ ESC, '[' as u16, '=' as u16, '3' as u16, 'h' as u16, 0 ];
static mut SET_ATTRIBUTE_STRING: [u16; SET_ATTRIBUTE_STRING_SIZE]             = [ ESC, '[' as u16, '0' as u16, 'm' as u16, ESC, '[' as u16, '4' as u16, '0' as u16, 'm' as u16, ESC, '[' as u16, '4' as u16, '0' as u16, 'm' as u16, 0 ];
static mut CLEAR_SCREEN_STRING: [u16; CLEAR_SCREEN_STRING_SIZE]               = [ ESC, '[' as u16, '2' as u16, 'J' as u16, 0 ];
static mut SET_CURSOR_POSITION_STRING: [u16; SET_CURSOR_POSITION_STRING_SIZE] = [ ESC, '[' as u16, '0' as u16, '0' as u16, ';' as u16, '0' as u16, '0' as u16, 'H' as u16, 0 ];

pub struct ConOut {
    port: Port<u8>,
    mode_ptr: usize,
}

impl ConOut {
    pub fn write_byte(&mut self, byte: u8) {
        self.port.write(byte);
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
    }

    pub fn output_string (&mut self, message: *mut u16) {
        let mode : *mut SimpleTextOutputMode = self.mode_ptr as *mut c_void as *mut SimpleTextOutputMode;
        let max_column = 80;
        let mut max_row = 25;
        unsafe {
          if (*mode).mode == 1 {
            max_row = 50;
          }
        }
    
        let mut i: usize = 0;
        loop {
          let output = (unsafe { *message.add(i) } & 0xffu16) as u8;
          i += 1;
          if output == 0 {
              break;
          } else {
              self.write_byte(output);
          }

          unsafe {
            match output {
              CHAR_BACKSPACE => {
                if (*mode).cursor_column > 0 {
                  (*mode).cursor_column = (*mode).cursor_column - 1;
                }
              },
              CHAR_LINEFEED  => {
                if (*mode).cursor_row < max_row - 1 {
                  (*mode).cursor_row = (*mode).cursor_row + 1;
                }
              },
              CHAR_CARRIAGE_RETURN => {
                (*mode).cursor_column = 0;
              },
              _ => {
                if (*mode).cursor_column < max_column - 1 {
                  (*mode).cursor_column = (*mode).cursor_column + 1;
                } else {
                  (*mode).cursor_column = 0;
                  if (*mode).cursor_row < max_row - 1 {
                    (*mode).cursor_row = (*mode).cursor_row + 1;
                  }
                }
              },
            };
          }
        }

    }

    pub fn set_cursor_position(&mut self, column: usize, row: usize) {
        unsafe {
          SET_CURSOR_POSITION_STRING[ROW_OFFSET + 0]    = ('0' as usize + ((row + 1) / 10)) as u16;
          SET_CURSOR_POSITION_STRING[ROW_OFFSET + 1]    = ('0' as usize + ((row + 1) % 10)) as u16;
          SET_CURSOR_POSITION_STRING[COLUMN_OFFSET + 0] = ('0' as usize + ((column + 1) / 10)) as u16;
          SET_CURSOR_POSITION_STRING[COLUMN_OFFSET + 1] = ('0' as usize + ((column + 1) % 10)) as u16;

          if (OUTPUT_ESC) {
            self.output_string(&mut SET_CURSOR_POSITION_STRING as *mut [u16; SET_CURSOR_POSITION_STRING_SIZE] as *mut u16);
          }
        }
        
        let mode : *mut SimpleTextOutputMode = self.mode_ptr as *mut c_void as *mut SimpleTextOutputMode;
        unsafe {
            (*mode).cursor_column = column as isize as i32;
            (*mode).cursor_row = row as isize as i32;
        }

        if column == 0 {
          self.port.write('\r' as u8);
        }
    }

    pub fn set_attribute(&mut self, attribute: usize) {
        if (attribute | 0x7f) != 0x7f {
          return ;
        }

        let mode : *mut SimpleTextOutputMode = self.mode_ptr as *mut c_void as *mut SimpleTextOutputMode;

        unsafe {
          if (*mode).attribute == (attribute as isize as i32) {
            return ;
          }
        }

        let mut saved_column : i32 = 0;
        let mut saved_row : i32 = 0;
        unsafe {
            saved_column = (*mode).cursor_column;
            saved_row = (*mode).cursor_row;
        }

        let mut foreground_control : usize = 0;
        match (attribute & 0x7) as u8 {
          EFI_BLACK => {foreground_control = 30},
          EFI_BLUE => {foreground_control = 34},
          EFI_GREEN => {foreground_control = 32},
          EFI_CYAN => {foreground_control = 36},
          EFI_RED => {foreground_control = 31},
          EFI_MAGENTA => {foreground_control = 35},
          EFI_BROWN => {foreground_control = 33},
          EFI_LIGHTGRAY => {foreground_control = 37},
          _ => {foreground_control = 37},
        }

        let mut bright_control : usize = (attribute >> 3) & 1;

        let mut background_control : usize = 0;
        match ((attribute >> 4) & 0x7) as u8 {
          EFI_BLACK => {background_control = 40},
          EFI_BLUE => {background_control = 44},
          EFI_GREEN => {background_control = 42},
          EFI_CYAN => {background_control = 46},
          EFI_RED => {background_control = 41},
          EFI_MAGENTA => {background_control = 45},
          EFI_BROWN => {background_control = 43},
          EFI_LIGHTGRAY => {background_control = 47},
          _ => {background_control = 47},
        }

        unsafe {
          SET_ATTRIBUTE_STRING[BRIGHT_CONTROL_OFFSET]         = ('0' as usize + bright_control) as u16;
          SET_ATTRIBUTE_STRING[FOREGROUND_CONTROL_OFFSET + 0] = ('0' as usize + (foreground_control / 10)) as u16;
          SET_ATTRIBUTE_STRING[FOREGROUND_CONTROL_OFFSET + 1] = ('0' as usize + (foreground_control % 10)) as u16;
          SET_ATTRIBUTE_STRING[BACKGROUND_CONTROL_OFFSET + 0] = ('0' as usize + (background_control / 10)) as u16;
          SET_ATTRIBUTE_STRING[BACKGROUND_CONTROL_OFFSET + 1] = ('0' as usize + (background_control % 10)) as u16;

          if (OUTPUT_ESC) {
            self.output_string(&mut SET_ATTRIBUTE_STRING as *mut [u16; SET_ATTRIBUTE_STRING_SIZE] as *mut u16);
          }
        }
        unsafe {
            (*mode).cursor_column = saved_column;
            (*mode).cursor_row = saved_row;
            (*mode).attribute = attribute as isize as i32;
        }
    }

    pub fn clear_screen(&mut self) {
        unsafe {
          if (OUTPUT_ESC) {
            self.output_string(&mut CLEAR_SCREEN_STRING as *mut [u16; CLEAR_SCREEN_STRING_SIZE] as *mut u16);
          }
        }

        self.set_cursor_position (0, 0);
    }

    pub fn set_mode(&mut self, mode_number: usize) -> Status {
      let mode : *mut SimpleTextOutputMode = self.mode_ptr as *mut c_void as *mut SimpleTextOutputMode;

      unsafe {
        if mode_number as isize as i32 > (*mode).max_mode {
          return Status::UNSUPPORTED;
        }

        (*mode).mode = mode_number as isize as i32;
      }

      self.clear_screen ();
      unsafe {
        if (OUTPUT_ESC) {
          self.output_string(&mut SET_MODE_STRING as *mut [u16; SET_MODE_STRING_SIZE] as *mut u16);
        }
      }
      
      unsafe {
        (*mode).mode = mode_number as isize as i32;
      }

      self.clear_screen ();
      Status::SUCCESS
    }

    pub fn new() -> ConOut {
        ConOut {
            port: unsafe { Port::new(0x3f8) },
            mode_ptr: unsafe { &mut STDOUT_MODE as *mut SimpleTextOutputMode as usize },
        }
    }
}

