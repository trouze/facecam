use objc2::{msg_send, rc::Retained, sel, ClassType, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSMenu, NSMenuItem, NSEventModifierFlags, NSView};
use objc2_av_foundation::{
    AVLayerVideoGravityResizeAspectFill, AVCaptureCenterStageControlMode, AVCaptureDevice,
    AVCaptureDeviceInput, AVCaptureSession, AVCaptureSessionPresetHigh, AVCaptureVideoPreviewLayer,
    AVMediaTypeVideo,
};
use objc2_foundation::{ns_string, NSPoint, NSRect, NSSize};

const BUBBLE_SIZE: f64 = 200.0;
const RADIUS: f64 = BUBBLE_SIZE / 2.0;

fn bubble_rect() -> NSRect {
    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(BUBBLE_SIZE, BUBBLE_SIZE))
}

/// Creates the layer-backed view that hosts the circular webcam preview.
///
/// Mirrors CamBubble's `CameraView`: a 200x200 container whose sublayer is an
/// aspect-fill, mirrored `AVCaptureVideoPreviewLayer` clipped to a circle,
/// with a right-click "Quit FaceCam" menu.
pub fn camera_bubble_view(mtm: MainThreadMarker) -> Retained<NSView> {
    let container = NSView::initWithFrame(NSView::alloc(mtm), bubble_rect());
    container.setWantsLayer(true);

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
        return container;
    };
    let Ok(input) = (unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }) else {
        return container;
    };
    unsafe { session.addInput(&input) };

    let preview = unsafe { AVCaptureVideoPreviewLayer::layerWithSession(&session) };

    let gravity = unsafe { AVLayerVideoGravityResizeAspectFill }.expect("video gravity constant");
    unsafe {
        preview.setVideoGravity(&gravity);
        preview.setFrame(bubble_rect());
        preview.setCornerRadius(RADIUS);
        preview.setMasksToBounds(true);

        if let Some(connection) = preview.connection() {
            connection.setAutomaticallyAdjustsVideoMirroring(false);
            connection.setVideoMirrored(true);
        }

        session.startRunning();
    }

    if let Some(layer) = container.layer() {
        layer.addSublayer(&preview);
    }

    attach_quit_menu(mtm, &container);

    container
}

fn attach_quit_menu(mtm: MainThreadMarker, view: &NSView) {
    let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));

    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            ns_string!("Quit FaceCam"),
            Some(sel!(terminate:)),
            ns_string!("q"),
        )
    };
    quit_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
    menu.addItem(&quit_item);

    unsafe { view.setMenu(Some(&menu)) };
}
