############################################
# Startup footer
############################################

if [[ -z "${LIQUID_NO_FASTFETCH:-}" ]]; then
  if command -v fastfetch >/dev/null 2>&1; then
    fastfetch
  elif command -v neofetch >/dev/null 2>&1; then
    neofetch
  fi
fi
