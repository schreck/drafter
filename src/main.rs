use beryllium::{
    Event, InitFlags, SDL, SdlGlAttr, WindowFlags, WindowPosition,
};

fn main() {
    let sdl = SDL::init(InitFlags::Everything).expect("SDL init failed");

    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, 1).unwrap(); // 1 = Core profile

    let _win = sdl
        .create_gl_window("Drafter", WindowPosition::Centered, 800, 600, WindowFlags::Shown)
        .expect("window creation failed");

    'main_loop: loop {
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            if matches!(event, Event::Quit(_)) {
                break 'main_loop;
            }
        }
    }
}
