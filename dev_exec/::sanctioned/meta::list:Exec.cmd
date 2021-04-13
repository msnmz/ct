#!/usr/local/bin/xs -f
// vim: et ts=4 sw=4 ft=ron
Script([//-0

    Alias ("files",
        Exec (output: Stream,
            cmd: "find",
            args: [
                "dev_exec",
                "-type", "f",
                "-print0",
            ]
        )
    ),

    Alias ("commands",
        Exec (output: Stream,
            cmd: "xargs",
            stdin: Source("files"), cwd: Some("."),
            args: [
                "-0", "-I:::",
                "xs",
                "-ah", "file", ":::",
                "-c",
                r#####"
                    Script([//-1
                        Alias ("input",
                            Exec (output: Stream,
                                cmd: "cat",
                                args: [(var: "file")],
                            )
                        ),
                        Alias ("debang",
                            Exec (output: Stream,
                                cmd: "sed",
                                stdin: Source("input"), cwd: Some("."),
                                args: [
                                    "/^#!/d"
                                ]
                            )
                        ),
                        Let ("json",
                            Exec (output: String,
                                cmd: "xc",
                                args: ["rj"],
                                stdin: Source("debang"), cwd: Some("."),
                            )
                        ),
                        Alias ("commands",
                            Exec (output: Stream,
                                cmd: "jq",
                                args: [
                                    "--null-input",
                                    "--raw-output",
                                    "--argjson", "script", (var: "json"),
                                    r####"$script
                                    | .[0] # Script()
                                    | .[] as { $cmd, $args } ?// $__
                                    | if $cmd != null
                                        then $cmd
                                        else empty end
                                    "####
                                ]
                            )
                        ),
                        WriteFile ("/dev/stdout", (source: "commands")),
                    ])//Script-1
                "#####
            ]
        )
    ),
    //WriteFile ("/dev/stderr", (source: "commands")),
    Alias ("sort",
        Exec (output: Stream,
            cmd: "sort",
            stdin: Source("commands"), cwd: Some("."),
        )
    ),
    //WriteFile ("/dev/stderr", (source: "sort")),
    Alias ("uniq",
        Exec (output: Stream,
            cmd: "uniq",
            stdin: Source("sort"), cwd: Some("."),
        )
    ),
    WriteFile ("/dev/stdout", (source: "uniq")),
])//Script-0
