host := "mooncake.local.athmer.org"

ci:
    cargo check --all-targets
    cargo fmt --check
    # deny default clippy violations
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-targets -- -W clippy::pedantic
    cargo nextest run

build:
    cross build --target=arm-unknown-linux-gnueabihf --release

setup:
    ansible-playbook -u root -i {{host}}, .ci/setup.yml

deploy: build
    ansible-playbook -u root -i {{host}}, .ci/deploy.yml

scp: build
    scp ./target/arm-unknown-linux-gnueabihf/release/cooyote root@{{ host }}:/usr/local/bin/
