ussagi aws loginというコマンドで下記のスクリプトを実行できるようにしたいです。

#!/usr/bin/env bash
set -euo pipefail

# aws-profile-sso.sh
# Usage:
#   ./aws-profile-sso.sh            # interactive menu
#   ./aws-profile-sso.sh <profile>  # non-interactive (or use `source` to export into current shell)
#   source ./aws-profile-sso.sh     # interactive, but AWS_PROFILE exported into current shell

AWS_CLI_BIN="$(command -v aws || true)"
if [ -z "$AWS_CLI_BIN" ]; then
  echo "Error: aws CLI not found in PATH. Install/configure AWS CLI v2 first." >&2
  exit 2
fi

# Collect candidates from ~/.aws/config and ~/.aws/credentials
profiles=()

if [ -f "${HOME}/.aws/config" ]; then
  while IFS= read -r p; do
    # lines look like: [profile adet]
    p="$(printf '%s' "$p" | sed -E 's/^\[profile[[:space:]]+([^]]+)\].*/\1/')"
    profiles+=("$p")
  done < <(grep -E '^\[profile[[:space:]]+' "${HOME}/.aws/config" || true)
fi

if [ -f "${HOME}/.aws/credentials" ]; then
  while IFS= read -r p; do
    # lines look like: [tenpla] or [default]
    p="$(printf '%s' "$p" | sed -E 's/^\[([^]]+)\].*/\1/')"
    profiles+=("$p")
  done < <(grep -E '^\[' "${HOME}/.aws/credentials" || true)
fi

# dedupe while preserving order (no associative arrays for macOS bash)
unique_profiles=()
seen_list=""

for p in "${profiles[@]}"; do
  if [ -n "$p" ] && ! grep -q "|$p|" <<< "$seen_list"; then
    unique_profiles+=("$p")
    seen_list="${seen_list}|$p|"
  fi
done

# if nothing found, still allow manual input
if [ ${#unique_profiles[@]} -eq 0 ]; then
  echo "No profiles found in ~/.aws/config or ~/.aws/credentials."
  read -rp "Enter the AWS profile name you want to use: " manual_profile
  if [ -z "$manual_profile" ]; then
    echo "No profile provided; aborting." >&2
    exit 1
  fi
  CHOSEN="$manual_profile"
else
  # if user provided first arg, use it if it exists (or accept it anyway)
  if [ $# -ge 1 ] && [ -n "${1-}" ]; then
    CHOSEN="$1"
    # optionally warn if not present in discovered list
    found=0
    for p in "${unique_profiles[@]}"; do
      if [ "$p" = "$CHOSEN" ]; then found=1; break; fi
    done
    if [ $found -eq 0 ]; then
      echo "Warning: profile '$CHOSEN' not found in ~/.aws/config or credentials. Script will still attempt to use it."
    fi
  else
    echo "Select AWS profile to use:"
    PS3=$'Enter number (or 0 to type a custom profile): '
    select opt in "${unique_profiles[@]}" "Type custom profile"; do
      if [ -n "$opt" ]; then
        if [ "$opt" = "Type custom profile" ]; then
          read -rp "Profile name: " customp
          if [ -z "$customp" ]; then
            echo "Empty profile — try again."
            continue
          fi
          CHOSEN="$customp"
        else
          CHOSEN="$opt"
        fi
        break
      else
        echo "Invalid selection. Try again."
      fi
    done
  fi
fi

# Export AWS_PROFILE (this affects current shell only when sourced)
export AWS_PROFILE="$CHOSEN"
echo "Exported AWS_PROFILE=$AWS_PROFILE"

# Run SSO login for the chosen profile
echo "Running: aws sso login --profile \"$AWS_PROFILE\""
if ! aws sso login --profile "$AWS_PROFILE"; then
  echo "aws sso login failed for profile '$AWS_PROFILE'." >&2
  exit 3
fi

echo "SSO login completed for profile '$AWS_PROFILE'."
# If script was sourced, AWS_PROFILE remains in user's shell. If executed normally, it's only for this script's process.
