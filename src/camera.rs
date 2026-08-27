use std::cell::{Cell, RefCell};

use objc2::{
    define_class, msg_send, rc::Retained, sel, AnyThread, ClassType, DefinedClass,
    MainThreadMarker, MainThreadOnly,
};
use objc2::runtime::AnyObject;
use objc2_app_kit::{
    NSColor, NSEvent, NSMenu, NSMenuItem, NSEventModifierFlags, NSTrackingArea,
    NSTrackingAreaOptions, NSView,
};
use objc2_av_foundation::{
    AVLayerVideoGravityResizeAspectFill, AVCaptureCenterStageControlMode, AVCaptureDevice,
    AVCaptureDeviceInput, AVCaptureSession, AVCaptureSessionPresetHigh, AVCaptureVideoPreviewLayer,
    AVMediaTypeVideo,
};
use objc2_core_graphics::CGColor;
use objc2_foundation::{ns_string, NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CALayer, CATransaction};

const BUBBLE_SIZE: f64 = 200.0;
const MIN_SIZE: f64 = 100.0;
const MAX_SIZE: f64 = 600.0;

/// How far (degrees) from a 45-degree diagonal a pointer may sit and still
/// count as the corner resize grip.
const CORNER_ANGLE_TOLERANCE_DEG: f64 = 30.0;

const RING_BORDER_WIDTH: f64 = 3.0;
const RESIZE_RING_OPACITY: f32 = 0.95;
const HOVER_RING_OPACITY: f32 = 0.25;

pub fn bubble_rect(size: f64) -> NSRect {
    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(size, size))
}

/// Which gesture the pointer is over.
#[derive(Clone, Copy, PartialEq)]
enum Grip {
    /// Inside the circle: press & drag moves the bubble.
    Move,
    /// Diagonal gap between circle and bounding box: `(right side, upper half)`
    /// in view coordinates; press & drag resizes from that corner.
    Corner(bool, bool),
    None,
}

#[derive(Clone, Copy)]
struct DragState {
    grip: Grip,
    start_frame: NSRect,
    start_size: f64,
    start_mouse: NSPoint,
}

/// Held (not only read) so the capture graph stays alive for the view's lifetime.
#[allow(dead_code)]
#[derive(Clone)]
pub struct BubbleIvars {
    session: Retained<AVCaptureSession>,
    preview: Retained<AVCaptureVideoPreviewLayer>,
    ring: Retained<CALayer>,
    drag: RefCell<Option<DragState>>,
    pointer_grip: Cell<Grip>,
}

define_class!(
    // SAFETY: NSView has no subclassing requirements, and BubbleView does not implement Drop.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = BubbleIvars]
    pub struct BubbleView;

    impl BubbleView {
        /// Keeps the preview and feedback ring filling the view, and the clip
        /// circle centered, as the bubble resizes.
        #[unsafe(method(layout))]
        fn layout(&self) {
            let bounds: NSRect = unsafe { msg_send![self, bounds] };
            let diameter = bounds.size.width.min(bounds.size.height);
            let radius = diameter / 2.0;

            // Disable implicit animations so layer changes apply instantly during
            // live resize (otherwise Core Animation animates frame/cornerRadius
            // from old→new, showing lagging perpendicular cut lines mid-drag).
            CATransaction::begin();
            CATransaction::setDisableActions(true);

            self.ivars().preview.setFrame(bounds);
            self.ivars().preview.setCornerRadius(radius);

            self.ivars().ring.setFrame(bounds);
            self.ivars().ring.setCornerRadius(radius);

            CATransaction::commit();

            unsafe { msg_send![super(self), layout] }
        }

        // We take full control of hit-testing so nothing is draggable or
        // resizable where there is no visible bubble.
        #[unsafe(method(mouseDownCanMoveWindow))]
        fn mouse_down_can_move_window(&self) -> bool {
            false
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            let Some(window) = self.window() else { return };

            match self.grip_at_pointer() {
                Grip::Move => window.performWindowDragWithEvent(event),
                grip @ Grip::Corner(..) => {
                    *self.ivars().drag.borrow_mut() = Some(DragState {
                        grip,
                        start_frame: window.frame(),
                        start_size: window.frame().size.width.min(window.frame().size.height),
                        start_mouse: window.convertPointToScreen(event.locationInWindow()),
                    });
                    self.update_ring_feedback();
                }
                Grip::None => {}
            }
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            // Use the grip latched at mouseDown — NOT a fresh grip_at_pointer()
            // call. Re-evaluating mid-drag would abort as soon as the growing
            // window moves the pointer inside the circle.
            let drag = self.ivars().drag.borrow();
            let Some(state) = drag.as_ref() else { return };
            let Grip::Corner(is_right, is_upper) = state.grip else { return };

            let Some(window) = self.window() else { return };

            let mouse = window.convertPointToScreen(event.locationInWindow());
            let dx = mouse.x - state.start_mouse.x;
            let dy = mouse.y - state.start_mouse.y; // screen coords: y grows upward

            // Uniform size driven by whichever axis leads.
            let grow_x = if is_right { dx } else { -dx };
            let grow_y = if is_upper { dy } else { -dy };
            let size = (state.start_size + grow_x.max(grow_y)).clamp(MIN_SIZE, MAX_SIZE);

            // Anchor the two edges opposite the grabbed corner:
            //   right corners keep the left edge; left corners keep the right.
            //   top  corners keep the bottom edge; bottom corners keep the top.
            let scale = size / state.start_size;
            let mut origin = state.start_frame.origin;
            if !is_right {
                origin.x += state.start_frame.size.width * (1.0 - scale);
            }
            if !is_upper {
                origin.y += state.start_frame.size.height * (1.0 - scale);
            }

            window.setFrame_display(rect_at(origin, size, size), true);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, _event: &NSEvent) {
            self.ivars().drag.borrow_mut().take();
            self.update_ring_feedback();
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, _event: &NSEvent) {
            self.update_ring_feedback();
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            self.ivars().pointer_grip.set(Grip::None);
            self.update_ring_feedback();
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, _event: &NSEvent) {
            self.ivars().pointer_grip.set(self.grip_at_pointer());
            self.update_ring_feedback();
        }
    }
);

fn rect_at(origin: NSPoint, width: f64, height: f64) -> NSRect {
    NSRect::new(origin, NSSize::new(width, height))
}
impl BubbleView {
    /// Classifies the current pointer position into an interaction grip:
    ///
    /// - inside the circle -> move grip
    /// - diagonal corner gaps between circle and box -> resize grips
    /// - everything else -> inert, so there are no phantom hot zones
    fn grip_at_pointer(&self) -> Grip {
        let Some(window) = self.window() else { return Grip::None };
        let local =
            self.convertPoint_fromView(window.mouseLocationOutsideOfEventStream(), None);
        let bounds: NSRect = unsafe { msg_send![self, bounds] };
        let center = NSSize::new(bounds.size.width / 2.0, bounds.size.height / 2.0);
        let dx = local.x - center.width;
        let dy = local.y - center.height; // view coords: y grows upward
        let distance = dx.hypot(dy);
        let radius = bounds.size.width.min(bounds.size.height) / 2.0;

        if distance <= radius * 0.97 {
            return Grip::Move;
        }
        if distance > radius + 16.0 {
            return Grip::None;
        }

        // In view coords, visually-upper half has y > center.
        let angle_off_diagonal = (dy.abs().atan2(dx.abs()).to_degrees() - 45.0).abs();
        if angle_off_diagonal > CORNER_ANGLE_TOLERANCE_DEG {
            return Grip::None;
        }

        Grip::Corner(dx > 0.0, dy > 0.0)
    }

    /// Ring feedback: faint halo when the pointer can grab something, solid
    /// ring while resizing. Hidden otherwise.
    fn update_ring_feedback(&self) {
        let target = if self.ivars().drag.borrow().is_some() {
            RESIZE_RING_OPACITY
        } else {
            match self.ivars().pointer_grip.get() {
                Grip::Move | Grip::Corner(..) => HOVER_RING_OPACITY,
                Grip::None => 0.0,
            }
        };
        self.ivars().ring.setOpacity(target);
    }
}

/// Creates the layer-backed view that hosts the circular webcam preview.
///
/// Mirrors CamBubble's `CameraView`: an aspect-fill, mirrored
/// `AVCaptureVideoPreviewLayer` clipped to a circle, with a right-click
/// "Quit facecam" menu. Extends it with custom hit-testing (circle moves the
/// bubble, corner gaps resize it) plus a visible feedback ring.
pub fn camera_bubble_view(mtm: MainThreadMarker) -> Retained<BubbleView> {
    let session: Retained<AVCaptureSession> =
        unsafe { msg_send![AVCaptureSession::class(), new] };

    unsafe {
        session.setSessionPreset(AVCaptureSessionPresetHigh);

        // Disable Center Stage so the camera uses its full wide field of view.
        let device_class = AVCaptureDevice::class();
        if device_class.responds_to(sel!(setCenterStageControlMode:))
            && device_class.responds_to(sel!(setCenterStageEnabled:))
        {
            AVCaptureDevice::setCenterStageControlMode(AVCaptureCenterStageControlMode::App);
            AVCaptureDevice::setCenterStageEnabled(false);
        }
    }

    let media_type = unsafe { AVMediaTypeVideo }.expect("AVMediaTypeVideo constant");
    let Some(device) = (unsafe { AVCaptureDevice::defaultDeviceWithMediaType(media_type) }) else {
        panic!("no camera found");
    };
    let Ok(input) = (unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }) else {
        panic!("failed to open camera");
    };
    unsafe { session.addInput(&input) };

    let preview = unsafe { AVCaptureVideoPreviewLayer::layerWithSession(&session) };

    let gravity = unsafe { AVLayerVideoGravityResizeAspectFill }.expect("video gravity constant");
    unsafe {
        preview.setVideoGravity(&gravity);
        preview.setFrame(bubble_rect(BUBBLE_SIZE));
        preview.setCornerRadius(BUBBLE_SIZE / 2.0);
        preview.setMasksToBounds(true);

        if let Some(connection) = preview.connection() {
            connection.setAutomaticallyAdjustsVideoMirroring(false);
            connection.setVideoMirrored(true);
        }

        session.startRunning();
    }

    // Feedback ring layered just above the preview; invisible until hover/resize.
    let ring: Retained<CALayer> =
        unsafe { msg_send![CALayer::class(), layer] };
    ring.setFrame(bubble_rect(BUBBLE_SIZE));
    ring.setCornerRadius(BUBBLE_SIZE / 2.0);
    ring.setBorderWidth(RING_BORDER_WIDTH);
    ring.setMasksToBounds(false);
    ring.setOpacity(0.0);

    if let Some(border_color) = system_ring_color() {
        ring.setBorderColor(Some(&border_color));
    }

    let bubble = BubbleView::alloc(mtm).set_ivars(BubbleIvars {
        session: session.clone(),
        preview: preview.clone(),
        ring: ring.clone(),
        drag: RefCell::new(None),
        pointer_grip: Cell::new(Grip::None),
    });
    let bubble: Retained<BubbleView> =
        unsafe { msg_send![super(bubble), initWithFrame: bubble_rect(BUBBLE_SIZE)] };

    bubble.setWantsLayer(true);
    if let Some(layer) = bubble.layer() {
        layer.addSublayer(&preview);
        layer.addSublayer(&ring);
    }

    install_tracking_area(&bubble);
    attach_quit_menu(mtm, &bubble);

    bubble
}

fn system_ring_color() -> Option<Retained<CGColor>> {
    // Warm white reads well over any desktop content.
    Some(NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.9).CGColor())
}

fn install_tracking_area(view: &BubbleView) {
    // Call initWithRect:options:owner:userInfo: via msg_send! so we can pass
    // the view directly as the owner (an `id` in ObjC) without needing an
    // explicit `&AnyObject` conversion.
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
    let opts = NSTrackingAreaOptions::MouseEnteredAndExited
        | NSTrackingAreaOptions::MouseMoved
        | NSTrackingAreaOptions::ActiveAlways
        | NSTrackingAreaOptions::InVisibleRect;
    let area: Retained<NSTrackingArea> = unsafe {
        msg_send![
            NSTrackingArea::alloc(),
            initWithRect: rect,
            options: opts,
            owner: view,
            userInfo: None::<&AnyObject>,
        ]
    };
    view.addTrackingArea(&area);
}

fn attach_quit_menu(mtm: MainThreadMarker, view: &NSView) {
    let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));

    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            ns_string!("Quit facecam"),
            Some(sel!(terminate:)),
            ns_string!("q"),
        )
    };
    quit_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    menu.addItem(&quit_item);

    unsafe { view.setMenu(Some(&menu)) };
}
