#!/usr/local/bin/xs -f
// vim: et ts=4 sw=4 ft=ron
Script([

    // All files exactly under DIR, NUL-separated list
    Alias ("files",
        Exec (output: Stream,
            cmd: "gfind",
            args: [
                (var: "source_dir"),
                "-mindepth", "1",
                "-maxdepth", "1",
                "-type", "f",
                "-execdir",
                    "xs",
                    "-ah", "file name", "{}",
                    "-c",
                    r####"
                        Script([
                            WriteFile("/dev/stdout", (var: "file name")),
                            WriteFile("/dev/stdout", (bs: [0])),
                        ])
                    "####,
                ";"
            ],
            env: {
                "PATH": "/usr/local/bin",
            },
        )
    ),

    WriteFile ("/dev/stdout",
        r####"colorscheme apprentice
set number
"####),

    Alias ("output",
        Exec (output: Stream,
            cmd: "xargs",
            args: [
                "-0", "-n1", "-I:::",
                "xs",
                "-ah", "file", ":::",
                "-ah", "source_namespace", (var: "source_namespace"),
                "-ah", "source_dir", (var: "source_dir"),
                "-f", "dev_exec/::sanctioned/dev:make_one_source_html",
            ],
            stdin: Source("files"), cwd: Some("."),
        )
    ),
    WriteFile ("/dev/stdout", (source: "output")),

    WriteFile ("/dev/stdout",
        r####"
qall!
"####),

])//Script
