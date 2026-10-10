use std::ptr::NonNull;

use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXError, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    AXUIElement, AXValue, AXValueType,
};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFDictionary, CFRetained, CFString, CFType, CGPoint, CGSize,
};

use crate::domain::input_source::{AxAttribute, AxElementSnapshot};

use super::super::focus_monitor::{AttributePlan, MAX_PARENT_DEPTH};

const FOCUSED_ELEMENT: &str = "AXFocusedUIElement";
const PARENT: &str = "AXParent";
const POSITION: &str = "AXPosition";
const SIZE: &str = "AXSize";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElementFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn is_process_trusted() -> bool {
    // SAFETY: this function has no parameters and only reads the current process trust state.
    unsafe { AXIsProcessTrusted() }
}

/// TCC にこのプロセスを認識させ、現在の信頼状態を返す。
///
/// `kAXTrustedCheckOptionPrompt: true` は使わない。ad-hoc 署名や再ビルド後に
/// System Settings のトグルを OFF に戻す既知の不具合があるため。
pub fn request_process_trust() -> bool {
    let prompt = unsafe { kAXTrustedCheckOptionPrompt };
    let disabled = CFBoolean::new(false);
    let options = CFDictionary::from_slices(&[prompt], &[disabled]);
    // SAFETY: type erasure preserves the CFDictionary object and its entries.
    let options: CFRetained<CFDictionary> = unsafe { CFRetained::cast_unchecked(options) };
    // SAFETY: the dictionary uses the documented CFString key and CFBoolean value.
    // 戻り値はキャッシュ由来で古いことがあるので、直後に AXIsProcessTrusted を正とする。
    let _ = unsafe { AXIsProcessTrustedWithOptions(Some(&options)) };
    is_process_trusted()
}

pub fn application_element(pid: i32) -> CFRetained<AXUIElement> {
    // SAFETY: AXUIElementCreateApplication accepts any process identifier and returns an owned ref.
    unsafe { AXUIElement::new_application(pid) }
}

pub fn snapshot_elements(
    application: &AXUIElement,
    plan: &AttributePlan,
) -> Vec<AxElementSnapshot> {
    let Some(mut current) = copy_element(application, FOCUSED_ELEMENT) else {
        return Vec::new();
    };
    let mut result = Vec::with_capacity(MAX_PARENT_DEPTH);
    for _ in 0..MAX_PARENT_DEPTH {
        let mut snapshot = AxElementSnapshot::new();
        for attribute in plan.attributes() {
            if let Some(value) = copy_attribute_text(&current, attribute_name(*attribute)) {
                snapshot = snapshot.with(*attribute, value);
            }
        }
        result.push(snapshot);
        let Some(parent) = copy_element(&current, PARENT) else {
            break;
        };
        current = parent;
    }
    result
}

/// Focused element frame in AX / Core Graphics global coordinates.
///
/// Per `kAXPositionAttribute`: origin is the **top-left** of the display that
/// shows the menu bar; X increases right, Y increases **downward** (pixels).
/// This matches Tauri `LogicalPosition` (top-left), so no Y-flip is applied.
pub fn focused_element_frame(pid: i32) -> Option<ElementFrame> {
    let application = application_element(pid);
    let focused = copy_element(&application, FOCUSED_ELEMENT)?;
    let position = copy_ax_point(&focused, POSITION)?;
    let size = copy_ax_size(&focused, SIZE)?;
    Some(ElementFrame {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn attribute_name(attribute: AxAttribute) -> &'static str {
    match attribute {
        AxAttribute::Role => "AXRole",
        AxAttribute::Subrole => "AXSubrole",
        AxAttribute::RoleDescription => "AXRoleDescription",
        AxAttribute::Title => "AXTitle",
        AxAttribute::Description => "AXDescription",
        AxAttribute::Identifier => "AXIdentifier",
        AxAttribute::DomIdentifier => "AXDOMIdentifier",
        AxAttribute::DomClassList => "AXDOMClassList",
        AxAttribute::Help => "AXHelp",
        AxAttribute::Placeholder => "AXPlaceholderValue",
    }
}

fn copy_value(element: &AXUIElement, name: &'static str) -> Option<CFRetained<CFType>> {
    let name = CFString::from_static_str(name);
    let mut value = std::ptr::null();
    // SAFETY: `value` is a valid out pointer and the returned create-rule object is owned below.
    let error = unsafe { element.copy_attribute_value(&name, NonNull::from(&mut value)) };
    if error != AXError::Success {
        return None;
    }
    let value = NonNull::new(value.cast_mut())?;
    // SAFETY: a successful AX copy call returns a non-null create-rule CF object.
    Some(unsafe { CFRetained::from_raw(value) })
}

fn copy_element(element: &AXUIElement, name: &'static str) -> Option<CFRetained<AXUIElement>> {
    copy_value(element, name)?.downcast().ok()
}

fn copy_attribute_text(element: &AXUIElement, name: &'static str) -> Option<String> {
    let value = copy_value(element, name)?;
    if let Ok(string) = value.clone().downcast::<CFString>() {
        return Some(string.to_string());
    }
    let opaque = value.downcast::<CFArray>().ok()?;
    // SAFETY: AXDOMClassList is a CFArray of CFString values.
    let array = unsafe { CFRetained::cast_unchecked::<CFArray<CFString>>(opaque) };
    let mut parts = Vec::with_capacity(array.len());
    for index in 0..array.len() {
        if let Some(string) = array.get(index) {
            parts.push(string.to_string());
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn copy_ax_point(element: &AXUIElement, name: &'static str) -> Option<CGPoint> {
    let value = copy_value(element, name)?;
    let ax_value = value.downcast::<AXValue>().ok()?;
    let mut point = CGPoint::ZERO;
    // SAFETY: out pointer is a valid CGPoint; type matches AXPosition.
    let ok = unsafe { ax_value.value(AXValueType::CGPoint, NonNull::from(&mut point).cast()) };
    ok.then_some(point)
}

fn copy_ax_size(element: &AXUIElement, name: &'static str) -> Option<CGSize> {
    let value = copy_value(element, name)?;
    let ax_value = value.downcast::<AXValue>().ok()?;
    let mut size = CGSize::ZERO;
    // SAFETY: out pointer is a valid CGSize; type matches AXSize.
    let ok = unsafe { ax_value.value(AXValueType::CGSize, NonNull::from(&mut size).cast()) };
    ok.then_some(size)
}
