if true
then
  repos='/etc/apk/repositories'
  versi='v3.8'
  mirro='mirror.leaseweb.com/alpine'

  set -euo pipefail

  echo "" | sudo tee "$repos"

  echo "https://$mirro/$versi/main" | sudo tee -a "$repos"
  echo "https://$mirro/$versi/community" | sudo tee -a "$repos"

  sudo apk update
  sudo apk upgrade
  sudo apk add git gcc libc-dev linux-headers build-base python3 libc6-compat \

  if test -d ct
  then
    git -C ct pull -s recursive -X theirs
  else
    git clone https://github.com/Good-Vibez/ct.git ct
  fi

  if test -d rust
  then
    git -C rust pull -s recursive -X theirs
  elif false
    git clone https://github.com/rust-lang/rust.git rust
  else
    true
  fi

  if ! true
  then
    cd rust
    cp config.toml.example config.toml
    python3 ./x.py build && python3 ./x.py install
  else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs >rustup.sh
    bash rustup.sh -y --profile minimal
    echo 'source $HOME/.cargo/env' >> $HOME/.profile
  fi
fi

# source $HOME/.profile
# cd ct/component/cargo
# find . -name Cargo.lock -execdir rm -rfv '{}' \;
# cargo test --workspace --all-targets && cargo test --workspace || exit 1
# cargo build --release --workspace --all-targets
# cd ../../
# for b in xs xc xr; do sudo cp -av .cache/cargo/release/$b /usr/local/bin; done
