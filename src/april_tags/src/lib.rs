#[no_mangle]
pub unsafe extern fn alloc_u8() -> usize {
    let layout = std::alloc::Layout::new::<u8>();
    let ptr = std::alloc::alloc(layout);

    return ptr as usize;
}

#[no_mangle]
pub unsafe extern fn dealloc_u8(ptr: usize) {
    let layout = std::alloc::Layout::new::<u8>();
    std::alloc::dealloc(ptr as *mut u8, layout);
}

mod wasm_imports {
    extern {
        pub fn console_log(ptr: usize, len: usize);
    }
}

fn console_log(text: &str) { unsafe { wasm_imports::console_log(text.as_ptr() as usize, text.len()); } }

macro_rules! wprintln {
    ($fstr:literal, $($arg:expr),+ $(,)?) => {
        crate::console_log(&format!($fstr, $($arg),+ ))
    };

    ($fstr:literal) => {
        crate::console_log(&format!($fstr))
    };
}

#[no_mangle]
pub extern fn process_greyscale_pixels(raw_pixels: usize, width: usize, height: usize) {
    let pixels = unsafe { std::slice::from_raw_parts_mut(raw_pixels as *mut u8, width*height) };

    wprintln!("Test");
}
