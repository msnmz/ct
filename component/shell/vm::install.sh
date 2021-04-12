# vim: et ts=2 sw=2 ft=bash

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
  gcc
  fish
  tmux
  htop
  make
  ccache
  zlib
  libressl
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

config::apt:sources() {
  echo "* Installing ubuntu apt sources"
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
  echo "* Installing arch pacman mirrorlist"
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

# ENABLE_INIT=1
# ENABLE_RBENV=1
# ENABLE_HOMEBREW=1
ENABLE_HOMEBREW_PACKAGES=1

  echo INIT "${ENABLE_INIT+x}"
  echo RBENV "${ENABLE_RBENV+x}"
  echo HMBRE "${ENABLE_HOMEBREW+x}"
  echo HMBPK "${ENABLE_HOMEBRW_PACKAGES+x}"

if test \
  "${ENABLE_INIT+x}" = "x" -o \
  0 = 1
then
  echo INIT
  mode::init
fi

if test \
  "${ENABLE_RBENV+x}" = "x" -o \
  0 = 1
then
  echo RBENV
  mode::rbenv:install-3.0.1
fi

if test \
  "${ENABLE_HOMEBREW+x}" = "x" -o \
  0 = 1
then
  echo HMBRW
  mode::homebrew:install
fi

if test \
  "${ENABLE_HOMEBREW_PACKAGES+x}" = "x" -o \
  0 = 1
then
  echo HMBRW PKGS
  brew: install "${BREW_PACKAGES[@]}"
fi

echo "*] Just chillin'"
