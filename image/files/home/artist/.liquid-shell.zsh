# Liquid shell module loader.
#
# These modules mirror the portable parts of github.com/atyrode/nix-dotfiles.
# Nix/Home Manager rebuild logic is intentionally replaced by a local zconf
# shell reload helper because this image is not Nix-managed.

if [[ -n "${LIQUID_SHELL_DIR:-}" ]]; then
  _liquid_shell_dir="$LIQUID_SHELL_DIR"
elif [[ -d "$HOME/liquid/image/files/home/artist/.liquid-shell.d" ]]; then
  _liquid_shell_dir="$HOME/liquid/image/files/home/artist/.liquid-shell.d"
else
  _liquid_shell_dir="$HOME/.liquid-shell.d"
fi

for _liquid_shell_module in \
  colors \
  utils \
  aliases \
  python \
  git \
  zconf \
  tmux \
  startup
do
  _liquid_shell_path="$_liquid_shell_dir/$_liquid_shell_module.zsh"
  if [[ -f "$_liquid_shell_path" ]]; then
    source "$_liquid_shell_path"
  fi
done

unset _liquid_shell_dir _liquid_shell_module _liquid_shell_path
