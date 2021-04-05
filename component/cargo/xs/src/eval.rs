pub trait Eval<'d> {
    fn eval(self, engine: Engine<'d>) -> Result<Engine<'d>>;
}
mod write;

pub fn eval<'d, S: Borrow<Script<'d>>>(
    script: S,
    engine: Engine<'d>,
) -> Result<Engine<'d>> {
    Eval::eval(script.borrow(), engine)
}

impl<'a, 'd> Eval<'d> for &'a Script<'d> {
    fn eval(self, engine: Engine<'d>) -> Result<Engine<'d>> {
        self.0
            .iter()
            .fold(Ok(engine), |engine, stmt| Eval::eval(stmt, te!(engine)))
    }
}

impl<'a, 'd> Eval<'d> for &'a Stmt<'d> {
    fn eval(self, engine: Engine<'d>) -> Result<Engine<'d>> {
        log::trace!("---- EVAL {:#?}", self);

        Ok(match self {
            &Stmt::AliasStmt(name) => {
                let mut ng = engine;
                let (name, stmt) = te!(
                    ng.aliases.remove_entry(name),
                    f!("No such alias: {}", name)
                );
                let mut ng = te!(stmt.eval(ng));
                ng.aliases.insert(name, stmt);
                ng
            }
            &Stmt::Alias(name, ref stmt) => {
                log::trace!("alias {}", name);
                let mut ng = engine;
                let name: String = name.to_owned();
                let stmt: &Stmt<'d> = stmt.as_ref();
                let stmt: Stmt<'d> = stmt.to_owned();
                ng.aliases.insert(name, stmt);
                ng
            }
            Stmt::WriteValue(rel_path, expr) => {
                log::trace!("evaluating path {:?}", rel_path);
                let mut ng = engine;
                ng = te!(rel_path.eval(ng));
                let rel_path = mem::take(&mut ng.r0);
                let rel_path: &str = te!(rel_path.borrow().try_into());

                use write::OpenFile;
                let mut file = te!(rel_path.open_file());

                log::trace!("evaluating file content {:?}", expr);
                ng = te!(expr.eval(ng));

                if let Some(engine::ProcessHandleState::Pipeline(mut line)) =
                    ng.h0_proc.take()
                {
                    log::trace!("Writing to {} from pipeline", rel_path);
                    let child = te!(
                        line.last_mut(),
                        format!(
                            "[FAILURE] empty pipeline as source to {}?",
                            rel_path
                        )
                    );

                    let child: &mut std::process::Child = child;
                    if let Some(stdout) = child.stdout.as_mut() {
                        log::trace!("slurping bytes into {} ...", rel_path);
                        let len = te!(io::copy(stdout, &mut file));
                        log::debug!("written {} bytes to {}", len, rel_path);
                    } else {
                        log::warn!(
                            "No stdout on last child of pipeline? {:?}",
                            child
                        );
                    }

                    let ng = line.into_iter().fold(Ok(()), |ok, child| {
                        let ok = te!(ok);
                        let mut h_proc = ng.assume_child(child);
                        te!(
                            h_proc.wait(),
                            f!("Waiting for pipeline item {:?}", h_proc)
                        );

                        te!(h_proc.complete_success());
                        Ok(ok)
                    });
                    te!(ng)
                } else {
                    let cont = te!(json::to_string(&ng.r0));
                    let cont: &str = &cont;
                    let bytes = cont.as_bytes();
                    let len = bytes.len();
                    log::trace!("writting {} bytes to {}", len, rel_path);
                    te!(io::Write::write_all(&mut file, cont.as_bytes()));
                    log::debug!("written {} bytes to {}", len, rel_path);
                }
                ng
            }
            Stmt::WriteFile(rel_path, expr) => {
                log::trace!("evaluating path {:?}", rel_path);
                let mut ng = engine;
                ng = te!(rel_path.eval(ng));
                let rel_path = mem::take(&mut ng.r0);
                let rel_path: &str = te!(rel_path.borrow().try_into());

                use write::OpenFile;
                let mut file = te!(rel_path.open_file());

                log::trace!("evaluating file content {:?}", expr);
                ng = te!(expr.eval(ng));

                match (ng.h0_proc.take(), ng.r0.borrow()) {
                    (
                        Some(engine::ProcessHandleState::Pipeline(mut line)),
                        _,
                    ) => {
                        log::trace!("Writing to {} from pipeline", rel_path);
                        let child = te!(
                            line.last_mut(),
                            format!(
                                "[FAILURE] empty pipeline as source to {}?",
                                rel_path
                            )
                        );

                        let child: &mut std::process::Child = child;
                        if let Some(stdout) = child.stdout.as_mut() {
                            log::trace!(
                                "slurping bytes into {} ...",
                                rel_path
                            );
                            let len = te!(io::copy(stdout, &mut file));
                            log::debug!(
                                "written {} bytes to {}",
                                len,
                                rel_path
                            );
                        } else {
                            log::warn!(
                                "No stdout on last child of pipeline? {:?}",
                                child
                            );
                        }

                        let ng = line.into_iter().fold(Ok(()), |ok, child| {
                            let ok = te!(ok);
                            let mut h_proc = ng.assume_child(child);
                            te!(
                                h_proc.wait(),
                                f!("Waiting for pipeline item {:?}", h_proc)
                            );

                            te!(h_proc.complete_success());
                            Ok(ok)
                        });
                        te!(ng)
                    }
                    (_, engine::Value::Str(cont)) => {
                        let bytes = cont.as_bytes();
                        let len = bytes.len();
                        log::trace!("writting {} bytes to {}", len, rel_path);
                        te!(io::Write::write_all(&mut file, bytes));
                        log::debug!("written {} bytes to {}", len, rel_path);
                    }
                    (_, engine::Value::Bytes(cont)) => {
                        let bytes = cont.as_ref();
                        let len = bytes.len();
                        log::trace!("writting {} bytes to {}", len, rel_path);
                        te!(io::Write::write_all(&mut file, bytes));
                        log::debug!("written {} bytes to {}", len, rel_path);
                    }
                    (_, other) => {
                        err!(format!(
                            "file content of expr not a string or bytes: {:?}",
                            other
                        ))
                    }
                }

                ng
            }
            Stmt::List(stmts) => {
                let ng = stmts
                    .into_iter()
                    .fold(Ok(engine), |ng, stmt| stmt.eval(te!(ng)));
                te!(ng)
            }
            Stmt::Loop(body) => {
                let mut ng = engine;
                loop {
                    ng = te!(body.eval(ng));
                }
            }
            Stmt::Let(name, body) => te!(body.eval(engine))
                .set_var(ng::with_name(name), ng::with_val_id()),
            Stmt::Expr(expr) => {
                te!(expr.eval(engine))
            }
            &Stmt::Exec {
                cmd: cmd_name @ "mkdir",
                ref args,
                ref output,
                ref stdin,
                cwd,
                allow_failure,
                ..
            } => {
                let mut ng = engine;

                let mut argvals = Vec::new();
                argvals.push("-".to_owned()); // exec name
                for arg in args {
                    ng = te!(Eval::eval(arg, ng));

                    let arg: &str = te!(
                        ng.r0.borrow().try_into(),
                        f!("arg for {}", cmd_name)
                    );

                    argvals.push(arg.to_owned());
                }
                let mut stdout = Vec::<u8>::new();
                fn app<I: io::Read, O: io::Write>(
                    argvals: Vec<String>,
                    _: I,
                    _: O,
                ) -> Result<()> {
                    let mut parent = false;
                    let mut verbose = false;
                    let mut paths = Vec::<String>::new();
                    for arg in argvals {
                        if arg.starts_with("-") {
                            let opts = &arg[1..];
                            for opt in opts.chars() {
                                if opt == 'p' {
                                    parent = true;
                                } else if opt == 'v' {
                                    verbose = true;
                                } else {
                                    err!(format!("Unknown opt: {}", opt));
                                }
                            }
                        } else {
                            paths.push(arg);
                        }
                    }
                    if !parent {
                        const MSG: &str = "mkdir WITHOUT -p is not supported";
                        log::error!("{}", MSG);
                        err!(MSG);
                    }
                    log::debug!(
                        "mkdir -{}{} {}",
                        if parent { "p" } else { "" },
                        if verbose { "v" } else { "" },
                        paths.iter().fold(String::new(), |mut s, p| {
                            s.push_str(p);
                            s.push_str(" ");
                            s
                        })
                    );
                    for path in paths {
                        if verbose {
                            eprintln!("mkdir -pv {}", path);
                        }
                        let err_msg = format!("mkdir -p {}", path);
                        te!(fs::create_dir_all(path), err_msg);
                    }
                    Ok(())
                }

                if let Some(cwd) = cwd {
                    if stdin != &ast::Io::Default {
                        log::warn!(
                            "Using stdin: with cwd: Some() is ignored for mkdir built-in"
                        );
                    }
                    argvals = argvals
                        .into_iter()
                        .filter_map(|mut arg| {
                            if arg == "-" {
                                None
                            } else if arg.starts_with("-") {
                                Some(arg)
                            } else {
                                arg.insert(0, '/');
                                arg.insert_str(0, cwd);
                                Some(arg)
                            }
                        })
                        .collect();
                }
                if stdin != &ast::Io::Default {
                    log::warn!(
                        "Using stdin: without cwd: Some() has no effect"
                    );
                }
                te!(app(argvals, io::empty(), &mut stdout));

                match output {
                    ast::Type::Display => {
                        te!(io::Write::write_all(&mut io::stdout(), &stdout))
                    }
                    _ => {}
                };

                if allow_failure {
                    log::warn!(
                        "Allow failure has no effect for built-in mkdir"
                    )
                }

                ng.r0 = te!(output::parse(stdout, &output));
                ng
            }
            &Stmt::Exec {
                cmd: cmd_name @ "xc",
                ref args,
                ref output,
                ref stdin,
                cwd,
                allow_failure,
                ..
            } => {
                let mut ng = engine;

                let mut argvals = Vec::new();
                argvals.push("-".to_owned()); // exec name
                for arg in args {
                    ng = te!(Eval::eval(arg, ng));

                    let arg: &str = te!(
                        ng.r0.borrow().try_into(),
                        f!("arg for {}", cmd_name)
                    );

                    argvals.push(arg.to_owned());
                }
                let mut stdout = Vec::<u8>::new();
                use xc::lib::lib::app;

                if let Some(cwd) = cwd {
                    match stdin {
                        &ast::Io::Source(name) => {
                            let stmt = ng.aliases.get(name);
                            let stmt = te!(
                                stmt,
                                format!(
                                    "Alias {:?} as source not found",
                                    name
                                )
                            );
                            ng = te!(stmt.clone().eval(ng));
                            match ng.h0_proc.take() {
                                Some(
                                    engine::ProcessHandleState::Pipeline(
                                        mut children,
                                    ),
                                ) => {
                                    let child = te!(children.last_mut(), format!(
                                        "[FAILURE] not children in initialized pipeline?"
                                    ));
                                    if let Some(pipeout) = child.stdout.take()
                                    {
                                        te!(app(argvals, pipeout, &mut stdout))
                                    } else {
                                        err!("XC IS LEFT WITH NOHTING IN THE WORLD!")
                                    }
                                }
                                other => {
                                    err!(format!("[FAILURE] Invalid ProcessHandleState passed to h0_proc: {:#?}", other));
                                }
                            }
                        }
                        ast::Io::Tty => {
                            err!(format!(
                                "Tty input not supported with xc built-in"
                            ))
                        }
                        ast::Io::Default => {
                            te!(app(argvals, io::stdin(), &mut stdout))
                        }
                        ast::Io::File(path) => {
                            ng = te!(path.eval(ng));
                            let path: &str = te!(ng.r0.borrow().try_into());
                            let path = format!("{}/{}", cwd, path);
                            log::debug!("reading {} for stdin", path);
                            let file = te!(fs::File::open(&path), path);

                            let ng = te!(app(
                                argvals,
                                io::BufReader::new(file),
                                &mut stdout
                            ));
                            ng
                        }
                    }
                } else {
                    if stdin != &ast::Io::Default {
                        log::warn!(
                            "Using stdin: without cwd: Some() has no effect"
                        );
                    }
                    te!(app(argvals, io::stdin(), &mut stdout))
                }

                match output {
                    ast::Type::Display => {
                        te!(io::Write::write_all(&mut io::stdout(), &stdout))
                    }
                    ast::Type::Stream => {
                        err!("Not supported: stream output for built-in xc")
                    }
                    _ => {
                        ng.r0 = te!(output::parse(stdout, &output));
                    }
                };

                if allow_failure {
                    log::warn!("Allow failure has no effect for built-in xc")
                }

                ng
            }
            &Stmt::Exec {
                cmd: ref cmd_name,
                ref env,
                ref args,
                ref output,
                ref stdin,
                cwd,
                allow_failure,
            } => {
                use std::process::{
                    Command as Cmd,
                    Stdio,
                };

                let mut ng = engine;
                let mut cmd = Cmd::new(cmd_name);
                for arg in args {
                    ng = te!(Eval::eval(arg, ng));

                    let arg = engine::Strval(String::new());
                    let arg = arg.as_argument(&ng.r0);
                    let arg = te!(arg, f!("arg for {}", cmd_name));

                    cmd.arg(arg);
                }
                for (name, val) in env {
                    ng = te!(Eval::eval(val, ng));
                    let val: &str = te!(
                        ng.r0.borrow().try_into(),
                        f!("env value: {}: {:?}", name, val)
                    );
                    cmd.env(name, val);
                }

                if let Some(cwd) = cwd {
                    cmd.current_dir(cwd);
                    match stdin {
                        ast::Io::Default => cmd.stdin(Stdio::null()),
                        ast::Io::Tty => cmd.stdin(Stdio::inherit()),
                        ast::Io::File(path) => {
                            ng = te!(path.eval(ng));
                            let path: &str = te!(ng.r0.borrow().try_into());
                            let path = format!("{}/{}", cwd, path);
                            log::debug!("reading {} for stdin", path);
                            let file = te!(fs::File::open(&path), path);
                            cmd.stdin(Stdio::from(file))
                        }
                        &ast::Io::Source(name) => {
                            let stmt = ng.aliases.get(name);
                            let stmt = te!(
                                stmt,
                                format!(
                                    "Alias {:?} as source not found",
                                    name
                                )
                            );
                            let stmt = stmt.clone();
                            ng = te!(stmt.eval(ng));
                            match ng.h0_proc.take() {
                                Some(
                                    engine::ProcessHandleState::Pipeline(
                                        mut children,
                                    ),
                                ) => {
                                    let child = te!(
                                        children.last_mut(),
                                        format!("[FAILURE] not children in initialized pipeline?")
                                    );
                                    if let Some(stdout) = child.stdout.take() {
                                        cmd.stdin(Stdio::from(stdout))
                                    } else {
                                        cmd.stdin(Stdio::null())
                                    }
                                }
                                other => {
                                    err!(format!("[FAILURE] Invalid ProcessHandleState passed to h0_proc: {:#?}", other));
                                }
                            }
                        }
                    };
                } else if stdin != &ast::Io::Default {
                    log::warn!(
                        "Using stdin: without cwd: Some() has no effect"
                    );
                }

                match output {
                    ast::Type::Display => cmd.stdout(Stdio::inherit()),
                    _ => cmd.stdout(Stdio::piped()),
                };

                let mut h_proc = ng.exec(cmd);
                te!(h_proc.start(), f!("Starting {}", cmd_name));

                match output {
                    ast::Type::Stream => {
                        te!(h_proc.return_to_h0());
                    }

                    _ => {
                        te!(h_proc.wait(), f!("Waiting {}", cmd_name));

                        let proc = te!(if allow_failure {
                            h_proc.complete()
                        } else {
                            h_proc.complete_success()
                        });

                        ng.r0 = te!(output::parse(proc.stdout, &output));
                    }
                }

                ng
            }
            &Stmt::ForEach(input_var, ref body) => {
                let mut ng = engine;

                let (_, var_value) = te!(ng.variables.remove_entry(input_var));
                let var_list: Vec<Value> = te!(var_value.try_into());

                for var_list_item in var_list {
                    ng.each = var_list_item;
                    ng = te!(body.eval(ng));
                }

                ng
            }
            &Stmt::Clone(name) => {
                let mut ng = engine;
                ng.r0 =
                    te!(ng.variables.get(name), f!("Var not found: {}", name))
                        .clone();
                ng
            }
        })
    }
}

impl<'a, 'd> Eval<'d> for &'a Expr<'d> {
    fn eval(self, mut ng: Engine<'d>) -> Result<Engine<'d>> {
        Ok(match *self {
            Expr::ReadSource { source } => {
                let source_stmt = te!(
                    ng.aliases.get(source),
                    format!("Not found alias {} as source", source)
                )
                .clone();
                ng = te!(source_stmt.eval(ng));
                match ng.h0_proc.take() {
                    Some(p @ engine::ProcessHandleState::Pipeline(_)) => {
                        ng.h0_proc = Some(p);
                        ng
                    }
                    other => err!(format!(
                        "Source {} is not a pipeline: {:?}",
                        source, other
                    )),
                }
            }
            Expr::Bs { ref bs } => {
                ng.r0 = Value::from(bs.clone());
                ng
            }
            Expr::Str2 { s } | Expr::Str(s) => {
                ng.set_r0(move |_park, _value| {
                    _park.drain(_value);
                    Value::Str(Cow::from(s))
                })
            }
            Expr::List(ref exprs) => {
                let mut ss = Vec::new();
                for expr in exprs {
                    ng = te!(expr.eval(ng));
                    ss.push(mem::take(&mut ng.r0));
                }
                ng.r0 = Value::from(ss);
                ng
            }
            Expr::Each => ng.r0_load_each(),
            Expr::Var { var } => {
                log::trace!("var[{}]: {:?}", var, ng.variables);
                let var = te!(
                    ng.variables.get(var),
                    format_args!("Var not found: {:?}", var)
                );
                ng.r0 = var.clone();
                ng
            }
        })
    }
}

use self::engine as ng;
use super::*;
use ast::*;
use engine::Value;
use std::format_args as f;
