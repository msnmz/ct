#!/usr/local/bin/xs -f
// vim: et ts=4 sw=4 ft=ron
Script([

Let ("names", Exec (output: String, cmd: "xs", args: ["-f",
    "dev_exec/::sanctioned/config::vagrant:enabled"
])),

Alias ("Vagrantfile.rb", Exec (output: Stream, cmd: "xs", args: ["-f",
    "dev_exec/::sanctioned/vagrant:Vagrantfile.rb", "-ah",
    "names", (var: "names"),
])),

WriteFile ("/dev/stdout", (source: "Vagrantfile.rb")),

])//Script
