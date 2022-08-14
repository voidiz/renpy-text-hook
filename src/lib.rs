use detour::static_detour;
use std::ffi::{c_void, CString};
use std::io::Write;
use std::os::unix::net::UnixStream;

use libc::{dlsym, RTLD_DEFAULT};

const SOCKET_PATH: &'static str = "/tmp/renpy-text-hook";

static_detour! {
    static PyUnicodeUcs4FormatHook: unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
}

type PyUnicodeUcs4Format =
    extern "C" fn(format: *const c_void, args: *const c_void) -> *const c_void;

/// Connect to the socket created by the injector, write to it, and close it
fn write_to_stream(line: &String) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write(line.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    Ok(())
}

fn py_unicode_ucs4_format_detour(format: *const c_void, args: *const c_void) -> *const c_void {
    // The format char pointer is located at offset 0x18
    let str_field = unsafe { format.offset(0x18) };

    // Dereference the pointer to get the format char pointer
    let str = unsafe { *(str_field as *const *const u32) };

    // Collect all chars up to null terminator
    let mut chars: Vec<char> = vec![];
    for i in 0.. {
        let c = unsafe {
            match char::from_u32(*str.offset(i)) {
                Some(value) => value,
                None => {
                    println!("Invalid UTF-32 character");
                    '\0'
                }
            }
        };

        if c == '\0' {
            break;
        }

        chars.push(c);
    }
    let line: String = chars.iter().collect();

    if let Err(e) = write_to_stream(&line) {
        println!("Could not write line to socket: {:?}", e);
    }

    // Call the original function
    unsafe { PyUnicodeUcs4FormatHook.call(format, args) }
}

unsafe extern "C" fn main() {
    // Find the real PyUnicodeUCS4_Format function
    let symbol = CString::new("PyUnicodeUCS4_Format").unwrap();
    let symbol_addr = dlsym(RTLD_DEFAULT, symbol.as_ptr());
    let f_ptr: PyUnicodeUcs4Format = std::mem::transmute(symbol_addr);

    // Hook it
    let hook_result = PyUnicodeUcs4FormatHook
        .initialize(f_ptr, py_unicode_ucs4_format_detour)
        .and_then(|hook| hook.enable());

    if let Err(e) = hook_result {
        println!("Couldn't hook PyUnicodeUCS4_Format: {:?}", e);
    }
}

// Equivalent to __attribute__((constructor))
#[used]
#[link_section = ".ctors"]
pub static INITIALIZE: unsafe extern "C" fn() = main;
