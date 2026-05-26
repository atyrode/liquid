GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
CYAN="\033[0;36m"
NC="\033[0m"

c_ok() { echo -e "${GREEN}$1${NC}"; }
c_ko() { echo -e "${RED}$1${NC}"; }
c_folder() { echo -e "${CYAN}$1${NC}"; }
c_file() { echo -e "${YELLOW}$1${NC}"; }

prompt_yes_no() {
  local prompt_message="$1"
  local answer
  while true; do
    echo -n "$prompt_message (y/n): "
    read -r answer
    case "$answer" in
      [Yy]*) return 0 ;;
      [Nn]*) return 1 ;;
      *) echo "Please answer y or n." ;;
    esac
  done
}

VENV_DIR=".venv"
GITIGNORE=".gitignore"

_venv_exists() { [[ -d "$VENV_DIR" ]]; }
_venv_is_active() { [[ -n "$VIRTUAL_ENV" ]]; }

_create_venv() {
  if command -v uv >/dev/null 2>&1; then
    c_ok "Using uv to create venv"
    uv venv
  else
    python3 -m venv "$VENV_DIR"
  fi
}

_activate_venv() {
  source "$VENV_DIR/bin/activate"
  c_ok "Activated venv"
}

_deactivate_venv() {
  deactivate
  c_ko "Deactivated venv"
}

_setup_gitignore() {
  [[ -f "$GITIGNORE" ]] || touch "$GITIGNORE"
  grep -q "^$VENV_DIR$" "$GITIGNORE" || echo "$VENV_DIR" >> "$GITIGNORE"
}

venv() {
  local parent_dir
  parent_dir="$(basename "$(pwd)")"
  if _venv_is_active; then
    _deactivate_venv
  elif _venv_exists; then
    _activate_venv
  elif prompt_yes_no "Create venv in $parent_dir?"; then
    _create_venv
    _setup_gitignore
    _activate_venv
  fi
}

pipreq() {
  _venv_is_active || venv || return 1
  if command -v uv >/dev/null 2>&1; then
    uv pip install -r requirements.txt
  else
    pip install -r requirements.txt
  fi
}
alias pipr="pipreq"

pipfreeze() {
  _venv_is_active || return 1
  if command -v uv >/dev/null 2>&1; then
    uv pip freeze > requirements.txt
  else
    pip freeze > requirements.txt
  fi
}
alias pipf="pipfreeze"
