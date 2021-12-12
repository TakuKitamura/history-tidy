#!/bin/bash

# set PROMPT_COMMAND to append a history line to a .history-tidy file
history_tidy_prompt="history -a ~/.history-tidy; history -cr ~/.history-tidy"
if [ -z "$PROMPT_COMMAND" ]
    then
        # not already set
        PROMPT_COMMAND=history_tidy_prompt
    else
        # already set
        PROMPT_COMMAND="${history_tidy_prompt};${PROMPT_COMMAND#;}"
fi
