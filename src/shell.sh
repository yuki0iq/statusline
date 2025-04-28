# Fix mojibake by enforcing text mode or disabling statusline altogether
case $TERM in
    dumb)
        return
        ;;
    linux|tmux-*)
        _sl_mode=text
        ;;
    *)
        _sl_mode=$PS1_MODE
        ;;
esac

# Disable process lingering if it's not being needed
exec {_sl_control_fd}<> <(true)
trap 'echo >&$_sl_control_fd' DEBUG

# Disable features that are already covered by statusline
MAILCHECK=-1
VIRTUAL_ENV_DISABLE_PROMPT=1

_sl_stamp() {
    echo "${EPOCHREALTIME/[.,]/}";
}

declare -g _sl_start=$(_sl_stamp)
declare -g _sl_elapsed

_sl_prompt_command() {
    if [[ -n "$_sl_start" ]]; then
        _sl_elapsed="$(($(_sl_stamp) - _sl_start))"
        _sl_start=
    else
        _sl_elapsed=0
    fi

    jobs -n
}

PS0='${SHELL:0:0$((_sl_start=$(_sl_stamp), 0))}'
PROMPT_COMMAND='_sl_prompt_command'
PS1='$("<exec>" run --mode "$_sl_mode" --return-code "$?" --jobs-count "\j" --elapsed-time "$_sl_elapsed" --control-fd 3 3<&$_sl_control_fd &)'

# Nice features
alias ssh='WORKGROUP_CHAIN="$("<exec>" chain)" ssh -o "SendEnv=WORKGROUP_CHAIN"'

