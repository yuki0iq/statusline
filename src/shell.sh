set -x

PS1_PIPE="$(mktemp -u /tmp/statusline-XXXXXXXX)"
mkfifo $PS1_PIPE
sleep inf >$PS1_PIPE &

VIRTUAL_ENV_DISABLE_PROMPT=1
PS1_START="$(date +%s%N | head -c -4)"
PS1_ELAPSED=0

# trap 'echo >&$PS1_FD' DEBUG
# PS0='${PS1_START:0:$((PS1_START=${EPOCHREALTIME/[.,]/}, 0))}'
# PS0='${PS1_START:0:$((PS1_START=$(echo >$PS1_PIPE; date +%s%N | head -c -7), 0))}'
PS1='\j $(export EXIT_CODE=$?; [[ -n "$PS1_START" ]] && export PS1_ELAPSED="$(($(date +%s%N | head -c -4) - PS1_START))" || export PS1_ELAPSED=0 ; "<exec>" --run "$EXIT_CODE" "0" "$PS1_ELAPSED" 3<$PS1_PIPE & )'

alias ssh='WORKGROUP_CHAIN="$("<exec>" --ssh-new-connection)" ssh -o "SendEnv=WORKGROUP_CHAIN"'

