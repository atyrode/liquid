export ZSH=/opt/oh-my-zsh
ZSH_THEME="robbyrussell"
plugins=(git tmux)

if [ -f "$ZSH/oh-my-zsh.sh" ]; then
  . "$ZSH/oh-my-zsh.sh"
fi

if [ -f /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh ]; then
  . /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
fi

if [ -f /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]; then
  . /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
fi

autoload -Uz compinit
compinit

if [ -f /etc/profile.d/liquid-dev-tools.sh ]; then
  . /etc/profile.d/liquid-dev-tools.sh
fi

if [ -f "$HOME/.liquid-shell.zsh" ]; then
  . "$HOME/.liquid-shell.zsh"
fi
