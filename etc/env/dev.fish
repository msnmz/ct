bind -M insert \cr 'tmux send-keys -t ct:Edit.2 Space'
bind -M insert \ct 'tmux send-keys -t ct:Edit.2 q ; tmux respawn-pane -t ct:Edit.2'
