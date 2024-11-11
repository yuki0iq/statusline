exec {PS1_FD}<> <(true)

VIRTUAL_ENV_DISABLE_PROMPT=1
PS1_START="${EPOCHREALTIME/[.,]/}"

trap 'echo >&$PS1_FD' DEBUG
PS0='${PS1_START:0:$((PS1_START=${EPOCHREALTIME/[.,]/}, 0))}'
PROMPT_COMMAND='
    if [[ -n "$PS1_START" ]]; then
        PS1_END="${EPOCHREALTIME/[.,]/}"
        PS1_ELAPSED="$((PS1_END - PS1_START))"
        PS1_START=
    else
        PS1_ELAPSED=0
    fi
    jobs -n
'
PS1='$("<exec>" run --return-code "$?" --jobs-count "\j" --elapsed-time "$PS1_ELAPSED" --control-fd 3 3<&$PS1_FD &)'

alias ssh='WORKGROUP_CHAIN="$("<exec>" chain)" ssh -o "SendEnv=WORKGROUP_CHAIN"'

