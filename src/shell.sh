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
    ("<exec>" --top 3<&$PS1_FD &)
    PS1="$("<exec>" --bottom "$?" "$(($(jobs | wc -l) - 1))" "$PS1_ELAPSED")"
'
PS1='$USER@$HOSTNAME:$PWD> '

