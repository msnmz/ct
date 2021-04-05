# vim: et ts=2 sw=2
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  > rustup-init
chmod 755 rustup-init
./rustup-init -y
