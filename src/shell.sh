export PS1_START=0

get_elapsed_time() {
    if [[ -n "$PS1_START" ]]; then
        PS1_END="${EPOCHREALTIME/,/}"
        PS1_ELAPSED="$((PS1_END - PS1_START))"
        PS1_START=
    else
        PS1_ELAPSED=0
    fi
}

export PS0='${PS1_START:0:$((PS1_START=${EPOCHREALTIME/,/},0))}'
export PROMPT_COMMAND='get_elapsed_time'
export PS1='$("<exec>" --run "$?" "\j" "$PS1_ELAPSED")'

