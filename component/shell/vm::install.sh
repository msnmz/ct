# vim: et ts=2 sw=2 ft=bash

main() {
  ui::doing "BASE"
  CDI::install:base_devel
  ui::doing "RBENV"
  CDI::install:rbenv
  CDI::install:user_paths.ccache
  ui::doing "HMBRW"
  CDI::user_init:load
  CDI::install:homebrew
  ui::doing "HMBRW_PKGS"
  brew:install "${BREW_PACKAGES[@]}"
  config::fish
  config::bash
  config::tmux
  ui::doing "OMF"
  CDI::install:omf
  ui::doing "CARGO"
  CDI::install:cargo
  CDI::user_init:load
  ui::doing "XFCE4"
  UDI::install:xfce4
  ui::doing "CT"
  CDI::install:ct
echo "*] Just chillin'"
}

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
config::fish() {
  mkdir -pv $HOME/.config/fish/conf.d
  tee $HOME/.config/fish/conf.d/osx_gnu.fish >/dev/null <<-'EOS'
  if test (uname -s) = "Darwin"
    set -gx PATH /usr/local/opt/coreutils/libexec/gnubin $PATH
    set -gx PATH /usr/local/opt/gnu-sed/libexec/gnubin $PATH
  end
EOS
  tee $HOME/.config/fish/conf.d/vi.fish >/dev/null <<-'EOS'
  set -U fish_key_bindings fish_vi_key_bindings
EOS
}
config::bash() {
  tee -a $HOME/.bashrc >/dev/null <<-'EOS'
  source $HOME/.user_paths
  source $HOME/.user_init
EOS
}
config::tmux() {
  tee $HOME/.tmux.conf >/dev/null <<-'EOS'
  set -g escape-time 0
  set -g mode-keys vi
  set -g status-style bg=colour24
  set -g status-left-style bg=colour162
  set -g status-right-style bg=colour17,fg=colour92
  set -g default-shell /home/linuxbrew/.linuxbrew/bin/fish
  set -g default-terminal screen-256color
EOS
}

pacman:install() {
  sudo pacman -Syu --noconfirm --needed --quiet "$@"
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

CDI::linux:distro() {
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

CDI::install:base_devel() {
  case "$( CDI::linux:distro )" in
    (Ubuntu)
      config::apt:sources
      apt:install \
	"${COMMON_PACKAGES[@]}" \
	"${DEBIAN_APT_NEEDS[@]}" \
	"${RUBY_DEPS_DEB[@]}" \
      ;;
    (Arch)
      config::pacman:mirrorlist
      pacman:install \
	"${COMMON_PACKAGES[@]}" \
	"${PACMAN_AUR_NEEDS}" \
      ;;
  esac
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

CDI::install:rbenv-build() {
  target="$(rbenv root)"/plugins

  mkdir -p "$target"
  gh:init "rbenv/ruby-build" "$target/ruby-build"
}
CDI::install:rbenv() {
  if $HOME/.rbenv/bin/rbenv version >/dev/null 2>/dev/null
  then
    true
  else
    gh:init "rbenv/rbenv" "$HOME/.rbenv"
    cd ~/.rbenv && src/configure && make -C src

    CDI::user_paths:add "$HOME/.rbenv/bin"
    CDI::user_init:add.eval '$(rbenv init -)'
    CDI::user_init:load

    CDI::install:rbenv-build
    curl -fsSL "https://github.com/rbenv/rbenv-installer/raw/master/bin/rbenv-doctor" | bash
  fi
}
CDI::install:ruby.3.0.1() {
  ui::doing "RB_3.0.1"
  CDI::user_init:load
  if rbenv versions --bare --skip-aliases | grep 3.0.1
  then
    true
  else
    rbenv install 3.0.1
  fi
}
CDI::install:homebrew() {
  if which brew >/dev/null 2>/dev/null
  then
    true
  else
    curl -fsSL "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh" | bash
    CDI::user_init:add.eval '$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)'
  fi
}

CDI::install:user_paths.ccache() {
  case "$( CDI::linux:distro )" in
    Ubuntu)
      CDI::user_paths:add "/lib/ccache"
      ;;
    Arch)
      CDI::user_paths:add "/lib/ccache/bin"
      ;;
  esac
}

CDI::install:omf() {
  if fish -c 'omf >/dev/null' 2>/dev/null
  then
    true
  else
    CDI::user_init:load
    curl -sL https://get.oh-my.fish >omf::install.fish
    fish omf::install.fish --noninteractive
    fish -c 'omf install flash'
  fi
}

CDI::install:cargo() {
  if $HOME/.cargo/bin/cargo --version >/dev/null 2>/dev/null
  then
    true
  else
    ui::doing "RUST"
    brew: install rustup-init
    rustup-init -y
    CDI::user_init:add.eval 'source $HOME/.cargo/env'
  fi
}

CDI::install:ct() {
  brew: tap Good-Vibez/tap
  brew:install2 --HEAD \
    Good-Vibez/tap/xs \
    Good-Vibez/tap/xc \
  ;
}

UDI::install:xfce4() {
  case "$( CDI::linux:distro )" in
    Ubuntu)
      apt:install.delayed xfce4
      ;;
    Arch)
      pacman:install xfce4 xorg-xinit
      ;;
  esac
}

brew:() {
  CDI::user_init:load
  brew "$@"
}
rbenv:() {
  CDI::user_init:load
  rbenv "$@"
}

brew:install() {
  brew:install2 "" "${@}"
}
brew:install2() {
  brargs="$1"; shift;

  if jq --version >/dev/null 2>/dev/null
  then
    brew: info --json --formulae "${@}" \
    | jq \
      --raw-output \
      --join-output \
      --compact-output '.
	| map(select((.installed | length) == 0))
	| map(.name)
	| join("\u0000")
      ' \
    | xargs -0 -I::: brew install $brargs ::: # NOTE: DO NOT QUOTE $brargs
  else
    brew install "${@}"
  fi
}

if test "${VMINSTALLLIB-x}" = "x"
then
  ui::doing "MAIN"
  main
fi
