//! 別プロセスから TIS enabled-only を列挙する（同一プロセス内キャッシュ回避）。
//! 出力: 1 行 1 InputSourceID（メニューバー相当の Layout / InputMode のみ）。

#[cfg(target_os = "macos")]
fn main() {
    macos::run();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("tis-list-enabled is macOS-only");
    std::process::exit(1);
}

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;
    use std::os::raw::{c_char, c_long};
    use std::ptr::null;

    type CFIndex = c_long;

    #[link(name = "Carbon", kind = "framework")]
    unsafe extern "C" {
        static kTISPropertyInputSourceID: *const c_void;
        static kTISPropertyInputSourceCategory: *const c_void;
        static kTISPropertyInputSourceType: *const c_void;
        static kTISCategoryKeyboardInputSource: *const c_void;
        static kTISTypeKeyboardLayout: *const c_void;
        static kTISTypeKeyboardInputMode: *const c_void;

        fn TISCreateInputSourceList(
            properties: *const c_void,
            include_all_installed: u8,
        ) -> *const c_void;
        fn TISGetInputSourceProperty(
            input_source: *const c_void,
            property_key: *const c_void,
        ) -> *const c_void;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(value: *const c_void);
        fn CFArrayGetCount(array: *const c_void) -> CFIndex;
        fn CFArrayGetValueAtIndex(array: *const c_void, index: CFIndex) -> *const c_void;
        fn CFEqual(a: *const c_void, b: *const c_void) -> u8;
        fn CFStringGetLength(string: *const c_void) -> CFIndex;
        fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
        fn CFStringGetCString(
            string: *const c_void,
            buffer: *mut c_char,
            buffer_size: CFIndex,
            encoding: u32,
        ) -> u8;
    }

    const UTF8_ENCODING: u32 = 0x0800_0100;

    pub(super) fn run() {
        // SAFETY: null properties / false = currently enabled sources.
        let array = unsafe { TISCreateInputSourceList(null(), 0) };
        if array.is_null() {
            std::process::exit(1);
        }
        // SAFETY: create-rule array.
        let count = unsafe { CFArrayGetCount(array) };
        for index in 0..count {
            // SAFETY: index within count.
            let raw = unsafe { CFArrayGetValueAtIndex(array, index) };
            if !is_menu_bar_keyboard_source(raw) {
                continue;
            }
            // SAFETY: property constants are process-lifetime.
            let id_ref = unsafe { TISGetInputSourceProperty(raw, kTISPropertyInputSourceID) };
            if let Some(id) = cfstring_to_string(id_ref) {
                println!("{id}");
            }
        }
        // SAFETY: balances TISCreateInputSourceList.
        unsafe { CFRelease(array) };
    }

    fn is_menu_bar_keyboard_source(source: *const c_void) -> bool {
        unsafe {
            let category = TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory);
            let source_type = TISGetInputSourceProperty(source, kTISPropertyInputSourceType);
            if category.is_null() || source_type.is_null() {
                return false;
            }
            if CFEqual(category, kTISCategoryKeyboardInputSource) == 0 {
                return false;
            }
            CFEqual(source_type, kTISTypeKeyboardLayout) != 0
                || CFEqual(source_type, kTISTypeKeyboardInputMode) != 0
        }
    }

    fn cfstring_to_string(value: *const c_void) -> Option<String> {
        if value.is_null() {
            return None;
        }
        let length = unsafe { CFStringGetLength(value) };
        let capacity = unsafe { CFStringGetMaximumSizeForEncoding(length, UTF8_ENCODING) } + 1;
        let capacity = usize::try_from(capacity).ok()?;
        let mut buffer = vec![0_i8; capacity];
        let success = unsafe {
            CFStringGetCString(
                value,
                buffer.as_mut_ptr(),
                CFIndex::try_from(capacity).ok()?,
                UTF8_ENCODING,
            )
        };
        if success == 0 {
            return None;
        }
        let bytes = buffer
            .into_iter()
            .take_while(|byte| *byte != 0)
            .map(|byte| byte.to_ne_bytes()[0])
            .collect::<Vec<_>>();
        String::from_utf8(bytes).ok()
    }
}
