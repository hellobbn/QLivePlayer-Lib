/* generated by rust_qt_binding_generator */
use libc::{c_char, c_ushort, c_int};
use std::slice;
use std::char::decode_utf16;

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr::null;

use crate::implementation::*;


pub enum QString {}

fn set_string_from_utf16(s: &mut String, str: *const c_ushort, len: c_int) {
    let utf16 = unsafe { slice::from_raw_parts(str, to_usize(len)) };
    let characters = decode_utf16(utf16.iter().cloned())
        .map(|r| r.unwrap());
    s.clear();
    s.extend(characters);
}



fn to_usize(n: c_int) -> usize {
    if n < 0 {
        panic!("Cannot cast {} to usize", n);
    }
    n as usize
}


fn to_c_int(n: usize) -> c_int {
    if n > c_int::max_value() as usize {
        panic!("Cannot cast {} to c_int", n);
    }
    n as c_int
}


pub struct QLivePlayerLibQObject {}

pub struct QLivePlayerLibEmitter {
    qobject: Arc<AtomicPtr<QLivePlayerLibQObject>>,
}

unsafe impl Send for QLivePlayerLibEmitter {}

impl QLivePlayerLibEmitter {
    /// Clone the emitter
    ///
    /// The emitter can only be cloned when it is mutable. The emitter calls
    /// into C++ code which may call into Rust again. If emmitting is possible
    /// from immutable structures, that might lead to access to a mutable
    /// reference. That is undefined behaviour and forbidden.
    pub fn clone(&mut self) -> QLivePlayerLibEmitter {
        QLivePlayerLibEmitter {
            qobject: self.qobject.clone(),
        }
    }
    fn clear(&self) {
        let n: *const QLivePlayerLibQObject = null();
        self.qobject.store(n as *mut QLivePlayerLibQObject, Ordering::SeqCst);
    }
}

pub trait QLivePlayerLibTrait {
    fn new(emit: QLivePlayerLibEmitter) -> Self;
    fn emit(&mut self) -> &mut QLivePlayerLibEmitter;
    fn get_url(&mut self, room_url: String, extras: String) -> String;
    fn run_danmaku_client(&mut self, unix_socket: String) -> ();
}

#[no_mangle]
pub extern "C" fn q_live_player_lib_new(
    q_live_player_lib: *mut QLivePlayerLibQObject,
) -> *mut QLivePlayerLib {
    let q_live_player_lib_emit = QLivePlayerLibEmitter {
        qobject: Arc::new(AtomicPtr::new(q_live_player_lib)),
    };
    let d_q_live_player_lib = QLivePlayerLib::new(q_live_player_lib_emit);
    Box::into_raw(Box::new(d_q_live_player_lib))
}

#[no_mangle]
pub unsafe extern "C" fn q_live_player_lib_free(ptr: *mut QLivePlayerLib) {
    Box::from_raw(ptr).emit().clear();
}

#[no_mangle]
pub unsafe extern "C" fn q_live_player_lib_get_url(ptr: *mut QLivePlayerLib, room_url_str: *const c_ushort, room_url_len: c_int, extras_str: *const c_ushort, extras_len: c_int, d: *mut QString, set: extern fn(*mut QString, str: *const c_char, len: c_int)) {
    let mut room_url = String::new();
    set_string_from_utf16(&mut room_url, room_url_str, room_url_len);
    let mut extras = String::new();
    set_string_from_utf16(&mut extras, extras_str, extras_len);
    let o = &mut *ptr;
    let r = o.get_url(room_url, extras);
    let s: *const c_char = r.as_ptr() as *const c_char;
    set(d, s, r.len() as i32);
}

#[no_mangle]
pub unsafe extern "C" fn q_live_player_lib_run_danmaku_client(ptr: *mut QLivePlayerLib, unix_socket_str: *const c_ushort, unix_socket_len: c_int) {
    let mut unix_socket = String::new();
    set_string_from_utf16(&mut unix_socket, unix_socket_str, unix_socket_len);
    let o = &mut *ptr;
    o.run_danmaku_client(unix_socket)
}
