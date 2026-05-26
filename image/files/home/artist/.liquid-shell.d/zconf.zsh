############################################
# Liquid shell reload and help
############################################

zconf() {
  if [[ -n "$VIRTUAL_ENV" ]]; then
    deactivate
    echo -e "$(c_ok "Deactivated") virtual environment."
  fi

  unalias -a 2>/dev/null || true

  echo -e "$(c_folder "Reloading") Liquid shell configuration."
  exec zsh -l
}

atyrode() {
  echo ""
  echo -e "$(c_folder "==== Liquid Shell Help ====")"
  echo ""

  echo -e "$(c_ok "Runtime controls:")"
  echo -e "  $(c_file "*") ~/liquid-control/start"
  echo -e "  $(c_file "*") ~/liquid-control/attach"
  echo -e "  $(c_file "*") ~/liquid-control/stop"
  echo -e "  $(c_file "*") ~/liquid-control/config"
  echo -e "  $(c_file "*") ~/liquid-control/bluetooth"
  echo -e "  $(c_file "*") ~/liquid-control/doctor"
  echo ""

  echo -e "$(c_ok "Shell functions:")"
  echo -e "  $(c_file "*") zconf    - reload the login shell"
  echo -e "  $(c_file "*") atyrode  - show this help"
  echo -e "  $(c_file "*") hub      - clone github.com/atyrode/<repo>"
  echo -e "  $(c_file "*") atmux    - attach to a tmux session"
  echo -e "  $(c_file "*") venv     - create/activate/deactivate .venv"
  echo -e "  $(c_file "*") revenv   - recreate .venv and reinstall requirements"
  echo -e "  $(c_file "*") unvenv   - remove .venv"
  echo ""

  echo -e "$(c_ok "Shell aliases:")"
  echo -e "  $(c_file "*") cl, htop, ls, python, py, pip, pymake"
  echo -e "  $(c_file "*") pipr, pipf, pipd"
  echo ""
}
