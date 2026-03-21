FROM storytellerf/rust-in-docker:latest-dev

ARG USER_NAME

USER root

# 如果需要中文输入法
RUN apt update && DEBIAN_FRONTEND=noninteractive apt install -y fcitx fcitx-googlepinyin

USER $USER_NAME