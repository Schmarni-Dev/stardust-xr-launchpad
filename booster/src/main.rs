use std::env::args;

fn main() -> color_eyre::Result<()> {
    let mut cmd = std::process::Command::new("/bin/bash");
    cmd.arg("-c");

    cmd.arg(args().skip(1).collect::<Vec<_>>().join(" "));
    let mut _seat = libseat::Seat::open(|seat_ref, event| match event {
        libseat::SeatEvent::Enable => {}
        libseat::SeatEvent::Disable => seat_ref.disable().unwrap(),
    })?;
    cmd.spawn().unwrap().wait()?;
    Ok(())
}
