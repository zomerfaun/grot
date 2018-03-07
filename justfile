export RUST_BACKTRACE="1"
export RUST_LOG="grot=debug"

@run:
    cargo run

@trace:
    RUST_LOG="grot=trace" cargo run

@trance:
    mpv http://tunein.com/radio/radiOzora-Psy-s201024

@chill:
    mpv https://tunein.com/radio/radiOzora-Chill-channel-s201021
