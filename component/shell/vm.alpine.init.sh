# vim: et ts=2 sw=2

repos='/etc/apk/repositories'
versi='v3.8'
mirro='mirror.leaseweb.com/alpine'

set -euo pipefail

echo "" | sudo tee "$repos"

echo "https://$mirro/$versi/main" | sudo tee -a "$repos"
echo "https://$mirro/$versi/community" | sudo tee -a "$repos"

sudo apk update
sudo apk upgrade
sudo apk add \
  git \
#   bash \
#   build-base \
#   ccache \
#   curl \
#   file \
#   gzip \
#   libc-dev \
#   libc6-compat \
#   libressl-dev \
#   linux-headers \
#   ncurses \
#   sudo \
#   zlib-dev \
#
#if ! id -u bob 2>/dev/null
#then
#  sudo adduser -D -u 2000 bob
#  yes h4ckb0b | sudo passwd bob
#  echo 'bob ALL=(ALL) NOPASSWD:ALL' | sudo tee -a /etc/sudoers
#  sudo -u bob sudo id
#fi

cd /
sudo sh -c '
  git init .
  echo .bash_history      >> .gitignore
  echo fish_history       >> .gitignore
  echo /var/log/messages  >> .gitignore
  ls -1 |
    grep -v -E "tmp|proc|sys" |
    xargs -I:: git add .gitignore ::
  git config --local user.name Root
  git config --local user.email root@root.root
  git commit --message root
'
