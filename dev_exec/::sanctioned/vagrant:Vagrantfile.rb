#!/usr/local/bin/xs -f
// vim: et ts=4 sw=4 ft=ron
Script([

//
// Streamify util
//
Alias ("streamify", Exec(output: Stream,
    cmd: "xs", args: ["-ah", "str", (var: "streamify.in:str"), "-c", r####"
    Script([
       WriteFile ("/dev/stdout", (var: "str")) ,
    ])
    "####],
)),

//
// JQ Scripts
//
Let ("Preprocess with box names (JQ)", Expr (r####"

    # $db provided as JQ argument
    # $names provided as JQ argument

    $names[] as $vm_name

    | $db.box_instance[$vm_name]                    as $instance
    | $db.BoxOs[$instance.os][$instance.provider]   as $os

    | $instance.network_allocation                  as $net_alloc_id
    | $db.BoxNetworkAllocation[$net_alloc_id]       as $net_alloc

    | ($instance.size // "x-small")                 as $size
    | $db.BoxSize[$size]                            as $size

    | (
        if false then empty
        else if $instance.provider == "vagrant_vbox" then "virtualbox"
        else if $instance.provider == "vagrant_docker" then "docker"
        else ("Unknown provider: \($instance.provider)" | error)
        end end end
    )                                               as $provider_type

    | (
        if $instance.provider == "vagrant_vbox"
        then
            $size[$instance.provider] + {
            }
        else if $instance.provider == "vagrant_docker"
        then
            $os
        else
            "Unknown provider: \($instance)" | error
        end end
        | to_entries
    )                                               as $provider_attr

    | (
        if $instance.provider == "vagrant_vbox"
        then
            [
                "['modifyvm', :id, '--graphicscontroller', 'vmsvga']",
                "['modifyvm', :id, '--accelerate3d', 'on']",
                "['modifyvm', :id, '--audio', 'none']",
                "['modifyvm', :id, '--clipboard-mode', 'disabled']",
                "['modifyvm', :id, '--draganddrop', 'disabled']",
                "['modifyvm', :id, '--vrde', 'off']",
                "['modifyvm', :id, '--teleporter', 'off']",
                "['modifyvm', :id, '--tracing-enabled', 'off']",
                "['modifyvm', :id, '--usbcardreader', 'off']",
                "['modifyvm', :id, '--recording', 'off']"
            ]
        else
            []
        end
    )                                               as $provider_customize

    # Theoretically docker boxes on vagrant don't need a "box" declaration,
    # but there is an error(?) about box missing, so we inject a krap value.
    | (
        $os |
        if $instance.provider == "vagrant_docker" then .box = "generic/alpine38" else . end
    )                                               as $os

    | {
        #$db,
        #$instance,
        $net_alloc,
        $os,
        $provider_attr,
        $provider_customize,
        $provider_type,
        #$size,
        $vm_name,
    }

"####)), // JQ


Let ("Box entries to Vagrantfile.rb entries (JQ)", Expr (r####"

    @json   "  config.vm.define \(.vm_name) do |config|",
    @json   "    config.vm.box = \(.os.box // empty)",
    @json   "    config.vm.usable_port_range = \(.net_alloc.port_range)",
    @json   "    config.vm.provider \(.provider_type) do |provider|",

    (       .provider_attr[] |
            "      provider.\(.key) = \(.value | @json)"
    ),
    (       .provider_customize[] |
            "      provider.customize \(.)"
    ),
    @json   "    end",
            "  end"

"####)),


//
// Database (pure)
//
Let ("db", Exec (output: String,
    cmd: "xs",
    args: ["-f",
        "dev_exec/::sanctioned/db::",
        "-ah", "component", "vagrant",
        "-ah", "selector", "$db"
    ],
)),


//
// Query DB => (result + DB)
//

// Turn command line args "names" to a steam.
Let ("streamify.in:str", Expr((var: "names"))),
Alias ("names:stream", AliasStmt("streamify")),

// Turn argument "names" from YAML to JSON
Let ("names.json", Exec (output: String,
    cmd: "xc", args: ["yj"],
    stdin: Source("names:stream"), cwd: Some("."),
)),
//WriteFile ("/dev/stderr", (var: "names.json")),

// Resolve box names to box_instances
Alias ("box_instances.jsons", Exec (output: Stream,
    cmd: "jq", args: [
        "--exit-status",
        "--null-input",
        "--argjson", "db", (var: "db"),
        "--argjson", "names", (var: "names.json"),
        (var: "Preprocess with box names (JQ)"),
    ],
)),
//WriteFile ("/dev/stderr", (source: "box_instances.jsons")),

Alias ("Vagrantfile.rb-entries", Exec (output: Stream,
    cmd: "jq", args: [
        "--exit-status",
        "--raw-output",
        "--argjson", "db", (var: "db"),
        "--argjson", "names", (var: "names.json"),

        (var: "Box entries to Vagrantfile.rb entries (JQ)")
    ],
    stdin: Source("box_instances.jsons"), cwd: Some("."),
)),

WriteFile ("/dev/stdout", "Vagrant.configure '2' do |config|"),
WriteFile ("/dev/stdout", (bs: [0x0a])),

WriteFile ("/dev/stdout", (source: "Vagrantfile.rb-entries")),

WriteFile ("/dev/stdout", "end"),

])//Script
