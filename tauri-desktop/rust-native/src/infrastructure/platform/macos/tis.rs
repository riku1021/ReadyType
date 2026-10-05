use std::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use objc2::MainThreadMarker;

use crate::{
    application::ports::input_source::GatewayError,
    domain::input_source::InputSourceId,
    infrastructure::platform::{
        input_source::{NativeInputSource, TisBackend},
        macos::input_source_display_name,
    },
};

type CFIndex = isize;
type DispatchTime = u64;

const UTF8_ENCODING: u32 = 0x0800_0100;
const DISPATCH_TIME_NOW: DispatchTime = 0;
/// Electron 等でメニューだけ更新され実入力が残るケース向けの再選択タイミング。
const FOLLOWUP_RESELECT_DELAYS_MS: [u64; 3] = [40, 120, 250];
static FOLLOWUP_EPOCH: AtomicU64 = AtomicU64::new(0);

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyInputSourceID: *const c_void;
    static kTISPropertyLocalizedName: *const c_void;
    static kTISPropertyBundleID: *const c_void;
    static kTISPropertyInputModeID: *const c_void;
    static kTISPropertyIconImageURL: *const c_void;
    static kTISPropertyInputSourceLanguages: *const c_void;
    static kTISPropertyInputSourceIsEnabled: *const c_void;
    static kTISPropertyInputSourceIsSelectCapable: *const c_void;
    static kTISPropertyInputSourceCategory: *const c_void;
    static kTISPropertyInputSourceType: *const c_void;
    static kTISCategoryKeyboardInputSource: *const c_void;
    static kTISTypeKeyboardLayout: *const c_void;
    static kTISTypeKeyboardInputMode: *const c_void;

    fn TISCopyCurrentKeyboardInputSource() -> *const c_void;
    fn TISCreateInputSourceList(
        properties: *const c_void,
        include_all_installed: u8,
    ) -> *const c_void;
    fn TISGetInputSourceProperty(
        input_source: *const c_void,
        property_key: *const c_void,
    ) -> *const c_void;
    fn TISSelectInputSource(input_source: *const c_void) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRetain(value: *const c_void) -> *const c_void;
    fn CFRelease(value: *const c_void);
    fn CFArrayGetCount(array: *const c_void) -> CFIndex;
    fn CFArrayGetValueAtIndex(array: *const c_void, index: CFIndex) -> *const c_void;
    fn CFBooleanGetValue(value: *const c_void) -> u8;
    fn CFEqual(a: *const c_void, b: *const c_void) -> u8;
    fn CFStringGetLength(string: *const c_void) -> CFIndex;
    fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
    fn CFStringGetCString(
        string: *const c_void,
        buffer: *mut i8,
        buffer_size: CFIndex,
        encoding: u32,
    ) -> u8;
}

// `_dispatch_main_q` is the backing storage for `dispatch_get_main_queue()`.
#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_sync_f(
        queue: *const c_void,
        context: *mut c_void,
        work: Option<unsafe extern "C" fn(*mut c_void)>,
    );
    fn dispatch_after_f(
        when: DispatchTime,
        queue: *const c_void,
        context: *mut c_void,
        work: Option<unsafe extern "C" fn(*mut c_void)>,
    );
    fn dispatch_time(when: DispatchTime, delta: i64) -> DispatchTime;
}

#[derive(Debug)]
pub struct OwnedTisInputSource(NonNull<c_void>);

impl OwnedTisInputSource {
    unsafe fn from_create_rule(pointer: *const c_void) -> Result<Self, GatewayError> {
        NonNull::new(pointer.cast_mut())
            .map(Self)
            .ok_or_else(|| GatewayError::Operation("TIS returned a null input source".to_owned()))
    }

    unsafe fn retain(pointer: *const c_void) -> Result<Self, GatewayError> {
        let pointer = NonNull::new(pointer.cast_mut()).ok_or_else(|| {
            GatewayError::Operation("TIS returned a null input source".to_owned())
        })?;
        // SAFETY: `pointer` is a non-null CF object borrowed from a live CFArray.
        unsafe { CFRetain(pointer.as_ptr()) };
        Ok(Self(pointer))
    }

    fn as_ptr(&self) -> *const c_void {
        self.0.as_ptr()
    }
}

impl Clone for OwnedTisInputSource {
    fn clone(&self) -> Self {
        // SAFETY: self owns a valid retained CF object.
        unsafe { CFRetain(self.as_ptr()) };
        Self(self.0)
    }
}

impl Drop for OwnedTisInputSource {
    fn drop(&mut self) {
        // SAFETY: this balances the create/retain operation that produced this owner.
        unsafe { CFRelease(self.as_ptr()) };
    }
}

// Handles are created/used on the main queue and only moved between threads as opaque owners.
unsafe impl Send for OwnedTisInputSource {}

struct OwnedCfArray(NonNull<c_void>);

impl OwnedCfArray {
    unsafe fn from_create_rule(pointer: *const c_void) -> Result<Self, GatewayError> {
        NonNull::new(pointer.cast_mut())
            .map(Self)
            .ok_or_else(|| GatewayError::Operation("TIS returned a null source list".to_owned()))
    }
}

impl Drop for OwnedCfArray {
    fn drop(&mut self) {
        // SAFETY: this balances TISCreateInputSourceList's create-rule return.
        unsafe { CFRelease(self.0.as_ptr()) };
    }
}

pub struct SystemTisBackend;

impl TisBackend for SystemTisBackend {
    type Handle = OwnedTisInputSource;

    fn list(&mut self, locale: &str) -> Result<Vec<NativeInputSource<Self::Handle>>, GatewayError> {
        let locale = locale.to_owned();
        run_on_main_thread(move || list_on_main(&locale))
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        run_on_main_thread(current_on_main)
    }

    fn select(&mut self, handle: &Self::Handle) -> Result<(), GatewayError> {
        let handle = handle.clone();
        run_on_main_thread(move || select_on_main(&handle))
    }
}

fn list_on_main(locale: &str) -> Result<Vec<NativeInputSource<OwnedTisInputSource>>, GatewayError> {
    // SAFETY: null properties means no filter; false requests enabled sources only.
    let array =
        unsafe { OwnedCfArray::from_create_rule(TISCreateInputSourceList(std::ptr::null(), 0))? };
    // SAFETY: array owns a valid CFArray for this scope.
    let count = unsafe { CFArrayGetCount(array.0.as_ptr()) };
    let mut sources = Vec::with_capacity(usize::try_from(count).unwrap_or_default());
    for index in 0..count {
        // SAFETY: index is within the count returned for this valid CFArray.
        let raw = unsafe { CFArrayGetValueAtIndex(array.0.as_ptr(), index) };
        // メニューバーの入力ソース一覧に合わせ、キーボードレイアウト／入力モード以外は除外する。
        if !is_menu_bar_keyboard_source(raw) {
            continue;
        }
        // SAFETY: values in the TIS array are TISInputSource CF objects.
        let handle = unsafe { OwnedTisInputSource::retain(raw)? };
        let Some(id) = property_string(raw, unsafe { kTISPropertyInputSourceID }) else {
            continue;
        };
        let fallback_name = property_string(raw, unsafe { kTISPropertyLocalizedName })
            .unwrap_or_else(|| id.clone());
        let bundle_id = property_string(raw, unsafe { kTISPropertyBundleID });
        let mode_id = property_string(raw, unsafe { kTISPropertyInputModeID });
        let icon_path = {
            let url = unsafe { TISGetInputSourceProperty(raw, kTISPropertyIconImageURL) };
            input_source_display_name::path_from_cfurl(url)
        };
        let name = input_source_display_name::resolve_display_name(
            locale,
            &id,
            bundle_id.as_deref(),
            mode_id.as_deref(),
            icon_path.as_deref(),
            &fallback_name,
        );
        let language_code = primary_language_code(raw);
        let enabled =
            property_bool(raw, unsafe { kTISPropertyInputSourceIsEnabled }).unwrap_or(false);
        let selectable =
            property_bool(raw, unsafe { kTISPropertyInputSourceIsSelectCapable }).unwrap_or(false);
        sources.push(NativeInputSource {
            id: InputSourceId::parse(&id)
                .map_err(|error| GatewayError::Operation(error.to_string()))?,
            name,
            language_code,
            enabled,
            selectable,
            handle,
        });
    }
    Ok(sources)
}

/// メニューバー「入力ソース」相当: キーボードカテゴリかつ Layout / InputMode のみ。
/// Dictation・絵文字パレット・PressAndHold などは除外する。
fn is_menu_bar_keyboard_source(source: *const c_void) -> bool {
    // SAFETY: TIS category/type constants and properties are process-lifetime CFStrings.
    let category = unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory) };
    let source_type = unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceType) };
    if category.is_null() || source_type.is_null() {
        return false;
    }
    let is_keyboard = unsafe { CFEqual(category, kTISCategoryKeyboardInputSource) != 0 };
    if !is_keyboard {
        return false;
    }
    unsafe {
        CFEqual(source_type, kTISTypeKeyboardLayout) != 0
            || CFEqual(source_type, kTISTypeKeyboardInputMode) != 0
    }
}

fn current_on_main() -> Result<InputSourceId, GatewayError> {
    // SAFETY: the returned object follows the create rule and is released by the owner.
    let source =
        unsafe { OwnedTisInputSource::from_create_rule(TISCopyCurrentKeyboardInputSource())? };
    let id = property_string(source.as_ptr(), unsafe { kTISPropertyInputSourceID })
        .ok_or_else(|| GatewayError::Operation("current input source has no ID".to_owned()))?;
    InputSourceId::parse(&id).map_err(|error| GatewayError::Operation(error.to_string()))
}

fn select_on_main(handle: &OwnedTisInputSource) -> Result<(), GatewayError> {
    // SAFETY: handle owns a live TISInputSource object.
    let status = unsafe { TISSelectInputSource(handle.as_ptr()) };
    if status != 0 {
        return Err(GatewayError::Operation(format!(
            "TISSelectInputSource returned OSStatus {status}"
        )));
    }
    // メニューバーだけ先に変わり、フォーカス中フィールドの入力コンテキストが遅れる対策。
    schedule_followup_reselects(handle.clone());
    Ok(())
}

fn schedule_followup_reselects(handle: OwnedTisInputSource) {
    let epoch = FOLLOWUP_EPOCH.fetch_add(1, Ordering::AcqRel) + 1;
    for delay_ms in FOLLOWUP_RESELECT_DELAYS_MS {
        let handle = handle.clone();
        dispatch_after_ms(delay_ms, move || {
            if FOLLOWUP_EPOCH.load(Ordering::Acquire) != epoch {
                return;
            }
            // SAFETY: handle keeps the TISInputSource alive across the delayed callback.
            let _ = unsafe { TISSelectInputSource(handle.as_ptr()) };
        });
    }
}

fn dispatch_after_ms<F>(delay_ms: u64, work: F)
where
    F: FnOnce() + Send + 'static,
{
    struct Context<F> {
        work: Option<F>,
    }

    unsafe extern "C" fn invoke<F: FnOnce()>(context: *mut c_void) {
        // SAFETY: context is the Box allocated exclusively for this delayed callback.
        let mut context = unsafe { Box::from_raw(context.cast::<Context<F>>()) };
        if let Some(work) = context.work.take() {
            work();
        }
    }

    let delta_ns = i64::try_from(delay_ms.saturating_mul(1_000_000)).unwrap_or(i64::MAX);
    // SAFETY: DISPATCH_TIME_NOW + nanosecond delta is the documented dispatch_time usage.
    let when = unsafe { dispatch_time(DISPATCH_TIME_NOW, delta_ns) };
    let context = Box::into_raw(Box::new(Context { work: Some(work) }));
    // SAFETY: main queue is process-wide; invoke frees the context exactly once.
    unsafe {
        dispatch_after_f(
            when,
            std::ptr::addr_of!(_dispatch_main_q),
            context.cast(),
            Some(invoke::<F>),
        );
    }
}

/// macOS 26+ の TIS / HIToolbox は main dispatch queue 上でのみ安全。
fn run_on_main_thread<F, R>(work: F) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    if MainThreadMarker::new().is_some() {
        return work();
    }

    struct Context<F, R> {
        work: Option<F>,
        result: MaybeUninit<R>,
    }

    unsafe extern "C" fn invoke<F, R>(context: *mut c_void)
    where
        F: FnOnce() -> R,
    {
        // SAFETY: context is the exclusive Context allocated by the caller for this sync call.
        let context = unsafe { &mut *context.cast::<Context<F, R>>() };
        let work = context.work.take().expect("TIS main-thread work missing");
        context.result.write(work());
    }

    let mut context = Context {
        work: Some(work),
        result: MaybeUninit::uninit(),
    };
    // SAFETY: `_dispatch_main_q` is the process main queue; invoke only touches `context`.
    unsafe {
        dispatch_sync_f(
            std::ptr::addr_of!(_dispatch_main_q),
            std::ptr::addr_of_mut!(context).cast(),
            Some(invoke::<F, R>),
        );
    }
    // SAFETY: invoke wrote the result before dispatch_sync_f returned.
    unsafe { context.result.assume_init() }
}

fn property_string(source: *const c_void, key: *const c_void) -> Option<String> {
    // SAFETY: source and key are valid TIS objects/constants; property is borrowed.
    let value = unsafe { TISGetInputSourceProperty(source, key) };
    if value.is_null() {
        return None;
    }
    cfstring_to_string(value)
}

fn primary_language_code(source: *const c_void) -> Option<String> {
    // SAFETY: source is a live TISInputSource; languages key is a process-lifetime constant.
    let languages = unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages) };
    if languages.is_null() {
        return None;
    }
    // SAFETY: property is documented as CFArray of CFString language codes.
    let count = unsafe { CFArrayGetCount(languages) };
    if count <= 0 {
        return None;
    }
    // SAFETY: index 0 is within the array length just checked.
    let first = unsafe { CFArrayGetValueAtIndex(languages, 0) };
    let code = cfstring_to_string(first)?;
    if code.is_empty() {
        None
    } else {
        Some(code)
    }
}

fn cfstring_to_string(value: *const c_void) -> Option<String> {
    if value.is_null() {
        return None;
    }
    // SAFETY: value is a CFString from TIS / CFArray.
    let length = unsafe { CFStringGetLength(value) };
    // SAFETY: length came from the same valid CFString.
    let capacity = unsafe { CFStringGetMaximumSizeForEncoding(length, UTF8_ENCODING) } + 1;
    let capacity = usize::try_from(capacity).ok()?;
    let mut buffer = vec![0_i8; capacity];
    // SAFETY: buffer has `capacity` bytes and value is a CFString.
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

fn property_bool(source: *const c_void, key: *const c_void) -> Option<bool> {
    // SAFETY: source and key are valid TIS objects/constants; property is borrowed.
    let value = unsafe { TISGetInputSourceProperty(source, key) };
    if value.is_null() {
        return None;
    }
    // SAFETY: requested properties are documented CFBoolean values.
    Some(unsafe { CFBooleanGetValue(value) } != 0)
}
