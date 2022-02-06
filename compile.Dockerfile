FROM archlinux:latest
WORKDIR /opt/quasar

RUN pacman -Syuu --noconfirm --needed rust cargo
RUN pacman -Syuu --noconfirm --needed vulkan-icd-loader
