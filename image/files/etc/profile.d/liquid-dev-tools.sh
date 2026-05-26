# Liquid development shell defaults.

if command -v batcat >/dev/null 2>&1 && ! command -v bat >/dev/null 2>&1; then
  alias bat='batcat'
fi

if command -v fdfind >/dev/null 2>&1 && ! command -v fd >/dev/null 2>&1; then
  alias fd='fdfind'
fi

if command -v btop >/dev/null 2>&1; then
  alias htop='btop'
fi

if command -v tree >/dev/null 2>&1; then
  alias ls='tree -L 1 --noreport'
fi

alias cl='clear'
alias python='python3'
alias py='python3'
alias pip='pip3'

if [ -n "${BASH_VERSION:-}" ] && command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init bash)"
fi

if [ -n "${ZSH_VERSION:-}" ] && command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init zsh)"
fi

if [ -n "${ZSH_VERSION:-}" ] && command -v fzf >/dev/null 2>&1; then
  eval "$(fzf --zsh)"
fi
