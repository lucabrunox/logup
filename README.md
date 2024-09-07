## Outlog use cases

Outlog is a UNIX-style command that can be used to pipe stdout to a logs location on disk or in the cloud without the need of an agent, logrotate, systemd or other configuration files. For example it can:
- Pipe stdout to cloud providers such as AWS Logs without the need to install an agent.
- Pipe stdout to disk files with log rotation, without the need to set up logrotate. (Not implemented yet)

Outlog is resilient: it does buffering to temp files to prevent the application from ever blocking when writing to stdout. (Not implemented yet)

Outlog is transparent: it passes through the original stdout without any additional info or error messages. (Not implemented yet)

For example:
```bash
$ echo foo | outlog --aws-log-group-name '/test/foo'
foo
```

The above command passed through the "foo" to stdout, but also created an AWS log group "/test/foo" with log stream name equal to the hostname, and sent the log "foo".

## Outlog installation

To install in ~/.cargo/bin:

```bash
git clone https://github.com/lucabrunox/outlog
cd outlog
cargo install --path .
```

## Outlog command line parameters

```bash
$ outlog --help
Usage: outlog [OPTIONS]

Options:
      --aws-log-group-name <AWS_LOG_GROUP_NAME>    
      --aws-log-stream-name <AWS_LOG_STREAM_NAME>  
  -h, --help                                       Print help
  -V, --version                                    Print version
```

## Outlog roadmap

- [X] Send logs to AWS Logs
- [ ] Read from file instead of just stdout
- [ ] Splitting by lines
- [ ] Buffering in-memory
- [ ] Buffering on-disk
- [ ] Output to disk files with log rotation
- [ ] Compression
- [ ] Logging of Outlog itself to disk
- [ ] Distributions
  - [ ] Cargo
  - [ ] Tar
  - [ ] Deb
  - [ ] Rpm
- [ ] Support more cloud providers
  - [ ] Azure
  - [ ] Google
  - [ ] Datadog

## Outlog license

License is GPLv3: https://www.gnu.org/licenses/gpl-3.0.html#license-text

All contributions are welcome!
