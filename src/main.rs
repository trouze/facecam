use objc2::{rc::Retained, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor,
    NSEventModifierFlags, NSFloatingWindowLevel, NSMenu, NSMenuItem, NSWindow,
    NSWindowStyleMask,
};
use objc2_foundation::{ns_string, NSPoint, NSRect, NSSize};

mod camera;

const BUBBLE_SIZE: f64 = 200.0;
const APP_NAME: &str = "facecam";

fn main() {
    let mtm = MainThreadMarker::new().expect("facecam must run on the main thread");

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    app.setMainMenu(Some(&build_menu_bar(mtm)));

    let window = build_window(mtm);

    let bubble = camera::camera_bubble_view(mtm);
    window.setContentView(Some(&bubble));

    window.center();
    window.makeKeyAndOrderFront(None);
    app.activate();

    app.run();
}

fn build_window(mtm: MainThreadMarker) -> Retained<NSWindow> {
    unsafe {
        let window = NSWindow::initWithContentRect_styleMask_backing_defer_screen(
            NSWindow::alloc(mtm),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(BUBBLE_SIZE, BUBBLE_SIZE),
            ),
            NSWindowStyleMask::Borderless | NSWindowStyleMask::FullSizeContentView,
            NSBackingStoreType::Buffered,
            false,
            None,
        );

        window.setOpaque(false);
        window.setBackgroundColor(Some(&NSColor::clearColor()));
        window.setHasShadow(true);
        window.setLevel(NSFloatingWindowLevel);

        window
    }
}

/// Minimal menu bar: a single app submenu with Hide and Quit, so Cmd+Q works.
fn build_menu_bar(mtm: MainThreadMarker) -> Retained<NSMenu> {
    let main_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));

    let app_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            ns_string!(APP_NAME),
            None,
            ns_string!(""),
        )
    };

    let app_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(APP_NAME));

    let hide_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            ns_string!("Hide facecam"),
            Some(sel!(hide:)),
            ns_string!("h"),
        )
    };
    hide_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    app_menu.addItem(&hide_item);

    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            ns_string!("Quit facecam"),
            Some(sel!(terminate:)),
            ns_string!("q"),
        )
    };
    quit_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    app_menu.addItem(&quit_item);

    app_item.setSubmenu(Some(&app_menu));
    main_menu.addItem(&app_item);

    main_menu
}
