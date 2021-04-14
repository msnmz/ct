# vim: et ts=2 sw=2 ft=bash

ENABLE_INIT=1
ENABLE_RBENV=1
ENABLE_HOMEBREW=1
ENABLE_HOMEBREW_PACKAGES=1
ENABLE_RUST=1
ENABLE_BSHRC=1
ENABLE_XFCE=1
ENABLE_XS=1


COMMON_PACKAGES=(
  ccache
  gcc
  git
  htop
  make
)
DEBIAN_APT_NEEDS=(
  build-essential
  apt-utils
  dialog
)
PACMAN_AUR_NEEDS=(
  base-devel
  pacman-contrib
)
PACMAN_MIRROR_UTIL=rate-arch-mirrors  # AUR
UBUNTU_MIRROR_UTIL=apt-smart          # pip3
RUBY_DEPS_DEB=(
  zlib1g-dev
  libssl-dev
)
BREW_PACKAGES=(
  # System and gnu
  gcc
  make
  ccache
  zlib
  curl
  findutils
  grep
  gnu-sed
  # libressl

  # Tools
  jq
  direnv
  pv
  nvim
  git

  # Dev/Workspace/Aesthetics
  fish
  tmux
  htop
  lsd
)

git:init() {
  source="$1"; shift;
  target="$1"; shift;

  if test -d $target
  then
    git -C $target remote update --prune
  else
    git clone --depth 1 $source $target
  fi
}

gh:init() {
  name="$1"; shift;
  target="$1"; shift;

  source="https://github.com/$name.git"
  git:init "$source" "$target"
}

ui::doing() {
  printf '==> %s\n' "$1"
}

config::apt:sources() {
  ui::doing "Instal ubuntu apt sources"
  sudo tee /etc/apt/sources.list >/dev/null <<-'EOS'
    deb http://mirror.eu.kamatera.com/ubuntu focal main restricted
    deb http://mirror.eu.kamatera.com/ubuntu focal-updates main restricted
    deb http://mirror.eu.kamatera.com/ubuntu focal universe
    deb http://mirror.eu.kamatera.com/ubuntu focal-updates universe
    deb http://mirror.eu.kamatera.com/ubuntu focal multiverse
    deb http://mirror.eu.kamatera.com/ubuntu focal-updates multiverse
    deb http://mirror.eu.kamatera.com/ubuntu focal-backports main restricted universe multiverse
    deb http://security.ubuntu.com/ubuntu focal-security main restricted
    deb http://security.ubuntu.com/ubuntu focal-security universe
    deb http://security.ubuntu.com/ubuntu focal-security multiverse
EOS
}

config::pacman:mirrorlist() {
  ui::doing "Instal arch pacman mirrorlist"
  sudo tee /etc/pacman.d/mirrorlist >/dev/null <<-'EOS'
    Server = https://archlinux.koyanet.lv/archlinux/$repo/os/$arch
    Server = http://mirror.puzzle.ch/archlinux/$repo/os/$arch
    Server = http://mirror.datacenter.by/pub/archlinux/$repo/os/$arch
    Server = https://archlinux.uk.mirror.allworldit.com/archlinux/$repo/os/$arch
    Server = http://mirror.easylee.nl/archlinux/$repo/os/$arch
EOS
}

pacman:install() {
  sudo pacman -Syu --noconfirm --needed "$@"
}

apt:install() {
  sudo env DEBIAN_FRONTEND=noninteractive apt-get update -y
  sudo env DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
  sudo env DEBIAN_FRONTEND=noninteractive apt-get install "$@" -y
}
apt:install.delayed() {
  sudo env DEBIAN_FRONTEND=noninteractive apt-get install "$@" -y
}

aur:install() {
  local name="$1"; shift;

  curl https://aur.archlinux.org/cgit/aur.git/snapshot/$name.tar.gz >$name.tar.gz

  tar xvf $name.tar.gz
  cd $name

  makepkg -sic --noconfirm
}

linux.distro() {
  node_name="$(uname --nodename)"
  case "$node_name" in
    ubuntu-*)
      echo "Ubuntu"
      ;;
    arch.*)
      echo "Arch"
      ;;
    *)
      printf 'Unknown node_name: %s\n' "$node_name" 1>&2
      exit 1
  esac
}

CDI_install_base_devel() {
  case "$( linux.distro )" in
    (Ubuntu)
      apt:install \
	"${COMMON_PACKAGES[@]}" \
	"${DEBIAN_APT_NEEDS[@]}" \
	"${RUBY_DEPS_DEB[@]}" \
      ;;
    (Arch)
      pacman:install \
	"${COMMON_PACKAGES[@]}" \
	"${PACMAN_AUR_NEEDS}" \
      ;;
  esac
}

CDI_install_rbenv_build() {
  target="$(rbenv root)"/plugins

  mkdir -p "$target"
  gh:init "rbenv/ruby-build" "$target/ruby-build"
}

CDI::user_paths:add() {
  extra_path="$1"; shift;
  echo 'export PATH="'"$extra_path"':$PATH"' >> ~/.user_paths
}
CDI::user_init:add.eval() {
  hook="$1"; shift;
  echo 'eval "'"$hook"'"' >> ~/.user_init
}
CDI::user_init:load() {
  source $HOME/.user_paths
  source $HOME/.user_init
}

CDI_install_rbenv() {
  gh:init "rbenv/rbenv" "$HOME/.rbenv"
  cd ~/.rbenv && src/configure && make -C src

  CDI::user_paths:add "$HOME/.rbenv/bin"
  CDI::user_init:add.eval '$(rbenv init -)'

  source "$HOME/.user_paths"
  source "$HOME/.user_init"

  CDI_install_rbenv_build
  curl -fsSL "https://github.com/rbenv/rbenv-installer/raw/master/bin/rbenv-doctor" | bash
}

CDI_install_homebrew() {
  curl -fsSL "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh" | bash
}

CDI_install_ccache_to_user_paths() {
  case "$( linux.distro )" in
    Ubuntu)
      CDI::user_paths:add "/lib/ccache"
      ;;
    Arch)
      CDI::user_paths:add "/lib/ccache/bin"
      ;;
  esac
}

mode::init() {
  CDI_install_base_devel
  CDI_install_rbenv
  CDI_install_ccache_to_user_paths
}

mode::homebrew:install() {
  CDI::user_init:load

  curl -fsSL "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh" >homebrew:install.sh
  bash homebrew:install.sh

  CDI::user_init:add.eval '$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)'
}

mode::rbenv:install-3.0.1() {
  CDI::user_init:load
  rbenv install 3.0.1
}

brew:() {
  CDI::user_init:load
  brew "$@"
}

xfce4::install() {
  case "$( linux.distro )" in
    Ubuntu)
      apt:install.delayed xfce4
      ;;
    Arch)
      pacman:install xfce4 xorg-xinit
      ;;
  esac
}

fish::config() {
  tee ~/.config/fish/conf.d/osx_gnu.fish >/dev/null <<-'EOS'
  if test (uname -s) = "Darwin"
    set -gx PATH /usr/local/opt/coreutils/libexec/gnubin $PATH
    set -gx PATH /usr/local/opt/gnu-sed/libexec/gnubin $PATH
  end
EOS
  tee ~/.config/fish/conf.d/vi.fish >/dev/null <<-'EOS'
  set -U fish_key_bindings fish_vi_key_bindings
EOS
}

omf::install() {
  CDI::user_init:load
  curl -sL https://get.oh-my.fish >omf::install.fish
  fish omf::install.fish --noninteractive
  omf i flash
}

  echo ["${ENABLE_INIT+x}"] INIT
  echo ["${ENABLE_RBENV+x}"] RBENV
  echo ["${ENABLE_HOMEBREW+x}"] HMBRE
  echo ["${ENABLE_HOMEBRW_PACKAGES+x}"] HMBPK
  echo ["${ENABLE_RUST+x}"] RUST
  echo ["${ENABLE_BSHRC+x}"] BSHRC
  echo ["${ENABLE_XFCE+x}"] XFCE
  echo ["${ENABLE_XS+x}"] XS_XC
  echo ["${ENABLE_OMF+x}"] OMF

if test \
  "${ENABLE_INIT+x}" = "x" -o \
  0 = 1
then
  ui::doing "INIT"
  mode::init
fi

if test \
  "${ENABLE_RBENV+x}" = "x" -o \
  0 = 1
then
  ui::doing "RBENV"
  mode::rbenv:install-3.0.1
fi

if test \
  "${ENABLE_HOMEBREW+x}" = "x" -o \
  0 = 1
then
  ui::doing "HMBRW"
  mode::homebrew:install
fi

if test \
  "${ENABLE_HOMEBREW_PACKAGES+x}" = "x" -o \
  0 = 1
then
  ui::doing "HMBRW PKGS"
  brew: install "${BREW_PACKAGES[@]}"
  fish::config
fi

if test \
  "${ENABLE_RUST+x}" = "x" -o \
  0 = 1
then
  ui::doing "RUST"
  brew: install rustup-init
  rustup-init -y
  CDI::user_init:add.eval 'source $HOME/.cargo/env'
fi

if test \
  "${ENABLE_BSHRC+x}" = "x" -o \
  0 = 1
then
  ui::doing "BSHRC"
  echo 'source $HOME/.user_paths' >> $HOME/.bashrc
  echo 'source $HOME/.user_init' >> $HOME/.bashrc
fi

if test \
  "${ENABLE_XFCE+x}" = "x" -o \
  0 = 1
then
  ui::doing "XFCE (4)"
  xfce4::install
fi

if test \
  "${ENABLE_XS+x}" = "x" -o \
  0 = 1
then
  ui::doing "XS - XC"
  brew: install --HEAD \
    Good-Vibez/tap/xs \
    Good-Vibez/tap/xc \
  ;
fi

if test \
  "${ENABLE_OMF+x}" = "x" -o \
  0 = 1
then
  ui::doing "OMF"
  omf::install
fi

echo "*] Just chillin'"
