############################################
# Shell behavior and aliases
############################################

alias cl="clear"
alias python="python3"
alias pip="pip3"
alias py="python3"
alias pymake="uv pip install -e ."

if command -v btop >/dev/null 2>&1; then
  alias htop="btop"
fi

if command -v tree >/dev/null 2>&1; then
  alias ls='tree -L 1 --noreport --charset "${TREE_CHARSET:-utf-8}"'
fi

if command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init zsh)"
fi

if command -v fzf >/dev/null 2>&1; then
  eval "$(fzf --zsh)"
fi
