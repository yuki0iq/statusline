exec {PS1_FD}<> <(true)

export PS1_START="${EPOCHREALTIME/[.,]/}"

trap 'echo >&$PS1_FD' DEBUG
export PS0='${PS1_START:0:$((PS1_START=${EPOCHREALTIME/[.,]/}, 0))}'
export PROMPT_COMMAND='
    if [[ -n "$PS1_START" ]]; then
        PS1_END="${EPOCHREALTIME/[.,]/}"
        PS1_ELAPSED="$((PS1_END - PS1_START))"
        PS1_START=
    else
        PS1_ELAPSED=0
    fi
    wait -n
    printf "\n\n"
'
export PS1='$("<exec>" --run "$?" "\j" "$PS1_ELAPSED" <&$PS1_FD &)'

