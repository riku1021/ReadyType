use super::{center_on_monitor, place_near_anchor, LogicalRect, GAP, WINDOW_HEIGHT, WINDOW_WIDTH};

#[test]
fn places_below_anchor_when_space_allows() {
    let monitor = LogicalRect {
        x: 1920.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    let anchor = LogicalRect {
        x: 2000.0,
        y: 100.0,
        width: 200.0,
        height: 24.0,
    };
    let (x, y) = place_near_anchor(anchor, monitor);
    assert_eq!(x, 2000.0);
    assert_eq!(y, 100.0 + 24.0 + GAP);
}

#[test]
fn flips_above_when_overflowing_bottom_of_secondary_monitor() {
    let monitor = LogicalRect {
        x: 1920.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    let anchor = LogicalRect {
        x: 2100.0,
        y: 1000.0,
        width: 120.0,
        height: 20.0,
    };
    let (x, y) = place_near_anchor(anchor, monitor);
    assert_eq!(x, 2100.0);
    assert_eq!(y, 1000.0 - WINDOW_HEIGHT - GAP);
    assert!(y >= monitor.y);
}

#[test]
fn clamps_horizontally_inside_secondary_monitor_not_primary() {
    let monitor = LogicalRect {
        x: 1920.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let anchor = LogicalRect {
        x: 2600.0,
        y: 40.0,
        width: 100.0,
        height: 20.0,
    };
    let (x, _y) = place_near_anchor(anchor, monitor);
    assert!(x + WINDOW_WIDTH <= monitor.x + monitor.width);
    assert!(x >= monitor.x);
}

#[test]
fn centers_on_given_monitor() {
    let monitor = LogicalRect {
        x: 1920.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    let (x, y) = center_on_monitor(monitor);
    assert!((x - (1920.0 + (1920.0 - WINDOW_WIDTH) / 2.0)).abs() < f64::EPSILON);
    assert!((y - (0.0 + (1080.0 - WINDOW_HEIGHT) / 2.0)).abs() < f64::EPSILON);
}

#[test]
fn logical_rect_contains_point_on_secondary() {
    let monitor = LogicalRect {
        x: 1920.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    assert!(monitor.contains(2000.0, 100.0));
    assert!(!monitor.contains(100.0, 100.0));
}
