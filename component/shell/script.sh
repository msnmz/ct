# vim: et ts=2 sw=2

set -euo pipefail

# sudo apk --update add \
#   bash \
#   build-base \
#   ccache \
#   curl \
#   file \
#   git \
#   gzip \
#   libc6-compat \
#   linux-headers \
#   ncurses \
#   ruby \
#   ruby-dbm \
#   ruby-etc \
#   ruby-irb \
#   ruby-json \
#   ruby-dev \
#   sudo \
#   zlib \
#   zlib-dev \
#   build-base ruby-dev libc-dev linux-headers \
#   libressl-dev postgresql-dev libxml2-dev libxslt-dev 1>&2
#
echo 'export PATH="/usr/lib/ccache/bin:$PATH"' >> ~/.profile
source $HOME/.profile

if ! test -d ~/.rbenv
then
  git clone https://github.com/rbenv/rbenv.git ~/.rbenv
else
  git -C ~/.rbenv pull
fi
cd ~/.rbenv && src/configure && make -C src

echo 'export PATH="$HOME/.rbenv/bin:$PATH"' >> ~/.profile
echo 'eval "$( rbenv init - )"' >> ~/.profile
source $HOME/.profile

mkdir -p "$(rbenv root)"/plugins
if ! test -d "$(rbenv root)"/plugins/ruby-build
then
  git clone https://github.com/rbenv/ruby-build.git "$(rbenv root)"/plugins/ruby-build
else
  git -C "$(rbenv root)"/plugins/ruby-build pull
fi

if test "$(which ruby)" = /usr/bin/ruby
then
  rbenv install 2.6.3
elif ruby --version | grep 2.6.3
then
  echo OK RUBY
else
  echo What is happening?
  echo '(+)' rbenv global 2.6.3
  rbenv global 2.6.3
  rbenv global
fi

if ! id -u linuxbrew
then
  sudo adduser -D -s /bin/bash linuxbrew
fi
echo 'linuxbrew ALL=(ALL) NOPASSWD:ALL' | sudo tee -a /etc/sudoers
echo 'export PATH=$HOME/.linuxbrew/bin:$HOME/.linuxbrew/sbin:$PATH' >> ~/.profile
source ~/.profile
if ! test -d $HOME/.linuxbrew
then
  git clone https://github.com/Homebrew/brew $HOME/.linuxbrew
else
  git -C $HOME/.linuxbrew pull
fi

brew update
brew doctor
