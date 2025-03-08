FROM mcr.microsoft.com/devcontainers/rust:latest

ARG node_version=v22.14.0
ARG node_root_dir=/usr/local/nodejs
RUN mkdir ${node_root_dir} && cd ${node_root_dir} \
&& curl -LO https://nodejs.org/dist/${node_version}/node-${node_version}-linux-x64.tar.xz \
&& tar xJf node-${node_version}-linux-x64.tar.xz --strip-components 1 \
&& rm node-${node_version}-linux-x64.tar.xz
ENV PATH="${node_root_dir}/bin:${PATH}"
RUN npm config set registry http://mirrors.cloud.tencent.com/npm/
RUN npm install -g pnpm

RUN curl -fsSL https://code-server.dev/install.sh | sh
RUN code-server --install-extension rust-lang.rust-analyzer
RUN code-server --install-extension tamasfe.even-better-toml
RUN code-server --install-extension fill-labs.dependi
RUN code-server --install-extension Vue.volar
RUN code-server --install-extension njzy.stats-bar
RUN code-server --install-extension Tencent-Cloud.coding-copilot
RUN code-server --install-extension cnbcool.cnb-welcome

RUN apt-get update && apt-get install -y webkit2gtk-4.1 libwebkit2gtk-4.0-dev build-essential file libxdo-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
RUN apt-get update && apt-get install -y curl wget unzip openssh-server git 

RUN code-server --install-extension donjayamanne.githistory
RUN code-server --install-extension tauri-apps.tauri-vscode

ENV LANG C.UTF-8
ENV LANGUAGE C.UTF-8