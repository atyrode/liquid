if [ -f /etc/profile.d/liquid-dev-tools.sh ]; then
  . /etc/profile.d/liquid-dev-tools.sh
fi

if [ -f /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh ]; then
  . /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
fi

if [ -f /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]; then
  . /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
fi

autoload -Uz compinit
compinit

if command -v fastfetch >/dev/null 2>&1; then
  fastfetch
elif command -v neofetch >/dev/null 2>&1; then
  neofetch
fi

if [ -z "${LIQUID_NO_AUTO_TMUX:-}" ] &&
  [ -z "${SSH_CONNECTION:-}" ] &&
  [ -z "${TMUX:-}" ] &&
  [ "$(tty 2>/dev/null)" = "/dev/tty1" ] &&
  command -v liquid-start >/dev/null 2>&1; then
  liquid-start --attach
fi
