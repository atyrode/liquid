############################################
# Git helpers
############################################

hub() {
  local repo="${1:-}"

  if [[ -z "$repo" ]]; then
    c_ko "Usage: hub <repo>"
    return 2
  fi

  git clone "https://github.com/atyrode/$repo.git" || return 1
  cd "$repo" || return 1
  [[ -d venv ]] && venv
  [[ -f requirements.txt ]] && pipreq
}
