#!/bin/bash
history_tidy_prompt () {
    local history_tidy_dict_path="$HOME/.history-tidy"

    if [ ! -d "$history_tidy_dict_path" ]; then
        mkdir "$history_tidy_dict_path"
    fi

    local history_path="$history_tidy_dict_path/history"
    history -a $history_path;
    history -cr $history_path;
    local exec_command=$(cat $history_tidy_dict_path/script);

    if [ -z "$exec_command" ]
    then
        history_tidy_status=0
        return 0;
    fi
    
    echo > $history_tidy_dict_path/script

    while :
    do
        echo $exec_command
        read -p "Do you exec this command? [Y/n] " choice
        case "$choice" in
            [Yy])
                eval $exec_command;
                history_tidy_status=$?
                return $?;
                ;;
            [Nn])
                echo Abort.
                history_tidy_status=$?
                return 1;
                ;;
            *)
                if [ -z "$choice" ]
                then
                    eval $exec_command;
                    history_tidy_status=$?
                    return $?;
                fi
                ;;
        esac
    done
}

if [ -z "$PROMPT_COMMAND" ]
    then
        # not already set
        PROMPT_COMMAND="history_tidy_prompt;"
    else
        # already set
        PROMPT_COMMAND="history_tidy_prompt;${PROMPT_COMMAND#;}"
fi
