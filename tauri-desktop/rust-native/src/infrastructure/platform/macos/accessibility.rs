use std::ptr::NonNull;

use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXCopyMultipleAttributeOptions, AXError, AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions, AXUIElement, AXValue, AXValueType,
};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFDictionary, CFRetained, CFString, CFType, CGPoint, CGSize,
};

use crate::domain::input_source::{AxAttribute, AxElementSnapshot};

use super::super::focus_monitor::{
    AttributePlan, EXPENSIVE_ATTRIBUTE_MAX_DEPTH, FINGERPRINT_IDENTITY_DEPTH, MAX_PARENT_DEPTH,
};

const FOCUSED_ELEMENT: &str = "AXFocusedUIElement";
const PARENT: &str = "AXParent";
const POSITION: &str = "AXPosition";
const SIZE: &str = "AXSize";
/// FOCUSED_ELEMENT 取得専用の短い上限。失敗時（elements=0）に 0.15s 近く待つのを避ける。
const FOCUSED_PROBE_TIMEOUT_SECONDS: f32 = 0.05;
/// 親ウォーク / enrich の AX IPC ハング上限（秒）。
const MESSAGING_TIMEOUT_SECONDS: f32 = 0.15;

fn is_expensive_attribute(attribute: AxAttribute) -> bool {
    matches!(
        attribute,
        AxAttribute::Description
            | AxAttribute::RoleDescription
            | AxAttribute::Title
            | AxAttribute::Help
            | AxAttribute::Placeholder
    )
}

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

pub fn request_process_trust() -> bool {
    let prompt = unsafe { kAXTrustedCheckOptionPrompt };
    let enabled = CFBoolean::new(true);
    let options = CFDictionary::from_slices(&[prompt], &[enabled]);
    // SAFETY: type erasure preserves the CFDictionary object and its entries.
    let options: CFRetained<CFDictionary> = unsafe { CFRetained::cast_unchecked(options) };
    // SAFETY: the dictionary uses the documented CFString key and CFBoolean value.
    unsafe { AXIsProcessTrustedWithOptions(Some(&options)) }
}

pub fn application_element(pid: i32) -> CFRetained<AXUIElement> {
    // SAFETY: AXUIElementCreateApplication accepts any process identifier and returns an owned ref.
    unsafe { AXUIElement::new_application(pid) }
}

pub fn snapshot_elements(
    application: &AXUIElement,
    plan: &AttributePlan,
) -> Vec<AxElementSnapshot> {
    snapshot_elements_with_depth(application, plan, MAX_PARENT_DEPTH)
}

/// 親チェーンの構造打ち切り: Window / Application まで記録したらそれ以上辿らない。
pub(crate) fn is_chain_stop_role(role: &str) -> bool {
    matches!(role, "AXWindow" | "AXApplication")
}

/// fingerprint 判定に使う局所属性（深い祖先では第一パスで省略可）。
fn is_identity_local_attribute(attribute: AxAttribute) -> bool {
    matches!(
        attribute,
        AxAttribute::Identifier | AxAttribute::DomIdentifier | AxAttribute::DomClassList
    )
}

/// 親ウォーク結果を保持し、第二パスで属性だけ足せるようにする。
pub struct RetainedAxChain {
    snapshots: Vec<AxElementSnapshot>,
    nodes: Vec<CFRetained<AXUIElement>>,
}

impl RetainedAxChain {
    pub fn snapshots(&self) -> &[AxElementSnapshot] {
        &self.snapshots
    }

    pub fn into_snapshots(self) -> Vec<AxElementSnapshot> {
        self.snapshots
    }

    /// 未取得の属性だけ足す（親の再ウォークはしない）。
    pub fn enrich(&mut self, attributes: &[AxAttribute]) {
        if attributes.is_empty() {
            return;
        }
        if let Some(node) = self.nodes.first() {
            apply_messaging_timeout(node, MESSAGING_TIMEOUT_SECONDS);
        }
        for (depth, (snapshot, node)) in
            self.snapshots.iter_mut().zip(self.nodes.iter()).enumerate()
        {
            let mut pending_attrs = Vec::new();
            let mut pending_names = Vec::new();
            for attribute in attributes {
                if is_expensive_attribute(*attribute) && depth > EXPENSIVE_ATTRIBUTE_MAX_DEPTH {
                    continue;
                }
                if !snapshot.value(*attribute).is_empty() {
                    continue;
                }
                pending_attrs.push(*attribute);
                pending_names.push(attribute_name(*attribute));
            }
            if pending_names.is_empty() {
                continue;
            }
            let values = copy_attribute_texts(node, &pending_names);
            for (attribute, value) in pending_attrs.into_iter().zip(values) {
                if let Some(value) = value {
                    *snapshot = std::mem::take(snapshot).with(attribute, value);
                }
            }
        }
    }

    /// focused 取得済みチェーンから親だけ伸ばす（`AXFocusedUIElement` の再 IPC をしない）。
    pub fn extend_parent_walk(
        mut self,
        plan: &AttributePlan,
        max_depth: usize,
        shallow_identity_attrs: bool,
    ) -> Self {
        let max_depth = max_depth.clamp(1, MAX_PARENT_DEPTH);
        if self.nodes.is_empty() || self.snapshots.len() >= max_depth {
            return self;
        }
        let last_role = self
            .snapshots
            .last()
            .map(|snapshot| snapshot.value(AxAttribute::Role).to_string())
            .unwrap_or_default();
        if is_chain_stop_role(&last_role) {
            return self;
        }
        let Some(start) = self.nodes.last().cloned() else {
            return self;
        };
        apply_messaging_timeout(&start, MESSAGING_TIMEOUT_SECONDS);
        let Some(mut current) = copy_element(&start, PARENT) else {
            return self;
        };
        for depth in self.snapshots.len()..max_depth {
            push_chain_node(
                &mut self.snapshots,
                &mut self.nodes,
                &current,
                plan,
                depth,
                shallow_identity_attrs,
            );
            let role_for_stop = self
                .snapshots
                .last()
                .map(|snapshot| snapshot.value(AxAttribute::Role).to_string())
                .unwrap_or_default();
            let role_for_stop = if role_for_stop.is_empty() {
                copy_attribute_text(&current, attribute_name(AxAttribute::Role)).unwrap_or_default()
            } else {
                role_for_stop
            };
            if is_chain_stop_role(&role_for_stop) {
                break;
            }
            let Some(parent) = copy_element(&current, PARENT) else {
                break;
            };
            current = parent;
        }
        self
    }
}

pub fn snapshot_elements_with_depth(
    application: &AXUIElement,
    plan: &AttributePlan,
    max_depth: usize,
) -> Vec<AxElementSnapshot> {
    snapshot_chain_with_depth(application, plan, max_depth, false).into_snapshots()
}

/// `shallow_identity_attrs`: true なら depth >= FINGERPRINT_IDENTITY_DEPTH で
/// Dom* / Identifier を読まない（fingerprint 第一パス用）。
pub fn snapshot_chain_with_depth(
    application: &AXUIElement,
    plan: &AttributePlan,
    max_depth: usize,
    shallow_identity_attrs: bool,
) -> RetainedAxChain {
    // FOCUSED は短い timeout。空フォーカスで 150ms 近くブロックしない。
    apply_messaging_timeout(application, FOCUSED_PROBE_TIMEOUT_SECONDS);
    let max_depth = max_depth.clamp(1, MAX_PARENT_DEPTH);
    let Some(mut current) = copy_element(application, FOCUSED_ELEMENT) else {
        return RetainedAxChain {
            snapshots: Vec::new(),
            nodes: Vec::new(),
        };
    };
    apply_messaging_timeout(application, MESSAGING_TIMEOUT_SECONDS);
    let mut snapshots = Vec::with_capacity(max_depth);
    let mut nodes = Vec::with_capacity(max_depth);
    for depth in 0..max_depth {
        push_chain_node(
            &mut snapshots,
            &mut nodes,
            &current,
            plan,
            depth,
            shallow_identity_attrs,
        );
        let role_for_stop = snapshots
            .last()
            .map(|snapshot| snapshot.value(AxAttribute::Role).to_string())
            .unwrap_or_default();
        let role_for_stop = if role_for_stop.is_empty() {
            copy_attribute_text(&current, attribute_name(AxAttribute::Role)).unwrap_or_default()
        } else {
            role_for_stop
        };
        if is_chain_stop_role(&role_for_stop) {
            break;
        }
        let Some(parent) = copy_element(&current, PARENT) else {
            break;
        };
        current = parent;
    }
    RetainedAxChain { snapshots, nodes }
}

fn push_chain_node(
    snapshots: &mut Vec<AxElementSnapshot>,
    nodes: &mut Vec<CFRetained<AXUIElement>>,
    current: &AXUIElement,
    plan: &AttributePlan,
    depth: usize,
    shallow_identity_attrs: bool,
) {
    let mut attrs = Vec::new();
    let mut names = Vec::new();
    for attribute in plan.attributes() {
        if is_expensive_attribute(*attribute) && depth > EXPENSIVE_ATTRIBUTE_MAX_DEPTH {
            continue;
        }
        if shallow_identity_attrs
            && is_identity_local_attribute(*attribute)
            && depth >= FINGERPRINT_IDENTITY_DEPTH
        {
            continue;
        }
        attrs.push(*attribute);
        names.push(attribute_name(*attribute));
    }
    let values = copy_attribute_texts(current, &names);
    let mut snapshot = AxElementSnapshot::new();
    for (attribute, value) in attrs.into_iter().zip(values) {
        if let Some(value) = value {
            snapshot = snapshot.with(attribute, value);
        }
    }
    snapshots.push(snapshot);
    nodes.push(current.into());
}

/// Focused element frame in AX / Core Graphics global coordinates.
///
/// Per `kAXPositionAttribute`: origin is the **top-left** of the display that
/// shows the menu bar; X increases right, Y increases **downward** (pixels).
/// This matches Tauri `LogicalPosition` (top-left), so no Y-flip is applied.
pub fn focused_element_frame(pid: i32) -> Option<ElementFrame> {
    let application = application_element(pid);
    apply_messaging_timeout(&application, FOCUSED_PROBE_TIMEOUT_SECONDS);
    let focused = copy_element(&application, FOCUSED_ELEMENT)?;
    apply_messaging_timeout(&application, MESSAGING_TIMEOUT_SECONDS);
    let position = copy_ax_point(&focused, POSITION)?;
    let size = copy_ax_size(&focused, SIZE)?;
    Some(ElementFrame {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn apply_messaging_timeout(element: &AXUIElement, seconds: f32) {
    // SAFETY: timeout は正の秒数。失敗しても読み取りは続行する。
    let _ = unsafe { element.set_messaging_timeout(seconds) };
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
    cf_type_to_attribute_text(copy_value(element, name)?)
}

/// 同一要素の複数属性を 1 回の AX IPC で取得する（欠落属性で全体失敗しない）。
fn copy_attribute_texts(element: &AXUIElement, names: &[&'static str]) -> Vec<Option<String>> {
    if names.is_empty() {
        return Vec::new();
    }
    if names.len() == 1 {
        return vec![copy_attribute_text(element, names[0])];
    }
    let name_refs: Vec<CFRetained<CFString>> = names
        .iter()
        .map(|name| CFString::from_static_str(name))
        .collect();
    let attributes = CFArray::from_retained_objects(&name_refs);
    let mut values_ptr: *const CFArray = std::ptr::null();
    // SAFETY: attributes は CFString の CFArray。options=0 で欠落を CFNull/AXError として返す。
    let error = unsafe {
        element.copy_multiple_attribute_values(
            attributes.as_ref(),
            AXCopyMultipleAttributeOptions(0),
            NonNull::from(&mut values_ptr),
        )
    };
    if error != AXError::Success {
        return names
            .iter()
            .map(|name| copy_attribute_text(element, name))
            .collect();
    }
    let Some(values_ptr) = NonNull::new(values_ptr.cast_mut()) else {
        return names.iter().map(|_| None).collect();
    };
    // SAFETY: 成功時は create-rule の CFArray。
    let values = unsafe { CFRetained::from_raw(values_ptr) };
    // SAFETY: 各要素は CFType（CFString / CFArray / CFNull / AXValue エラー）。
    let values = unsafe { CFRetained::cast_unchecked::<CFArray<CFType>>(values) };
    let mut out = Vec::with_capacity(names.len());
    for index in 0..names.len() {
        let text = values.get(index).and_then(cf_type_to_attribute_text);
        out.push(text);
    }
    out
}

fn cf_type_to_attribute_text(value: CFRetained<CFType>) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::is_chain_stop_role;

    #[test]
    fn chain_stop_role_matches_window_and_application() {
        let cases = [
            ("AXWindow", true),
            ("AXApplication", true),
            ("AXTextField", false),
            ("AXGroup", false),
            ("AXWebArea", false),
            ("", false),
            ("AXWindowExtra", false),
        ];
        for (role, expected) in cases {
            assert_eq!(
                is_chain_stop_role(role),
                expected,
                "role={role:?} expected={expected}"
            );
        }
    }
}
