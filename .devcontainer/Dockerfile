FROM debian:trixie-slim
#rust:1.86.0-bookworm

USER root

ARG USER_ID=1000
ARG GROUP_ID=1000
ARG USER_NAME=rustdev

ENV TZ=Europe/Rome
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone


# Create user with empty password
RUN groupadd -g $GROUP_ID $USER_NAME && \
    useradd -g $GROUP_ID -m -s /bin/bash -p "" -u $USER_ID $USER_NAME


# copied from here: https://github.com/rust-lang/docker-rust/blob/master/stable/bookworm/slim/Dockerfile
LABEL org.opencontainers.image.source=https://github.com/rust-lang/docker-rust

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.86.0 \
    POETRY_VERSION=2.1.2 \
    POETRY_NO_INTERACTION=1 \
    POETRY_VIRTUALENVS_IN_PROJECT=0 \
    POETRY_VIRTUALENVS_CREATE=1 \
    POETRY_CACHE_DIR='/var/cache/pypoetry' \
    POETRY_HOME='/usr/local'



# python3-maturin no "new" command

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc gnupg2 \
        libc6-dev \
        openssh-client \
        wget curl \
        python3 pipx nano bash git libpython3-dev sudo \ 
        ; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='a3339fb004c3d0bb9862ba0bce001861fe5cbde9c10d16591eb3f39ee6cd3e7f' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='62cfed758140f94a75074fb350e287dd26d1b6c9a4d6a18616757fb344720bcb' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='c64b33db2c6b9385817ec0e49a84bcfe018ed6e328fe755c3c809580cc70ce7a' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='fec8226fede82b4b886855e4f1b69cdd17a6f60afdddb17f9a814b743c2d5c47' ;; \
        ppc64el) rustArch='powerpc64le-unknown-linux-gnu'; rustupSha256='b1ef09a734ece551456635b25c91a97770392b74c7f793fbc58575ddf0442645' ;; \
        s390x) rustArch='s390x-unknown-linux-gnu'; rustupSha256='8e4e8d5ffd3e6996303faf45670009388f73a4796264230f04f5c29809620c20' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.28.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    curl -sSL https://install.python-poetry.org | python3 - ; \ 
    mkdir -p $POETRY_CACHE_DIR ; \ 
    apt-get remove -y --auto-remove \
        wget curl \
        ; \
    rm -rf /var/lib/apt/lists/*; \
    # NO WAY to have /app owned by user
    # add sudo support  https://code.visualstudio.com/remote/advancedcontainers/add-nonroot-user
    echo $USER_NAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USER_NAME ; \
    chmod 0440 /etc/sudoers.d/$USER_NAME ; \
    rustup component add rustfmt ; \
    cargo install cocogitto cargo-bump ; \
    cargo install --no-default-features --force cargo-make;
    # add user to wheel group https://stackoverflow.com/questions/76248945/vs-code-devcontainer-workspace-ownership-and-permission
    # usermod -aG wheel $USER_NAME


    # && chown $USER_NAME:$USER_NAME $POETRY_CACHE_DIR ;\


# USER ${USER_NAME}

RUN pipx install maturin ; \
        pipx ensurepath ;


WORKDIR /app

VOLUME /app

ENTRYPOINT bash
