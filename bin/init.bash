#!/bin/bash
history_tidy_dict_path="$HOME/.history-tidy"

if [ ! -d "$history_tidy_dict_path" ]; then
  mkdir "$history_tidy_dict_path"
fi

history_path="$history_tidy_dict_path/history"
history_tidy_prompt="history -a $history_path; history -cr $history_path; eval \$(/Users/kitamurataku/local/history-tidy/target/debug/history-tidy load)"
if [ -z "$PROMPT_COMMAND" ]
    then
        # not already set
        PROMPT_COMMAND="${history_tidy_prompt}"
    else
        # already set
        PROMPT_COMMAND="${history_tidy_prompt};${PROMPT_COMMAND#;}"
fi
