use super::{BootScreen, BootState};
use crate::{color, renderer::Framebuffer};

fn screen(pixels: Vec<u8>, width: u32, height: u32, duration: f32) -> BootScreen {
    BootScreen {
        pixels,
        width,
        height,
        duration,
    }
}

#[test]
fn boot_waits_for_duration_and_does_not_skip_screens_on_a_long_frame() {
    let mut boot = BootState::new(vec![
        screen(vec![color::RED], 1, 1, 1.0),
        screen(vec![color::BLUE], 1, 1, 1.0),
    ]);
    boot.update(0.5);
    assert_eq!(boot.current_screen().unwrap().pixels, [color::RED]);
    boot.update(10.0);
    assert_eq!(boot.current_screen().unwrap().pixels, [color::BLUE]);
    assert_eq!(boot.elapsed, 0.0);
    boot.update(0.5);
    assert!(!boot.is_finished());
    boot.update(0.5);
    assert!(boot.is_finished());
    assert!(boot.current_screen().is_none());
    boot.update(1.0);
    assert!(boot.is_finished());
}

#[test]
fn finished_boot_preserves_the_last_frame() {
    let mut boot = BootState::new(vec![screen(vec![color::RED], 1, 1, 1.0)]);
    let mut framebuffer = Framebuffer::new(2, 2);
    boot.render(&mut framebuffer);
    assert_eq!(
        framebuffer.pixels(),
        [color::RED, color::BLACK, color::BLACK, color::BLACK]
    );
    boot.update(1.0);
    boot.render(&mut framebuffer);
    assert_eq!(
        framebuffer.pixels(),
        [color::RED, color::BLACK, color::BLACK, color::BLACK]
    );
    let mut empty = BootState::new(vec![]);
    empty.update(1.0);
    assert!(empty.is_finished());
    empty.render(&mut framebuffer);
    assert_eq!(framebuffer.pixels()[0], color::RED);
}

#[test]
fn boot_crops_using_the_source_image_stride() {
    let boot = BootState::new(vec![screen(
        vec![
            color::RED,
            color::BLUE,
            color::BLACK,
            color::WHITE,
            color::GREEN,
            color::BLACK,
        ],
        3,
        2,
        1.0,
    )]);
    let mut framebuffer = Framebuffer::new(2, 2);
    boot.render(&mut framebuffer);
    assert_eq!(
        framebuffer.pixels(),
        [color::RED, color::BLUE, color::WHITE, color::GREEN]
    );
}

#[test]
fn custom_boot_screen_follows_both_engine_screens() {
    let mut boot = BootState::with_engine_screens(Some(screen(vec![color::RED], 1, 1, 2.0)));
    assert_eq!(boot.screens.len(), 3);
    for bytes in [
        crate::globals::BOOT_RUST_PICOCAD,
        crate::globals::BOOT_IRON_OXIDE,
    ] {
        let expected = BootScreen::from_bytes(bytes, crate::globals::DEFAULT_BOOT_SCREEN_DURATION);
        let current = boot.current_screen().unwrap();
        assert_eq!(
            (current.width, current.height),
            (expected.width, expected.height)
        );
        assert_eq!(current.pixels, expected.pixels);
        boot.update(crate::globals::DEFAULT_BOOT_SCREEN_DURATION);
    }
    assert_eq!(boot.current_screen().unwrap().pixels, [color::RED]);
    boot.update(2.0);
    assert!(boot.is_finished());
    assert_eq!(BootState::with_engine_screens(None).screens.len(), 2);
}
