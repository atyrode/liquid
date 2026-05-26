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
