function ls
  lsd $argv
end

function ll
  ls -Al $argv
end

function lt
  ls --tree $argv
end

function lsdd
  ll -R $argv dev_exec/::sanctioned dev_exec/
end
