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
  alias ls='tree -L 1 --noreport --charset "${TREE_CHARSET:-utf-8}"'
fi

alias cl='clear'
alias python='python3'
alias py='python3'
alias pip='pip3'

atmux() {
  tmux attach-session -t "$1"
}

hub() {
  repo="$1"
  git clone "https://github.com/atyrode/$repo.git" || return 1
  cd "$repo" || return 1
  [ -d venv ] && command -v venv >/dev/null 2>&1 && venv
  [ -f requirements.txt ] && command -v pipreq >/dev/null 2>&1 && pipreq
}

if [ -n "${BASH_VERSION:-}" ] && command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init bash)"
fi

if [ -n "${ZSH_VERSION:-}" ] && command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init zsh)"
fi

if [ -n "${ZSH_VERSION:-}" ] && command -v fzf >/dev/null 2>&1; then
  eval "$(fzf --zsh)"
fi
