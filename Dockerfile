FROM rustembedded/cross:arm-unknown-linux-gnueabihf-0.2.1

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install --assume-yes libusb-1.0-0-dev:armhf

# described e.g. here: https://capnfabs.net/posts/cross-compiling-rust-apps-raspberry-pi/
ENV PKG_CONFIG_LIBDIR_arm_unknown_linux_gnueabihf=/usr/lib/arm-linux-gnueabihf/pkgconfig

# https://stackoverflow.com/questions/42796310/cross-compile-using-go-build-cgo-enabled-warning-libudev-so-1-not-found
ENV RUSTFLAGS="-C link-arg=-Wl,-rpath-link,/lib/arm-linux-gnueabihf"