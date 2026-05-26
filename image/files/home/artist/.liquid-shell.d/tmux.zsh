############################################
# Tmux utilities
############################################

atmux() {
  local session="${1:-liquid}"
  tmux attach-session -t "$session"
}
