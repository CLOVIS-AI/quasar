FROM archlinux:latest
WORKDIR /opt/quasar

RUN pacman -Syuu --noconfirm --needed rust cargo cmake git shaderc vulkan-icd-loader
