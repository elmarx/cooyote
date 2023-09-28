host := "mooncake.local.athmer.org"

build:
    cross build --target=arm-unknown-linux-gnueabihf --release

setup:
    ansible-playbook -u root -i {{host}}, .ci/setup.yml

deploy: build
    ansible-playbook -u root -i {{host}}, .ci/deploy.yml

scp: build
    scp ./target/arm-unknown-linux-gnueabihf/release/cooyote root@{{ host }}:/usr/local/bin/
